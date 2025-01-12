use crate::FanError;
use crate::FanResult;

use crate::core::entity::base::{BaseEntity, CapacityStatus, Entity, EntityType, NaturalCapacity};
use chrono::prelude::*;
use std::collections::HashSet;
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

    /// 设置监护人
    pub fn set_guardian(
        &mut self,
        guardian: &NaturalPerson,
        scope: GuardianshipScope,
    ) -> FanResult<()> {
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
        let guardian = NaturalPerson::new(guardian_birth_date, MentalStatus::Normal);
        assert!(guardian.can_be_guardian());

        let ward_birth_date = Utc::now() - chrono::Duration::days(365 * 10); // 10岁
        let mut ward = NaturalPerson::new(ward_birth_date, MentalStatus::Normal);

        let mut scope = GuardianshipScope {
            permitted_actions: HashSet::new(),
        };
        scope
            .permitted_actions
            .insert("daily_necessities".to_string());

        assert!(ward.set_guardian(&guardian, scope).is_ok());
    }
}
