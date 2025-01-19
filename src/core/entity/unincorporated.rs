use crate::core::entity::base::{
    AuthorityScope, AuthorityStatus, BaseEntity, CapacityStatus, Entity, EntityType,
};
use crate::FanResult;
use crate::{FanError, ValidationErrorType};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// 非法人组织类型
#[derive(Debug, Clone, PartialEq)]
pub enum UnincorporatedOrgType {
    Partnership(PartnershipType), // 合伙企业
    IndividualBusiness,           // 个人独资企业
    Branch,                       // 分支机构
    ResidentCommittee,            // 居民委员会
    VillageCommittee,             // 村民委员会
    Other,                        // 其他
}

/// 合伙企业类型
#[derive(Debug, Clone, PartialEq)]
pub enum PartnershipType {
    General, // 普通合伙
    Limited, // 有限合伙
    Special, // 特殊普通合伙
}

/// 合伙人信息
#[derive(Debug, Clone)]
pub struct Partner {
    id: Uuid,                      // 合伙人ID
    partnership_type: PartnerType, // 合伙人类型
    contribution: f64,             // 出资额
    profit_sharing_ratio: f32,     // 利润分配比例
    liability_type: LiabilityType, // 责任承担方式
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartnerType {
    GeneralPartner, // 普通合伙人
    LimitedPartner, // 有限合伙人
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiabilityType {
    Unlimited, // 无限责任
    Limited,   // 有限责任
}

/// 非法人组织
#[derive(Debug, Clone)]
pub struct UnincorporatedOrg {
    base: BaseEntity,
    org_type: UnincorporatedOrgType,
    executive_partner: Option<Uuid>, // 执行事务合伙人（合伙企业特有）
    proprietor: Option<Uuid>,        // 投资人（个人独资企业特有）
    members: Vec<Partner>,           // 成员列表
    registered_address: String,
    establishment_date: DateTime<Utc>,
}

impl UnincorporatedOrg {
    pub fn new(
        org_type: UnincorporatedOrgType,
        registered_address: String,
        establishment_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        let authority_scope = AuthorityScope {
            status: AuthorityStatus::Full,
            permitted_authorities: HashSet::new(),
            restrictions: None,
        };

        Self {
            base: BaseEntity {
                id: Uuid::new_v4(),
                entity_type: EntityType::UnincorporatedOrg,
                capacity_status: CapacityStatus::UnincorporatedOrg(authority_scope),
                created_at: now,
                updated_at: now,
            },
            org_type,
            executive_partner: None,
            proprietor: None,
            members: Vec::new(),
            registered_address,
            establishment_date,
        }
    }

    /// 添加合伙人
    pub fn add_partner(&mut self, partner: Partner) -> FanResult<()> {
        match self.org_type {
            UnincorporatedOrgType::Partnership(_) => {
                self.members.push(partner);
                self.base.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(FanError::validation(
                "Only partnership can add partners",
                ValidationErrorType::EntityCapacityLacking,
                "add_partner",
                "UnincorporatedOrg",
            )),
        }
    }

    /// 设置执行事务合伙人
    pub fn set_executive_partner(&mut self, partner_id: Uuid) -> FanResult<()> {
        match self.org_type {
            UnincorporatedOrgType::Partnership(_) => {
                if self.members.iter().any(|p| p.id == partner_id) {
                    self.executive_partner = Some(partner_id);
                    self.base.updated_at = Utc::now();
                    Ok(())
                } else {
                    Err(FanError::validation(
                        "Partner not found",
                        ValidationErrorType::EntityError,
                        "set_executive_partner",
                        "UnincorporatedOrg",
                    ))
                }
            }
            _ => Err(FanError::validation(
                "Only partnership can set executive partner",
                ValidationErrorType::EntityCapacityLacking,
                "set_executive_partner",
                "UnincorporatedOrg",
            )),
        }
    }

    /// 添加职权范围
    pub fn add_authority(&mut self, authority: String) -> FanResult<()> {
        if let CapacityStatus::UnincorporatedOrg(scope) = &mut self.base.capacity_status {
            scope.permitted_authorities.insert(authority);
            self.base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::validation(
                "Invalid capacity status type",
                ValidationErrorType::EntityCapacityLacking,
                "update_authority_status",
                "UnincorporatedOrg",
            ))
        }
    }

    /// 更新职权状态
    pub fn update_authority_status(&mut self, new_status: AuthorityStatus) -> FanResult<()> {
        if let CapacityStatus::UnincorporatedOrg(scope) = &mut self.base.capacity_status {
            scope.status = new_status;
            self.base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::validation(
                "Invalid capacity status type",
                ValidationErrorType::EntityCapacityLacking,
                "update_authority_status",
                "UnincorporatedOrg",
            ))
        }
    }

    /// 检查是否可以进行特定活动
    pub fn can_perform_activity(&self, activity: &str) -> bool {
        if let CapacityStatus::UnincorporatedOrg(scope) = &self.base.capacity_status {
            match scope.status {
                AuthorityStatus::Full => {
                    scope.permitted_authorities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                AuthorityStatus::Limited => {
                    scope.permitted_authorities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                AuthorityStatus::Suspended => false,
            }
        } else {
            false
        }
    }
}

impl Entity for UnincorporatedOrg {
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
        match self.capacity_status() {
            CapacityStatus::UnincorporatedOrg(_) => true,
            _ => false,
        }
    }
}

/// 线程安全版本非法人组织
#[derive(Debug)]
pub struct SyncUnincorporatedOrg {
    base: Arc<RwLock<BaseEntity>>,
    org_type: UnincorporatedOrgType, // 不可变
    executive_partner: Arc<RwLock<Option<Uuid>>>,
    members: Arc<RwLock<Vec<Partner>>>,
    registered_address: Arc<RwLock<String>>,
    establishment_date: DateTime<Utc>, // 不可变
}

impl SyncUnincorporatedOrg {
    pub fn new(
        org_type: UnincorporatedOrgType,
        registered_address: String,
        establishment_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        let authority_scope = AuthorityScope {
            status: AuthorityStatus::Full,
            permitted_authorities: HashSet::new(),
            restrictions: None,
        };

        Self {
            base: Arc::new(RwLock::new(BaseEntity {
                id: Uuid::new_v4(),
                entity_type: EntityType::UnincorporatedOrg,
                capacity_status: CapacityStatus::UnincorporatedOrg(authority_scope),
                created_at: now,
                updated_at: now,
            })),
            org_type,
            executive_partner: Arc::new(RwLock::new(None)),
            members: Arc::new(RwLock::new(Vec::new())),
            registered_address: Arc::new(RwLock::new(registered_address)),
            establishment_date,
        }
    }

    pub fn add_partner(&self, partner: Partner) -> FanResult<()> {
        match self.org_type {
            UnincorporatedOrgType::Partnership(_) => {
                self.members.write().push(partner);
                self.base.write().updated_at = Utc::now();
                Ok(())
            }
            _ => Err(FanError::validation(
                "Only partnership can add partners",
                ValidationErrorType::EntityStatusIllegal,
                "add_partner",
                "SyncUnincorporatedOrg",
            )),
        }
    }

    pub fn set_executive_partner(&self, partner_id: Uuid) -> FanResult<()> {
        match self.org_type {
            UnincorporatedOrgType::Partnership(_) => {
                let members = self.members.read();
                if members.iter().any(|p| p.id == partner_id) {
                    drop(members); // 释放读锁
                    *self.executive_partner.write() = Some(partner_id);
                    self.base.write().updated_at = Utc::now();
                    Ok(())
                } else {
                    Err(FanError::validation(
                        "Partner not found",
                        ValidationErrorType::EntityError,
                        "set_executive_partner",
                        "SyncUnincorporatedOrg",
                    ))
                }
            }
            _ => Err(FanError::validation(
                "Only partnership can set executive partner",
                ValidationErrorType::EntityCapacityLacking,
                "set_executive_partner",
                "SyncUnincorporatedOrg",
            )),
        }
    }

    pub fn add_authority(&self, authority: String) -> FanResult<()> {
        let mut base = self.base.write();
        if let CapacityStatus::UnincorporatedOrg(scope) = &mut base.capacity_status {
            scope.permitted_authorities.insert(authority);
            base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::validation(
                "Invalid capacity status type",
                ValidationErrorType::EntityStatusIllegal,
                "add_authority",
                "SyncUnincorporatedOrg",
            ))
        }
    }

    pub fn update_authority_status(&self, new_status: AuthorityStatus) -> FanResult<()> {
        let mut base = self.base.write();
        if let CapacityStatus::UnincorporatedOrg(scope) = &mut base.capacity_status {
            scope.status = new_status;
            base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::validation(
                "Invalid capacity status type",
                ValidationErrorType::EntityStatusIllegal,
                "update_authority_status",
                "SyncUnincorporatedOrg",
            ))
        }
    }

    pub fn can_perform_activity(&self, activity: &str) -> bool {
        let base = self.base.read();
        if let CapacityStatus::UnincorporatedOrg(scope) = &base.capacity_status {
            match scope.status {
                AuthorityStatus::Full => {
                    scope.permitted_authorities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                AuthorityStatus::Limited => {
                    scope.permitted_authorities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                AuthorityStatus::Suspended => false,
            }
        } else {
            false
        }
    }

    pub fn from_unincorporated_org(org: UnincorporatedOrg) -> Self {
        Self {
            base: Arc::new(RwLock::new(org.base)),
            org_type: org.org_type,
            executive_partner: Arc::new(RwLock::new(org.executive_partner)),
            members: Arc::new(RwLock::new(org.members)),
            registered_address: Arc::new(RwLock::new(org.registered_address)),
            establishment_date: org.establishment_date,
        }
    }
}

impl Entity for SyncUnincorporatedOrg {
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
            CapacityStatus::UnincorporatedOrg(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partnership_creation() {
        let mut partnership = UnincorporatedOrg::new(
            UnincorporatedOrgType::Partnership(PartnershipType::General),
            "北京市海淀区xxx街道".to_string(),
            Utc::now(),
        );

        let partner = Partner {
            id: Uuid::new_v4(),
            partnership_type: PartnerType::GeneralPartner,
            contribution: 100000.0,
            profit_sharing_ratio: 0.5,
            liability_type: LiabilityType::Unlimited,
        };

        assert!(partnership.add_partner(partner.clone()).is_ok());
        assert!(partnership.set_executive_partner(partner.id).is_ok());
    }

    #[test]
    fn test_authority_management() {
        let mut org = UnincorporatedOrg::new(
            UnincorporatedOrgType::Partnership(PartnershipType::General),
            "北京市海淀区xxx街道".to_string(),
            Utc::now(),
        );

        org.add_authority("业务经营".to_string()).unwrap();
        assert!(org.can_perform_activity("业务经营"));

        org.update_authority_status(AuthorityStatus::Suspended)
            .unwrap();
        assert!(!org.can_perform_activity("业务经营"));
    }
}
