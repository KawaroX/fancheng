use crate::FanError;
use crate::FanResult;

use crate::core::entity::base::{BaseEntity, CapacityStatus, Entity, EntityType, NaturalCapacity};
use chrono::prelude::*;
use parking_lot::{Mutex, RwLock};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// # 精神状态
/// - Normal - 正常
/// - PartiallyImpaired - 部分受损
/// - SeverelyImpaired - 严重受损
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
///
/// 该结构体表示一个自然人，包含了自然人的基本信息及其与监护人之间的关系。
/// 自然人可以是成年人或未成年人，可能有监护人，也可能没有。
///
/// 字段:
/// - `base`: 基本实体信息，如姓名、性别等。
/// - `birth_date`: 出生日期，用于计算年龄。
/// - `mental_status`: 精神状态，表示自然人的心理健康状况。
/// - `guardian`: 可选的监护人信息，如果自然人为未成年人或因精神状态需要监护，则该字段存在。
/// - `is_guardian`: 表示当前自然人是否为监护人的标志。
#[derive(Debug, Clone)]
pub struct NaturalPerson {
    base: BaseEntity,
    birth_date: DateTime<Utc>,
    mental_status: MentalStatus,
    guardian: Option<Guardianship>,
    is_guardian: bool,
}


impl NaturalPerson {
    /// 创建一个新的自然人实体
    ///
    /// # 参数 Arguments
    ///
    /// * `birth_date` - Utc时间格式的出生日期，用于确定自然人的年龄
    /// * `mental_status` - 心智状态，用于评估自然人的行为能力
    ///
    /// # 返回 Returns
    ///
    /// 返回一个新的自然人实体，包含基本属性和根据出生日期与心智状态评估的行为能力
    pub fn new(birth_date: DateTime<Utc>, mental_status: MentalStatus) -> Self {
        // 获取当前UTC时间，用于设置创建和更新时间戳
        let now = Utc::now();

        // 根据出生日期和心智状态评估行为能力
        let capacity = Self::evaluate_capacity(&birth_date, &mental_status);

        // 构建并返回一个新的自然人实体
        Self {
            base: BaseEntity {
                // 生成唯一的实体ID
                id: Uuid::new_v4(),
                // 设置实体类型为自然人
                entity_type: EntityType::NaturalPerson,
                // 设置行为能力状态
                capacity_status: CapacityStatus::NaturalPerson(capacity),
                // 设置创建时间戳
                created_at: now,
                // 设置更新时间戳
                updated_at: now,
            },
            // 设置出生日期
            birth_date,
            // 设置心智状态
            mental_status,
            // 初始化法定监护人为空
            guardian: None,
            // 初始化是否为监护人的状态为否
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

    fn has_capacity(&self) -> bool {
        match &self.base.capacity_status {
            CapacityStatus::NaturalPerson(capacity) => match capacity {
                NaturalCapacity::Full => true,
                _ => false,
            },
            _ => false,
        }
    }
}

/// 线程安全的 NaturalPerson
#[derive(Clone, Debug)]
pub struct SyncNaturalPerson {
    base: Arc<RwLock<BaseEntity>>,
    birth_date: DateTime<Utc>, // 不需要锁，因为不可变
    mental_status: Arc<RwLock<MentalStatus>>,
    guardian: Arc<RwLock<Option<Guardianship>>>,
    is_guardian: Arc<RwLock<bool>>,
}

/// 给 SyncNaturalPerson（线程安全的NP） 实现 Entity trait
impl SyncNaturalPerson {
    /// 创建一个新的自然人实体
    ///
    /// # 参数 Arguments
    ///
    /// * `birth_date` - 自然人的出生日期，用于计算其民事行为能力
    /// * `mental_status` - 自然人的精神状态，用于评估其民事行为能力
    ///
    /// # 返回 Returns
    ///
    /// 返回一个新创建的自然人实体实例
    pub fn new(birth_date: DateTime<Utc>, mental_status: MentalStatus) -> Self {
        // 获取当前时间，用于记录实体的创建和更新时间
        let now = Utc::now();

        // 根据出生日期和精神状态评估自然人的民事行为能力
        let capacity = NaturalPerson::evaluate_capacity(&birth_date, &mental_status);

        // 构建并返回一个新的自然人实体
        Self {
            // 使用Arc和RwLock来管理实体的基础信息，确保线程安全和可变性
            base: Arc::new(RwLock::new(BaseEntity {
                // 为每个自然人实体分配一个唯一的UUID作为标识
                id: Uuid::new_v4(),
                // 设置实体类型为自然人
                entity_type: EntityType::NaturalPerson,
                // 根据自然人的民事行为能力设置其民事行为能力状态
                capacity_status: CapacityStatus::NaturalPerson(capacity),
                // 记录实体的创建时间
                created_at: now,
                // 记录实体的最后更新时间
                updated_at: now,
            })),
            // 自然人的出生日期
            birth_date,
            // 自然人的精神状态，使用Arc和RwLock确保线程安全和可变性
            mental_status: Arc::new(RwLock::new(mental_status)),
            // 自然人的监护人信息，初始设置为None，使用Arc和RwLock确保线程安全和可变性
            guardian: Arc::new(RwLock::new(None)),
            // 自然人是否是监护人的状态，初始设置为false，使用Arc和RwLock确保线程安全和可变性
            is_guardian: Arc::new(RwLock::new(false)),
        }
    }

    pub fn age(&self) -> u8 {
        let now = Utc::now();
        let age = now.year() - self.birth_date.year();
        age as u8
    }

    pub fn update_mental_status(&self, new_status: MentalStatus) -> FanResult<()> {
        let mut status = self.mental_status.write();

        *status = new_status.clone();

        let mut base = self.base.write();

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
            let guardian_guard = guardian.lock();
            if !guardian_guard.can_be_guardian()? {
                return Err(FanError::ValidationError("Invalid guardian".to_string()));
            }

            let base = guardian_guard.base.read();

            base.id
        };

        // println!("{}", guardian_id);

        // 按地址顺序加锁避免死锁
        let (ward_guard, guardian_guard) = if Arc::as_ptr(ward) < Arc::as_ptr(guardian) {
            (ward.try_lock().unwrap(), guardian.lock())
        } else {
            (guardian.try_lock().unwrap(), ward.lock())
        };

        // 更新被监护人状态
        {
            let mut ward_guardian = ward_guard.guardian.write();
            let mut ward_base = ward_guard.base.write();

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
            let mut guardian_is_guardian = guardian_guard.is_guardian.write();
            let mut guardian_base = guardian_guard.base.write();

            *guardian_is_guardian = true;
            guardian_base.updated_at = Utc::now();
        }

        Ok(())
    }

    pub fn can_be_guardian(&self) -> FanResult<bool> {
        let status = self.mental_status.read();

        let base = self.base.read();

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
            mental_status: Arc::new(RwLock::new(person.mental_status)),
            guardian: Arc::new(RwLock::new(person.guardian)),
            is_guardian: Arc::new(RwLock::new(person.is_guardian)),
        }
    }
}

impl Entity for SyncNaturalPerson {
    fn id(&self) -> Uuid {
        self.base.read().id
    }

    fn entity_type(&self) -> EntityType {
        self.base.read().entity_type.clone()
    }

    fn capacity_status(&self) -> CapacityStatus {
        self.base.read().capacity_status.clone()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.read().created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.read().updated_at
    }

    fn has_capacity(&self) -> bool {
        match self.capacity_status() {
            CapacityStatus::NaturalPerson(NaturalCapacity::Full) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            *sync_person.mental_status.read(),
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

        let ward_guard = person.guardian.read();
        assert!(ward_guard.is_some());
        assert_eq!(
            ward_guard.as_ref().unwrap().scope.permitted_actions.clone(),
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
