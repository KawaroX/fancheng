// src/core/entity.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::FanResult;

/// 民事主体的类型
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    NaturalPerson,         // 自然人
    LegalPerson,          // 法人
    UnincorporatedOrg,    // 非法人组织
}

/// 民事行为能力状态
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityStatus {
    Full,            // 完全民事行为能力
    Limited,         // 限制民事行为能力
    None,            // 无民事行为能力
}

/// 民事主体的基本特征
pub trait Entity {
    fn id(&self) -> Uuid;
    fn entity_type(&self) -> EntityType;
    fn capacity_status(&self) -> CapacityStatus;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// 基础主体信息
#[derive(Debug, Clone)]
struct BaseEntity {
    id: Uuid,
    entity_type: EntityType,
    capacity_status: CapacityStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// 自然人
#[derive(Debug, Clone)]
pub struct NaturalPerson {
    base: BaseEntity,
    age: u8,
    // 后续可以添加更多自然人特有的属性
}

/// 法人
#[derive(Debug, Clone)]
pub struct LegalPerson {
    base: BaseEntity,
    registered_capital: f64,
    // 后续可以添加更多法人特有的属性
}

/// 非法人组织
#[derive(Debug, Clone)]
pub struct UnincorporatedOrg {
    base: BaseEntity,
    org_type: UnincorporatedOrgType,
    members: Vec<Uuid>,
    // 后续可以添加更多非法人组织特有的属性
}

/// 非法人组织的类型
#[derive(Debug, Clone, PartialEq)]
pub enum UnincorporatedOrgType {
    Partnership,           // 合伙企业
    IndividualBusiness,   // 个人独资企业
    SocialOrg,            // 社会组织
    // ... 可以添加更多类型
}

// 为三种主体类型实现 Entity trait
impl Entity for NaturalPerson {
    fn id(&self) -> Uuid { self.base.id }
    fn entity_type(&self) -> EntityType { self.base.entity_type.clone() }
    fn capacity_status(&self) -> CapacityStatus { self.base.capacity_status.clone() }
    fn created_at(&self) -> DateTime<Utc> { self.base.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.base.updated_at }
}

// 类似地实现 LegalPerson 和 UnincorporatedOrg 的 Entity trait...
impl Entity for LegalPerson {
    fn id(&self) -> Uuid { self.base.id }
    fn entity_type(&self) -> EntityType { self.base.entity_type.clone() }
    fn capacity_status(&self) -> CapacityStatus { self.base.capacity_status.clone() }
    fn created_at(&self) -> DateTime<Utc> { self.base.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.base.updated_at }
}

impl Entity for UnincorporatedOrg {
    fn id(&self) -> Uuid { self.base.id }
    fn entity_type(&self) -> EntityType { self.base.entity_type.clone() }
    fn capacity_status(&self) -> CapacityStatus { self.base.capacity_status.clone() }
    fn created_at(&self) -> DateTime<Utc> { self.base.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.base.updated_at }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_natural_person() {
        let person = NaturalPerson::new(25);
        assert_eq!(person.entity_type(), EntityType::NaturalPerson);
        assert_eq!(person.age, 25);
    }
}