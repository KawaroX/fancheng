use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::FanResult;

/// 法律主体的类型
/// 目前我们主要关注自然人和法人这两种最基本的类型，但是还是设置一个非法人组织的枚举
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    NaturalPerson,         // 自然人
    LegalPerson,          // 法人
    UnincorporatedOrg,    // 非法人组织
}

/// 民事行为能力的状态
/// 这直接关系到法律主体可以进行的行为范围
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityStatus {
    Full,            // 完全民事行为能力
    Limited,         // 限制民事行为能力
    None,            // 无民事行为能力
}

/// 法律主体的基本结构
/// 包含了识别和表征一个法律主体所需的基本信息
#[derive(Debug, Clone)]
pub struct LegalEntity {
    /// 唯一标识符
    id: Uuid,
    /// 主体类型（自然人或法人）
    entity_type: EntityType,
    /// 民事行为能力状态
    capacity_status: CapacityStatus,
    /// 创建时间
    created_at: DateTime<Utc>,
    /// 最后更新时间
    updated_at: DateTime<Utc>,
}

impl LegalEntity {
    /// 创建一个新的法律主体
    pub fn new(entity_type: EntityType, capacity_status: CapacityStatus) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entity_type,
            capacity_status,
            created_at: now,
            updated_at: now,
        }
    }

    /// 获取主体的行为能力状态
    pub fn capacity_status(&self) -> &CapacityStatus {
        &self.capacity_status
    }

    /// 更新行为能力状态
    pub fn update_capacity_status(&mut self, new_status: CapacityStatus) -> FanResult<()> {
        self.capacity_status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_legal_entity() {
        let entity = LegalEntity::new(
            EntityType::NaturalPerson,
            CapacityStatus::Full,
        );
        
        assert_eq!(entity.entity_type, EntityType::NaturalPerson);
        assert_eq!(entity.capacity_status, CapacityStatus::Full);
    }

    #[test]
    fn test_update_capacity_status() {
        let mut entity = LegalEntity::new(
            EntityType::NaturalPerson,
            CapacityStatus::Full,
        );
        
        entity.update_capacity_status(CapacityStatus::Limited).unwrap();
        assert_eq!(entity.capacity_status, CapacityStatus::Limited);
    }
}