use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::fmt::Debug;
use uuid::Uuid;

/// 民事主体的类型
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    NaturalPerson,     // 自然人
    LegalPerson,       // 法人
    UnincorporatedOrg, // 非法人组织
}

/// 民事主体的基本特征
pub trait Entity {
    fn id(&self) -> Uuid;
    fn entity_type(&self) -> EntityType;
    fn capacity_status(&self) -> CapacityStatus;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn has_capacity(&self) -> bool;
}

impl Debug for dyn Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity {{ id: {}, entity_type: {:?}, capacity_status: {:?}, created_at: {}, updated_at: {} }}",
            self.id(),
            self.entity_type(),
            self.capacity_status(),
            self.created_at(),
            self.updated_at()
        )
    }
}

/// 基础主体信息
#[derive(Debug, Clone)]
pub struct BaseEntity {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub capacity_status: CapacityStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 民事行为能力状态
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityStatus {
    // 自然人的行为能力
    NaturalPerson(NaturalCapacity),
    // 法人的行为能力
    LegalPerson(BusinessScope),
    // 非法人组织的行为能力
    UnincorporatedOrg(AuthorityScope),
}

/// 自然人的行为能力状态
#[derive(Debug, Clone, PartialEq)]
pub enum NaturalCapacity {
    Full,    // 完全民事行为能力
    Limited, // 限制民事行为能力
    None,    // 无民事行为能力
}

/// 法人的经营范围
#[derive(Debug, Clone, PartialEq)]
pub struct BusinessScope {
    // 是否属于正常经营状态
    pub status: BusinessStatus,
    // 经营范围列表
    pub permitted_activities: HashSet<String>,
    // 特别限制（如果有）
    pub restrictions: Option<Vec<String>>,
}

/// 法人的经营状态
#[derive(Debug, Clone, PartialEq)]
pub enum BusinessStatus {
    Normal,     // 正常经营
    Restricted, // 受限经营
    Suspended,  // 经营被暂停
}

/// 非法人组织的职权范围
#[derive(Debug, Clone, PartialEq)]
pub struct AuthorityScope {
    // 职权状态
    pub status: AuthorityStatus,
    // 允许的职权范围
    pub permitted_authorities: HashSet<String>,
    // 特别限制（如果有）
    pub restrictions: Option<Vec<String>>,
}

/// 非法人组织的职权状态
#[derive(Debug, Clone, PartialEq)]
pub enum AuthorityStatus {
    Full,      // 完整职权
    Limited,   // 受限职权
    Suspended, // 职权被暂停
}

/// 默认 EntityType 为 NaturalPerson
impl Default for EntityType {
    fn default() -> Self {
        EntityType::NaturalPerson
    }
}

/// 默认 CapacityStatus 为 NaturalCapacity::None
impl Default for CapacityStatus {
    fn default() -> Self {
        CapacityStatus::NaturalPerson(NaturalCapacity::None)
    }
}
