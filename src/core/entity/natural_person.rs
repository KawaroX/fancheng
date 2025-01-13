use crate::FanError;
use crate::FanResult;

use crate::core::entity::base::{BaseEntity, CapacityStatus, Entity, EntityType, NaturalCapacity};
use chrono::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
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
    pub fn set_guardian(
        ward: &Arc<Mutex<Self>>,
        guardian: &Arc<Mutex<Self>>,
        scope: GuardianshipScope,
    ) -> FanResult<()> {
        // 尝试获取监护人信息，仅持有读取所需的短时间锁
        let guardian_id = {
            let guardian = guardian.try_lock().map_err(|e| match e {
                TryLockError::Poisoned(_) => FanError::LockError("Guardian lock poisoned".into()),
                TryLockError::WouldBlock => {
                    FanError::LockError("Guardian lock currently in use".into())
                }
            })?;
            if !guardian.can_be_guardian() {
                return Err(FanError::ValidationError("Invalid guardian".to_string()));
            }
            guardian.base.id
        };

        // 对 ward 和 guardian 按固定顺序加锁，避免死锁
        let (mut ward, mut guardian) = if Arc::as_ptr(ward) < Arc::as_ptr(guardian) {
            let ward = ward.try_lock().map_err(|e| match e {
                TryLockError::Poisoned(_) => FanError::LockError("Ward lock poisoned".into()),
                TryLockError::WouldBlock => {
                    FanError::LockError("Ward lock currently in use".into())
                }
            })?;
            let guardian = guardian.try_lock().map_err(|e| match e {
                TryLockError::Poisoned(_) => FanError::LockError("Guardian lock poisoned".into()),
                TryLockError::WouldBlock => {
                    FanError::LockError("Guardian lock currently in use".into())
                }
            })?;
            (ward, guardian)
        } else {
            let guardian = guardian.try_lock().map_err(|e| match e {
                TryLockError::Poisoned(_) => FanError::LockError("Guardian lock poisoned".into()),
                TryLockError::WouldBlock => {
                    FanError::LockError("Guardian lock currently in use".into())
                }
            })?;
            let ward = ward.try_lock().map_err(|e| match e {
                TryLockError::Poisoned(_) => FanError::LockError("Ward lock poisoned".into()),
                TryLockError::WouldBlock => {
                    FanError::LockError("Ward lock currently in use".into())
                }
            })?;
            (ward, guardian)
        };

        // 更新 ward 和 guardian 的状态
        ward.guardian = Some(Guardianship {
            guardian: guardian_id,
            ward: ward.base.id,
            scope,
            created_at: Utc::now(),
            valid_until: None,
        });
        ward.base.updated_at = Utc::now();

        guardian.is_guardian = true;
        guardian.base.updated_at = Utc::now();

        Ok(())
    }

    /// 判断是否可以作为监护人
    pub fn can_be_guardian(&self) -> bool {
        matches!(
            &self.base.capacity_status,
            CapacityStatus::NaturalPerson(NaturalCapacity::Full)
        ) && self.mental_status == MentalStatus::Normal
            && self.age() >= 18
    }
}

// 实现 Entity trait
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_natural_person_capacity() {
        let birth_date = Utc::now() - chrono::Duration::days(365 * 10); // 10岁
        let person = NaturalPerson::new(birth_date, MentalStatus::Normal);

        match person.capacity_status() {
            CapacityStatus::NaturalPerson(capacity) => {
                assert_eq!(capacity, NaturalCapacity::Limited);
            }
            _ => panic!("Expected NaturalPerson capacity status"),
        }
    }

    #[test]
    fn test_guardianship() {
        let guardian_birth_date = Utc::now() - chrono::Duration::days(365 * 30); // 30岁
        let guardian = Arc::new(Mutex::new(NaturalPerson::new(
            guardian_birth_date,
            MentalStatus::Normal,
        )));
        assert!(guardian.lock().unwrap().can_be_guardian());

        let ward_birth_date = Utc::now() - chrono::Duration::days(365 * 10); // 10岁
        let ward = Arc::new(Mutex::new(NaturalPerson::new(
            ward_birth_date,
            MentalStatus::Normal,
        )));

        let mut scope = GuardianshipScope {
            permitted_actions: HashSet::new(),
        };
        scope
            .permitted_actions
            .insert("daily_necessities".to_string());

        // 使用线程安全的 set_guardian
        let result = NaturalPerson::set_guardian(&ward, &guardian, scope);

        assert!(
            result.is_ok(),
            "Setting guardian should succeed, but got: {:?}",
            result
        );

        // 验证被监护人状态
        let ward_guardian = ward.lock().unwrap().guardian.clone();
        assert!(ward_guardian.is_some(), "Ward should have a guardian");
        assert_eq!(
            ward_guardian.unwrap().guardian,
            guardian.lock().unwrap().id(),
            "Guardian ID should match"
        );

        // 验证监护人状态
        assert!(
            guardian.lock().unwrap().is_guardian,
            "Guardian should be marked as guardian"
        );
    }

    #[test]
    fn test_set_guardian_success() {
        // 初始化监护人和被监护人
        let guardian = Arc::new(Mutex::new(NaturalPerson::new(
            Utc::now() - chrono::Duration::days(365 * 30), // 30岁
            MentalStatus::Normal,
        )));
        let ward = Arc::new(Mutex::new(NaturalPerson::new(
            Utc::now() - chrono::Duration::days(365 * 10), // 10岁
            MentalStatus::Normal,
        )));

        // 设置监护范围
        let mut scope = GuardianshipScope {
            permitted_actions: HashSet::new(),
        };
        scope
            .permitted_actions
            .insert("daily_necessities".to_string());

        // 调用 set_guardian
        let result = NaturalPerson::set_guardian(&ward, &guardian, scope);
        assert!(result.is_ok(), "Setting guardian should succeed");

        // 检查被监护人信息是否正确更新
        let ward_guardian = ward.lock().unwrap().guardian.clone();
        assert!(ward_guardian.is_some(), "Ward should have a guardian");
        assert_eq!(
            ward_guardian.unwrap().guardian,
            guardian.lock().unwrap().id(),
            "Guardian ID should match"
        );

        // 检查监护人状态是否正确更新
        assert!(
            guardian.lock().unwrap().is_guardian,
            "Guardian should be marked as guardian"
        );
    }

    #[test]
    fn test_set_guardian_invalid_guardian() {
        // 初始化无效的监护人（未成年人）
        let guardian = Arc::new(Mutex::new(NaturalPerson::new(
            Utc::now() - chrono::Duration::days(365 * 15), // 15岁
            MentalStatus::Normal,
        )));
        let ward = Arc::new(Mutex::new(NaturalPerson::new(
            Utc::now() - chrono::Duration::days(365 * 10), // 10岁
            MentalStatus::Normal,
        )));

        // 设置监护范围
        let scope = GuardianshipScope {
            permitted_actions: HashSet::new(),
        };

        // 调用 set_guardian
        let result = NaturalPerson::set_guardian(&ward, &guardian, scope);
        assert!(
            matches!(result, Err(FanError::ValidationError(_))),
            "Setting guardian should fail for invalid guardian"
        );
    }

    #[test]
    fn test_set_guardian_lock_error() {
        // 初始化监护人和被监护人
        let guardian = Arc::new(Mutex::new(NaturalPerson::new(
            Utc::now() - chrono::Duration::days(365 * 30), // 30岁
            MentalStatus::Normal,
        )));
        let ward = Arc::new(Mutex::new(NaturalPerson::new(
            Utc::now() - chrono::Duration::days(365 * 10), // 10岁
            MentalStatus::Normal,
        )));

        // 模拟锁冲突
        let guardian_clone = Arc::clone(&guardian);
        let ward_clone = Arc::clone(&ward);

        let _lock_guardian = guardian.lock().unwrap(); // 模拟另一个线程锁住监护人
        let _lock_ward = ward.lock().unwrap(); // 模拟另一个线程锁住被监护人

        // 在新线程中调用 set_guardian，模拟锁冲突
        let handle = thread::spawn(move || {
            let scope = GuardianshipScope {
                permitted_actions: HashSet::new(),
            };

            NaturalPerson::set_guardian(&ward_clone, &guardian_clone, scope)
        });

        let result = handle.join().unwrap();
        assert!(
            matches!(result, Err(FanError::LockError(_))),
            "Setting guardian should fail due to lock error"
        );
    }
}
