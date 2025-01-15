use crate::FanError;
use crate::FanResult;

use crate::core::entity::base::{
    BaseEntity, BusinessScope, BusinessStatus, CapacityStatus, Entity, EntityType,
};
use chrono::{DateTime, Utc};
use parking_lot::{Mutex, RwLock};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// 法人类型
#[derive(Debug, Clone, PartialEq)]
pub enum LegalPersonType {
    Company(CompanyType), // 公司
    Institution,          // 事业单位
    SocialOrganization,   // 社会团体
    Foundation,           // 基金会
}

/// 公司类型
#[derive(Debug, Clone, PartialEq)]
pub enum CompanyType {
    Limited,         // 有限责任公司
    JointStock,      // 股份有限公司
    ForeignInvested, // 外商投资企业
    StateOwned,      // 国有企业
}

/// 法人
#[derive(Debug, Clone)]
pub struct LegalPerson {
    base: BaseEntity,
    legal_person_type: LegalPersonType,
    registered_capital: f64,
    legal_representative: Uuid, // 法定代表人ID
    registered_address: String,
    establishment_date: DateTime<Utc>,
}

impl LegalPerson {
    pub fn new(
        legal_person_type: LegalPersonType,
        registered_capital: f64,
        legal_representative: Uuid,
        registered_address: String,
        establishment_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        // 创建默认的经营范围
        let business_scope = BusinessScope {
            status: BusinessStatus::Normal,
            permitted_activities: HashSet::new(),
            restrictions: None,
        };

        Self {
            base: BaseEntity {
                id: Uuid::new_v4(),
                entity_type: EntityType::LegalPerson,
                capacity_status: CapacityStatus::LegalPerson(business_scope),
                created_at: now,
                updated_at: now,
            },
            legal_person_type,
            registered_capital,
            legal_representative,
            registered_address,
            establishment_date,
        }
    }

    /// 添加经营范围
    pub fn add_permitted_activity(&mut self, activity: String) -> FanResult<()> {
        if let CapacityStatus::LegalPerson(scope) = &mut self.base.capacity_status {
            scope.permitted_activities.insert(activity);
            self.base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::ValidationError(
                "Invalid capacity status type".to_string(),
            ))
        }
    }

    /// 添加经营限制
    pub fn add_restriction(&mut self, restriction: String) -> FanResult<()> {
        if let CapacityStatus::LegalPerson(scope) = &mut self.base.capacity_status {
            if scope.restrictions.is_none() {
                scope.restrictions = Some(Vec::new());
            }
            if let Some(restrictions) = &mut scope.restrictions {
                restrictions.push(restriction);
            }
            self.base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::ValidationError(
                "Invalid capacity status type".to_string(),
            ))
        }
    }

    /// 更新经营状态
    pub fn update_business_status(&mut self, new_status: BusinessStatus) -> FanResult<()> {
        if let CapacityStatus::LegalPerson(scope) = &mut self.base.capacity_status {
            scope.status = new_status;
            self.base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::ValidationError(
                "Invalid capacity status type".to_string(),
            ))
        }
    }

    /// 检查是否可以进行特定活动
    pub fn can_perform_activity(&self, activity: &str) -> bool {
        if let CapacityStatus::LegalPerson(scope) = &self.base.capacity_status {
            match scope.status {
                BusinessStatus::Normal => {
                    scope.permitted_activities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                BusinessStatus::Restricted => {
                    scope.permitted_activities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                BusinessStatus::Suspended => false,
            }
        } else {
            false
        }
    }

    /// 更改法定代表人
    pub fn change_legal_representative(&mut self, new_representative: Uuid) -> FanResult<()> {
        self.legal_representative = new_representative;
        self.base.updated_at = Utc::now();
        Ok(())
    }
}

impl Entity for LegalPerson {
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

/// 线程安全版本法人
#[derive(Debug, Clone)]
pub struct SyncLegalPerson {
    base: Arc<RwLock<BaseEntity>>,
    legal_person_type: LegalPersonType,      // 不可变，不需要锁
    registered_capital: Arc<RwLock<f64>>,    // 注册资本可能变更
    legal_representative: Arc<RwLock<Uuid>>, // 法定代表人可能变更
    registered_address: Arc<RwLock<String>>, // 注册地址可能变更
    establishment_date: DateTime<Utc>,       // 不可变，不需要锁
}

impl SyncLegalPerson {
    pub fn new(
        legal_person_type: LegalPersonType,
        registered_capital: f64,
        legal_representative: Uuid,
        registered_address: String,
        establishment_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        let business_scope = BusinessScope {
            status: BusinessStatus::Normal,
            permitted_activities: HashSet::new(),
            restrictions: None,
        };

        Self {
            base: Arc::new(RwLock::new(BaseEntity {
                id: Uuid::new_v4(),
                entity_type: EntityType::LegalPerson,
                capacity_status: CapacityStatus::LegalPerson(business_scope),
                created_at: now,
                updated_at: now,
            })),
            legal_person_type,
            registered_capital: Arc::new(RwLock::new(registered_capital)),
            legal_representative: Arc::new(RwLock::new(legal_representative)),
            registered_address: Arc::new(RwLock::new(registered_address)),
            establishment_date,
        }
    }

    pub fn update_registered_capital(&self, new_capital: f64) -> FanResult<()> {
        if new_capital <= 0.0 {
            return Err(FanError::ValidationError(
                "Invalid capital amount".to_string(),
            ));
        }

        *self.registered_capital.write() = new_capital;
        self.base.write().updated_at = Utc::now();
        Ok(())
    }

    pub fn change_legal_representative(&self, new_representative: Uuid) -> FanResult<()> {
        *self.legal_representative.write() = new_representative;
        self.base.write().updated_at = Utc::now();
        Ok(())
    }

    pub fn add_permitted_activity(&self, activity: String) -> FanResult<()> {
        let mut base = self.base.write();
        if let CapacityStatus::LegalPerson(scope) = &mut base.capacity_status {
            scope.permitted_activities.insert(activity);
            base.updated_at = Utc::now();
            Ok(())
        } else {
            Err(FanError::ValidationError(
                "Invalid capacity status type".to_string(),
            ))
        }
    }

    pub fn can_perform_activity(&self, activity: &str) -> bool {
        let base = self.base.read();
        if let CapacityStatus::LegalPerson(scope) = &base.capacity_status {
            match scope.status {
                BusinessStatus::Normal => {
                    scope.permitted_activities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                BusinessStatus::Restricted => {
                    scope.permitted_activities.contains(activity)
                        && !scope
                            .restrictions
                            .as_ref()
                            .map_or(false, |r| r.contains(&activity.to_string()))
                }
                BusinessStatus::Suspended => false,
            }
        } else {
            false
        }
    }

    pub fn from_legal_person(person: LegalPerson) -> Self {
        Self {
            base: Arc::new(RwLock::new(person.base)),
            legal_person_type: person.legal_person_type,
            registered_capital: Arc::new(RwLock::new(person.registered_capital)),
            legal_representative: Arc::new(RwLock::new(person.legal_representative)),
            registered_address: Arc::new(RwLock::new(person.registered_address)),
            establishment_date: person.establishment_date,
        }
    }
}

impl Entity for SyncLegalPerson {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legal_person_activities() {
        let mut company = LegalPerson::new(
            LegalPersonType::Company(CompanyType::Limited),
            1_000_000.0,
            Uuid::new_v4(),
            "北京市朝阳区xxx街道".to_string(),
            Utc::now(),
        );

        // 添加经营范围
        company
            .add_permitted_activity("软件开发".to_string())
            .unwrap();
        assert!(company.can_perform_activity("软件开发"));
        assert!(!company.can_perform_activity("房地产开发"));

        // 添加限制
        company.add_restriction("软件开发".to_string()).unwrap();
        assert!(!company.can_perform_activity("软件开发"));
    }

    #[test]
    fn test_business_status_change() {
        let mut company = LegalPerson::new(
            LegalPersonType::Company(CompanyType::Limited),
            1_000_000.0,
            Uuid::new_v4(),
            "北京市朝阳区xxx街道".to_string(),
            Utc::now(),
        );

        company
            .add_permitted_activity("软件开发".to_string())
            .unwrap();
        assert!(company.can_perform_activity("软件开发"));

        // 暂停经营状态
        company
            .update_business_status(BusinessStatus::Suspended)
            .unwrap();
        assert!(!company.can_perform_activity("软件开发"));
    }
}
