//! 意思表示的核心定义
//! 包括意思表示的类型、结构和基本行为

use chrono::{DateTime, Utc};
use uuid::Uuid;
use super::content::IntentContent;
use crate::core::entity::Entity;
use std::collections::HashMap;
use std::sync::Arc;

/// # 意思表示的类型
/// - Offer：要约
/// - Acceptance：承诺
/// - CounterOffer：反要约
/// - Revocation：撤回
/// - Withdrawal：撤销
/// - OfferInvitation：要约邀请
#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationType {
    /// 要约：希望与他人订立合同的意思表示
    Offer,
    /// 承诺：接受要约的意思表示
    Acceptance,
    /// 反要约：对要约的实质性变更
    CounterOffer,
    /// 要约的撤回：要约到达前的撤销
    Revocation,
    /// 要约的撤销：要约到达后的撤销
    Withdrawal,
    /// 要约邀请：希望他人向自己发出要约的意思表示
    OfferInvitation,
}

/// 意思表示的状态
#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationStatus {
    Created,    // 意思表示创建但尚未生效
    Effective,  // 意思表示已经生效
    Revoked,    // 意思表示被撤回
    Withdrawn,  // 意思表示被撤销
}

/// 意思表示的核心结构
///
/// 本结构体用于定义和描述一个意思表示的基本信息，包括其唯一标识符、类型、表意人、相对人、具体内容、生成时间、有效期及当前状态。意思表示是民法上的概念，指表意人通过语言、文字或其他方式表达其内心意思的行为，是法律行为的基础。
/// 它包含以下字段：
/// - id：唯一标识符，使用UUID来保证全局范围内的唯一性。
/// - declaration_type：意思表示的类型，使用DeclarationType枚举来定义。
/// - declarant：意思表示的表意人，使用 trait object 来动态引用实体对象。
/// - recipient：相对人，使用Option<Arc<dyn Entity>>来动态引用实体对象，允许为空。
/// - content：具体的内容，使用IntentContent结构体来描述。
/// - created_at：生成时间，使用DateTime<Utc>来记录生成时间，采用UTC时间标准。
/// - valid_until：有效期，使用Option<DateTime<Utc>>来记录有效期，可能为空表示长期有效。
/// - status：当前状态，使用DeclarationStatus枚举来定义。
#[derive(Debug)]
pub struct IntentDeclaration {
    /// 唯一标识符，使用UUID来唯一标识一个意思表示，确保在全局范围内的唯一性。
    id: Uuid,
    /// 意思表示的类型，通过DeclarationType枚举来定义意思表示的类型，如要约、承诺等。
    declaration_type: DeclarationType,
    /// 意思表示的表意人（使用 trait object），使用Arc<dyn Entity>来动态引用表意人对象，允许在运行时与各种具体的实体类型互操作。
    declarant: Arc<dyn Entity>,
    /// 意思表示的相对人（可能为空，比如要约邀请），使用Option<Arc<dyn Entity>>来动态引用意思表示的相对人对象，允许为空以支持要约邀请等场景。
    recipient: Option<Arc<dyn Entity>>,
    /// 意思表示的具体内容，通过IntentContent结构体来详细描述意思表示的具体内容。
    content: IntentContent,
    /// 意思表示的生成时间，使用DateTime<Utc>来记录意思表示的生成时间，采用UTC时间标准。
    created_at: DateTime<Utc>,
    /// 意思表示的有效期，使用Option<DateTime<Utc>>来记录意思表示的有效截止时间，可能为空表示长期有效。
    valid_until: Option<DateTime<Utc>>,
    /// 意思表示的当前状态，通过DeclarationStatus枚举来定义意思表示的当前状态，如生效、失效等。
    status: DeclarationStatus,
}


impl IntentDeclaration {
    /// 创建新的意思表示
    pub fn new(
        declaration_type: DeclarationType,
        declarant: Arc<dyn Entity>,
        recipient: Option<Arc<dyn Entity>>,
        content: IntentContent,
        valid_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            declaration_type,
            declarant,
            recipient,
            content,
            created_at: Utc::now(),
            valid_until,
            status: DeclarationStatus::Created,
        }
    }

    /// 判断意思表示是否仍然有效
    pub fn is_valid(&self) -> bool {
        // 检查状态
        if self.status != DeclarationStatus::Effective {
            return false;
        }

        // 检查是否在有效期内
        if let Some(valid_until) = self.valid_until {
            if Utc::now() > valid_until {
                return false;
            }
        }

        true
    }

    /// 验证表意人的行为能力
    pub fn validate_capacity(&self) -> Result<(), &'static str> {
        // 检查表意人的行为能力
        if !self.declarant.has_capacity() {
            return Err("表意人无行为能力");
        }

        // 如果有相对人，也需要检查相对人的行为能力
        if let Some(ref recipient) = self.recipient {
            if !recipient.has_capacity() {
                return Err("相对人无行为能力");
            }
        }

        Ok(())
    }

    /// 撤回意思表示（在到达相对人之前）
    pub fn revoke(&mut self) -> Result<(), &'static str> {
        if self.status != DeclarationStatus::Created {
            return Err("只能撤回尚未生效的意思表示");
        }
        self.status = DeclarationStatus::Revoked;
        Ok(())
    }

    /// 撤销意思表示（在到达相对人之后）
    pub fn withdraw(&mut self) -> Result<(), &'static str> {
        if self.status != DeclarationStatus::Effective {
            return Err("只能撤销已经生效的意思表示");
        }
        self.status = DeclarationStatus::Withdrawn;
        Ok(())
    }

    /// 使意思表示生效
    pub fn make_effective(&mut self) -> Result<(), &'static str> {
        if self.status != DeclarationStatus::Created {
            return Err("只有新创建的意思表示才能生效");
        }
        self.status = DeclarationStatus::Effective;
        Ok(())
    }

    pub fn declaration_type(&self) -> DeclarationType {
        self.declaration_type.clone()
    }

    pub fn declarant(&self) -> Arc<dyn Entity> {
        self.declarant.clone()
    }

    pub fn recipient(&self) -> Option<Arc<dyn Entity>> {
        self.recipient.clone()
    }

    pub fn content(&self) -> IntentContent {
        self.content.clone()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn valid_until(&self) -> Option<DateTime<Utc>> {
        self.valid_until
    }

    pub fn status(&self) -> DeclarationStatus {
        self.status.clone()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;
    use crate::core::entity;

    #[test]
    fn test_intent_declaration_with_entities() {
        let birthday = Utc.with_ymd_and_hms(1990, 1, 1, 0, 0, 0).unwrap();
        // 创建一个自然人作为表意人
        let declarant = Arc::new(entity::NaturalPerson::new(birthday, entity::MentalStatus::Normal));

        // 创建一个公司法定代表人
        let legal_representative = entity::NaturalPerson::new(birthday, entity::MentalStatus::Normal);

        // 创建一个法人作为相对人
        let recipient = Arc::new(entity::LegalPerson::new(entity::LegalPersonType::Company(entity::CompanyType::Limited), 1000000.0, legal_representative.id(), "北京".to_string(), Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap()));

        // 创建意思表示
        let declaration = IntentDeclaration::new(
            DeclarationType::Offer,
            declarant,
            Some(recipient),
            IntentContent::default(),
            None,
        );

        // 验证行为能力
        assert!(declaration.validate_capacity().is_ok());
    }
}