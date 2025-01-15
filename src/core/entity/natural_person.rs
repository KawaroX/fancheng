use crate::FanError;
use crate::FanResult;

use crate::core::entity::base::{BaseEntity, CapacityStatus, Entity, EntityType, NaturalCapacity};
use chrono::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

/// 精神状态
#[derive(Debug, Clone, PartialEq)]
pub enum MentalStatus {
    Normal,            // 正常
    PartiallyImpaired, // 部分受损
    SeverelyImpaired,  // 严重受损
}

/// 监护关系
#[derive(Debug, Clone)]
pub struct Guardianship {
    guardian: Uuid,                     // 监护人ID
    ward: Uuid,                         // 被监护人ID
    scope: GuardianshipScope,           // 监护范围
    created_at: DateTime<Utc>,          // 监护关系建立时间
    valid_until: Option<DateTime<Utc>>, // 监护关系有效期
}

/// 监护范围
#[derive(Debug, Clone)]
pub struct GuardianshipScope {
    permitted_actions: HashSet<String>, // 允许的行为类型
}

/// 自然人
#[derive(Debug, Clone)]
pub struct NaturalPerson {
    base: BaseEntity,
    birth_date: DateTime<Utc>,
    mental_status: MentalStatus,
    guardian: Option<Guardianship>,
    is_guardian: bool,
}

impl NaturalPerson {
    pub fn new(birth_date: DateTime<Utc>, mental_status: MentalStatus) -> Self {
        let now = Utc::now();
        let capacity = Self::evaluate_capacity(&birth_date, &mental_status);

        Self {
            base: BaseEntity {
                id: Uuid::new_v4(),
                entity_type: EntityType::NaturalPerson,
                capacity_status: CapacityStatus::NaturalPerson(capacity),
                created_at: now,
                updated_at: now,
            },
            birth_date,
            mental_status,
            guardian: None,
            is_guardian: false,
        }
    }

    /// 计算年龄
    pub fn age(&self) -> u8 {
        let now = Utc::now();
        let age = now.year() - self.birth_date.year();
        age as u8 // 简化的计算，实际应该考虑月份和日期
    }

    /// 评估行为能力
    fn evaluate_capacity(
        birth_date: &DateTime<Utc>,
        mental_status: &MentalStatus,
    ) -> NaturalCapacity {
        let now: DateTime<Utc> = Utc::now();
        let age = (now.year() - birth_date.year()) as u8;

        match (age, mental_status) {
            (age, MentalStatus::Normal) if age >= 18 => NaturalCapacity::Full,
            (age, MentalStatus::Normal) if age >= 8 => NaturalCapacity::Limited,
            (_, MentalStatus::PartiallyImpaired) => NaturalCapacity::Limited,
            (_, MentalStatus::SeverelyImpaired) => NaturalCapacity::None,
            _ => NaturalCapacity::None,
        }
    }

    /// 更新精神状态并重新评估行为能力
    pub fn update_mental_status(&mut self, new_status: MentalStatus) -> FanResult<()> {
        self.mental_status = new_status;
        if let CapacityStatus::NaturalPerson(_) = &mut self.base.capacity_status {
            self.base.capacity_status = CapacityStatus::NaturalPerson(Self::evaluate_capacity(
                &self.birth_date,
                &self.mental_status,
            ));
        }
        self.base.updated_at = Utc::now();
        Ok(())
    }

    /// 设置监护人，并修改作为监护人的 NaturalPerson 实例
    pub fn set_guardian(&mut self, guardian: &mut Self, scope: GuardianshipScope) -> FanResult<()> {
        if !guardian.can_be_guardian() {
            return Err(FanError::ValidationError("Invalid guardian".to_string()));
        }

        self.guardian = Some(Guardianship {
            guardian: guardian.base.id,
            ward: self.base.id,
            scope,
            created_at: Utc::now(),
            valid_until: None,
        });
        self.base.updated_at = Utc::now();
        guardian.is_guardian = true;
        guardian.base.updated_at = Utc::now();
        Ok(())
    }

    // /// 设置监护人，并修改作为监护人的 NaturalPerson 实例
    // pub fn set_guardian(
    //     ward: &Arc<Mutex<Self>>,
    //     guardian: &Arc<Mutex<Self>>,
    //     scope: GuardianshipScope,
    // ) -> FanResult<()> {
    //     // 尝试获取监护人信息，仅持有读取所需的短时间锁
    //     let guardian_id = {
    //         let guardian = guardian.try_lock().map_err(|e| match e {
    //             TryLockError::Poisoned(_) => FanError::LockError("Guardian lock poisoned".into()),
    //             TryLockError::WouldBlock => {
    //                 FanError::LockError("Guardian lock currently in use".into())
    //             }
    //         })?;
    //         if !guardian.can_be_guardian() {
    //             return Err(FanError::ValidationError("Invalid guardian".to_string()));
    //         }
    //         guardian.base.id
    //     };

    //     // 对 ward 和 guardian 按固定顺序加锁，避免死锁
    //     let (mut ward, mut guardian) = if Arc::as_ptr(ward) < Arc::as_ptr(guardian) {
    //         let ward = ward.try_lock().map_err(|e| match e {
    //             TryLockError::Poisoned(_) => FanError::LockError("Ward lock poisoned".into()),
    //             TryLockError::WouldBlock => {
    //                 FanError::LockError("Ward lock currently in use".into())
    //             }
    //         })?;
    //         let guardian = guardian.try_lock().map_err(|e| match e {
    //             TryLockError::Poisoned(_) => FanError::LockError("Guardian lock poisoned".into()),
    //             TryLockError::WouldBlock => {
    //                 FanError::LockError("Guardian lock currently in use".into())
    //             }
    //         })?;
    //         (ward, guardian)
    //     } else {
    //         let guardian = guardian.try_lock().map_err(|e| match e {
    //             TryLockError::Poisoned(_) => FanError::LockError("Guardian lock poisoned".into()),
    //             TryLockError::WouldBlock => {
    //                 FanError::LockError("Guardian lock currently in use".into())
    //             }
    //         })?;
    //         let ward = ward.try_lock().map_err(|e| match e {
    //             TryLockError::Poisoned(_) => FanError::LockError("Ward lock poisoned".into()),
    //             TryLockError::WouldBlock => {
    //                 FanError::LockError("Ward lock currently in use".into())
    //             }
    //         })?;
    //         (ward, guardian)
    //     };

    //     // 更新 ward 和 guardian 的状态
    //     ward.guardian = Some(Guardianship {
    //         guardian: guardian_id,
    //         ward: ward.base.id,
    //         scope,
    //         created_at: Utc::now(),
    //         valid_until: None,
    //     });
    //     ward.base.updated_at = Utc::now();

    //     guardian.is_guardian = true;
    //     guardian.base.updated_at = Utc::now();

    //     Ok(())
    // }

    /// 判断是否可以作为监护人
    pub fn can_be_guardian(&self) -> bool {
        matches!(
            &self.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::Full)
        ) && self.mental_status == MentalStatus::Normal
            && self.age() >= 18
    }
}

// 给 NaturalPerson 实现 Entity trait
impl Entity for NaturalPerson {
    fn id(&self) -> Uuid {
        self.base.id
    }
    fn entity_type(&self) -> EntityType {
        self.base.entity_type.clone()
    }
    fn capacity_status(&self) -> CapacityStatus {
        self.base.capacity_status.clone()
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }
    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}

/// 线程安全的 NaturalPerson
#[derive(Clone, Debug)]
pub struct SyncNaturalPerson {
    base: Arc<RwLock<BaseEntity>>,
    birth_date: DateTime<Utc>, // 不需要锁，因为不可变
    mental_status: Arc<Mutex<MentalStatus>>,
    guardian: Arc<RwLock<Option<Guardianship>>>,
    is_guardian: Arc<Mutex<bool>>,
}

/// 给 SyncNaturalPerson（线程安全的NP） 实现 Entity trait
impl SyncNaturalPerson {
    pub fn new(birth_date: DateTime<Utc>, mental_status: MentalStatus) -> Self {
        let now = Utc::now();
        let capacity = NaturalPerson::evaluate_capacity(&birth_date, &mental_status);

        Self {
            base: Arc::new(RwLock::new(BaseEntity {
                id: Uuid::new_v4(),
                entity_type: EntityType::NaturalPerson,
                capacity_status: CapacityStatus::NaturalPerson(capacity),
                created_at: now,
                updated_at: now,
            })),
            birth_date,
            mental_status: Arc::new(Mutex::new(mental_status)),
            guardian: Arc::new(RwLock::new(None)),
            is_guardian: Arc::new(Mutex::new(false)),
        }
    }

    pub fn age(&self) -> u8 {
        let now = Utc::now();
        let age = now.year() - self.birth_date.year();
        age as u8
    }

    pub fn update_mental_status(&self, new_status: MentalStatus) -> FanResult<()> {
        let mut status = self
            .mental_status
            .lock()
            .map_err(|_| FanError::LockError("Mental status lock poisoned".into()))?;

        *status = new_status.clone();

        let mut base = self
            .base
            .write()
            .map_err(|_| FanError::LockError("Base lock poisoned".into()))?;

        if let CapacityStatus::NaturalPerson(_) = &mut base.capacity_status {
            base.capacity_status = CapacityStatus::NaturalPerson(NaturalPerson::evaluate_capacity(
                &self.birth_date,
                &new_status,
            ));
        }
        base.updated_at = Utc::now();

        Ok(())
    }

    pub fn set_guardian(
        ward: &Arc<Mutex<Self>>,
        guardian: &Arc<Mutex<Self>>,
        scope: GuardianshipScope,
    ) -> FanResult<()> {
        // 先检查监护人资格
        let guardian_id = {
            let guardian_guard = guardian
                .try_lock()
                .map_err(|_| FanError::LockError("Guardian lock poisoned".into()))?;
            if !guardian_guard.can_be_guardian()? {
                return Err(FanError::ValidationError("Invalid guardian".to_string()));
            }

            let base = guardian_guard
                .base
                .read()
                .map_err(|_| FanError::LockError("Guardian base lock poisoned".into()))?;

            base.id
        };

        // println!("{}", guardian_id);

        // 按地址顺序加锁避免死锁
        let (ward_guard, guardian_guard) = if Arc::as_ptr(ward) < Arc::as_ptr(guardian) {
            (
                ward.try_lock()
                    .map_err(|_| FanError::LockError("Ward lock poisoned".into()))?,
                guardian
                    .lock()
                    .map_err(|_| FanError::LockError("Guardian lock poisoned".into()))?,
            )
        } else {
            (
                guardian
                    .lock()
                    .map_err(|_| FanError::LockError("Guardian lock poisoned".into()))?,
                ward.lock()
                    .map_err(|_| FanError::LockError("Ward lock poisoned".into()))?,
            )
        };

        // 更新被监护人状态
        {
            let mut ward_guardian = ward_guard
                .guardian
                .write()
                .map_err(|_| FanError::LockError("Ward guardian lock poisoned".into()))?;
            let mut ward_base = ward_guard
                .base
                .write()
                .map_err(|_| FanError::LockError("Ward base lock poisoned".into()))?;

            *ward_guardian = Some(Guardianship {
                guardian: guardian_id,
                ward: ward_base.id,
                scope,
                created_at: Utc::now(),
                valid_until: None,
            });
            ward_base.updated_at = Utc::now();
        }

        // 更新监护人状态
        {
            let mut guardian_is_guardian = guardian_guard
                .is_guardian
                .lock()
                .map_err(|_| FanError::LockError("Guardian status lock poisoned".into()))?;
            let mut guardian_base = guardian_guard
                .base
                .write()
                .map_err(|_| FanError::LockError("Guardian base lock poisoned".into()))?;

            *guardian_is_guardian = true;
            guardian_base.updated_at = Utc::now();
        }

        Ok(())
    }

    pub fn can_be_guardian(&self) -> FanResult<bool> {
        let status = self
            .mental_status
            .lock()
            .map_err(|_| FanError::LockError("Mental status lock poisoned".into()))?;

        let base = self
            .base
            .read()
            .map_err(|_| FanError::LockError("Base lock poisoned".into()))?;

        Ok(matches!(
            &base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::Full)
        ) && *status == MentalStatus::Normal
            && self.age() >= 18)
    }

    // 从非线程安全版本转换
    pub fn from_natural_person(person: NaturalPerson) -> Self {
        Self {
            base: Arc::new(RwLock::new(person.base)),
            birth_date: person.birth_date,
            mental_status: Arc::new(Mutex::new(person.mental_status)),
            guardian: Arc::new(RwLock::new(person.guardian)),
            is_guardian: Arc::new(Mutex::new(person.is_guardian)),
        }
    }
}

impl Entity for SyncNaturalPerson {
    fn id(&self) -> Uuid {
        self.base.read().map(|base| base.id).unwrap_or_default()
    }

    fn entity_type(&self) -> EntityType {
        self.base
            .read()
            .map(|base| base.entity_type.clone())
            .unwrap_or_default()
    }

    fn capacity_status(&self) -> CapacityStatus {
        self.base
            .read()
            .map(|base| base.capacity_status.clone())
            .unwrap_or_default()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base
            .read()
            .map(|base| base.created_at)
            .unwrap_or_default()
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base
            .read()
            .map(|base| base.updated_at)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use uuid::Uuid;

    fn get_test_date() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()
    }

    fn get_test_guardianship_scope() -> GuardianshipScope {
        GuardianshipScope {
            permitted_actions: HashSet::from(["care".to_string(), "education".to_string()]),
        }
    }

    // 测试创建一个自然人
    #[test]
    fn test_create_natural_person() {
        let birth_date = get_test_date();
        let mental_status = MentalStatus::Normal;
        let person = NaturalPerson::new(birth_date, mental_status);

        assert_eq!(person.age(), 5); // Assuming current year is 2025
        assert_eq!(person.mental_status, MentalStatus::Normal);
    }

    // 测试自然人的行为能力评估
    #[test]
    fn test_evaluate_capacity() {
        let birth_date = get_test_date();
        let person_baby = NaturalPerson::new(birth_date, MentalStatus::Normal);

        let birth_date = Utc.with_ymd_and_hms(2014, 03, 12, 0, 0, 0).unwrap();
        let person_teenage = NaturalPerson::new(birth_date, MentalStatus::Normal);

        let birth_date = Utc.with_ymd_and_hms(2004, 03, 12, 0, 0, 0).unwrap();
        let person_adult = NaturalPerson::new(birth_date, MentalStatus::Normal);
        let person_ophelia = NaturalPerson::new(birth_date, MentalStatus::PartiallyImpaired);
        let person_madman = NaturalPerson::new(birth_date, MentalStatus::SeverelyImpaired);
        println!("{:#?}", person_ophelia);
        assert_eq!(
            person_baby.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::None)
        );
        assert_eq!(
            person_teenage.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::Limited)
        );
        assert_eq!(
            person_adult.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::Full)
        );
        assert_eq!(
            person_ophelia.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::Limited)
        );
        assert_eq!(
            person_madman.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::None)
        );
    }

    // 测试更新自然人的精神状态并重新评估行为能力
    #[test]
    fn test_update_mental_status() {
        let birth_date = get_test_date();
        let mut person = NaturalPerson::new(birth_date, MentalStatus::Normal);
        person
            .update_mental_status(MentalStatus::SeverelyImpaired)
            .unwrap();

        assert_eq!(person.mental_status, MentalStatus::SeverelyImpaired);
        assert_eq!(
            person.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::None)
        );
    }

    // 测试判断自然人是否可以作为监护人
    #[test]
    fn test_can_be_guardian() {
        let birth_date = get_test_date();
        let person_baby = NaturalPerson::new(birth_date, MentalStatus::Normal);

        let birth_date = Utc.with_ymd_and_hms(2014, 03, 12, 0, 0, 0).unwrap();
        let person_teenage = NaturalPerson::new(birth_date, MentalStatus::Normal);

        let birth_date = Utc.with_ymd_and_hms(2004, 03, 12, 0, 0, 0).unwrap();
        let person_adult = NaturalPerson::new(birth_date, MentalStatus::Normal);

        assert!(!person_baby.can_be_guardian());
        assert!(!person_teenage.can_be_guardian());
        assert!(person_adult.can_be_guardian());
    }

    // 测试设置监护关系
    #[test]
    fn test_set_guardian() {
        let birth_date = get_test_date();
        let mut person = NaturalPerson::new(birth_date, MentalStatus::SeverelyImpaired);

        let birth_date = Utc.with_ymd_and_hms(2003, 12, 25, 0, 0, 0).unwrap();
        let mut guardian = NaturalPerson::new(birth_date, MentalStatus::Normal);
        let scope = get_test_guardianship_scope();

        person.set_guardian(&mut guardian, scope.clone()).unwrap();

        assert!(person.guardian.is_some());
        assert_eq!(person.guardian.clone().unwrap().guardian, guardian.base.id);
        assert_eq!(
            person.guardian.clone().unwrap().scope.permitted_actions,
            scope.permitted_actions
        );
    }

    // 测试不能设置无资格的监护人
    #[test]
    fn test_set_invalid_guardian() {
        let birth_date = get_test_date();
        let mut person = NaturalPerson::new(birth_date, MentalStatus::Normal);
        let mut invalid_guardian = NaturalPerson::new(birth_date, MentalStatus::SeverelyImpaired); // 不符合监护人条件
        let scope = get_test_guardianship_scope();

        let result = person.set_guardian(&mut invalid_guardian, scope);

        assert!(result.is_err());
    }

    // 测试线程安全的自然人创建
    #[test]
    fn test_sync_natural_person_creation() {
        let birth_date = get_test_date();
        let mental_status = MentalStatus::Normal;
        let sync_person = SyncNaturalPerson::new(birth_date, mental_status);

        assert_eq!(sync_person.age(), 5);
        assert_eq!(sync_person.age(), 5);
    }

    // 测试更新线程安全的自然人精神状态
    #[test]
    fn test_sync_update_mental_status() {
        let birth_date = get_test_date();
        let sync_person = SyncNaturalPerson::new(birth_date, MentalStatus::Normal);

        sync_person
            .update_mental_status(MentalStatus::PartiallyImpaired)
            .unwrap();
        assert_eq!(
            *sync_person.mental_status.lock().unwrap(),
            MentalStatus::PartiallyImpaired
        );
    }

    // 测试线程安全的监护人设置
    // FIXME: 这个测试偶尔会出错，不知道为什么，大概率是和锁有关（废话）。到时候请 zsy 大佬看看问题所在
    #[test]
    fn test_sync_set_guardian() {
        let mut birth_date = get_test_date();
        let person = SyncNaturalPerson::new(birth_date, MentalStatus::Normal);

        birth_date = Utc.with_ymd_and_hms(2003, 12, 25, 0, 0, 0).unwrap();
        let guardian = SyncNaturalPerson::new(birth_date, MentalStatus::Normal);
        let scope = get_test_guardianship_scope();

        // println!("{:#?}", guardian);

        // let birth_date = Utc.with_ymd_and_hms(2003, 12, 25, 0, 0, 0).unwrap();
        // let guardian2 = SyncNaturalPerson::new(birth_date, MentalStatus::Normal);
        // let scope = get_test_guardianship_scope();
        // println!("{:#?}", guardian2);

        // assert_eq!(
        //     guardian.can_be_guardian().unwrap(),
        //     guardian2.can_be_guardian().unwrap()
        // );

        SyncNaturalPerson::set_guardian(
            &Arc::new(Mutex::new(person.clone())),
            &Arc::new(Mutex::new(guardian)),
            scope.clone(),
        )
        .unwrap();

        let ward_guard = person.guardian.read().unwrap();
        assert!(ward_guard.is_some());
        assert_eq!(
            ward_guard.as_ref().unwrap().scope.permitted_actions,
            scope.permitted_actions
        );
    }

    // 测试从非线程安全版本转换为线程安全版本
    #[test]
    fn test_from_natural_person() {
        let birth_date = get_test_date();
        let mental_status = MentalStatus::Normal;
        let person = NaturalPerson::new(birth_date, mental_status);
        let sync_person = SyncNaturalPerson::from_natural_person(person);

        assert_eq!(sync_person.age(), 5);
    }
}
