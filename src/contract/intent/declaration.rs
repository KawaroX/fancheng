//! 意思表示的核心定义
//! 包括意思表示的类型、结构和基本行为

use chrono::{DateTime, Utc};
use uuid::Uuid;
use super::content::IntentContent;
use crate::core::entity::Entity;
use std::sync::Arc;
use crate::{FanError, FanResult, ValidationErrorType};

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

/// # 意思表示的状态
/// - Created：创建但尚未生效
/// - Effective：生效
/// - Revoked：撤回
/// - Withdrawn：撤销
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
    /// 基于意思表示实质内容计算的匹配码
    /// 用于判断意思表示是否实质性一致
    match_code: String,
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
    /// 意思表示到达相对人的时间
    delivered_at: Option<DateTime<Utc>>,
    /// 意思表示的当前状态，通过DeclarationStatus枚举来定义意思表示的当前状态，如生效、失效等。
    status: DeclarationStatus,
}


impl IntentDeclaration {
    /// # 创建新的意思表示
    ///
    /// 本方法用于创建一个新的意思表示，包括表意人、相对人、具体内容、有效期等信息。
    ///
    /// - `declaration_type`: 意思表示的类型，决定了意思表示的性质。
    /// - `declarant`: 表意人，即发出意思表示的一方，使用实体接口的实现。
    /// - `recipient`: 可选的相对人，即接收意思表示的一方，使用实体接口的实现。
    /// - `content`: 意思表示的具体内容。
    /// - `valid_until`: 可选的有效期，表示意思表示在特定时间之前有效。
    ///
    /// 返回一个新的意思表示实例，包含上述信息及创建时间、状态等元数据。
    pub fn new(
        declaration_type: DeclarationType,
        declarant: Arc<dyn Entity>,
        recipient: Option<Arc<dyn Entity>>,
        content: IntentContent,
        valid_until: Option<DateTime<Utc>>,
    ) -> FanResult<Self> {
        // 先验证表意人的行为能力
        if !declarant.has_capacity() {
            return Err(FanError::validation(
                "表意人无行为能力",
                ValidationErrorType::EntityCapacityLacking,
                "new_intent_declaration",
                "IntentDeclaration",
            ))
        }

        // 验证相对人的行为能力
        if let Some(ref r) = recipient {
            if !r.has_capacity() {
                return Err(FanError::validation(
                    "相对人无行为能力",
                    ValidationErrorType::EntityCapacityLacking,
                    "new_intent_declaration",
                    "IntentDeclaration",
                ))
            }
        }

        let mut instance = Self {
            id: Uuid::new_v4(),
            match_code: String::new(),  // 临时空值
            declaration_type,
            declarant,
            recipient,
            content,
            created_at: Utc::now(),
            valid_until,
            delivered_at: None,
            status: DeclarationStatus::Created,
        };

        // 计算并设置哈希值
        instance.match_code = instance.calculate_match_code();

        Ok(instance)
    }

    /// # 计算内容哈希值
    ///
    /// 本函数旨在为当前声明计算一个唯一的哈希值，该哈希值基于当事人ID、内容的必要哈希值和声明类型。
    /// 这用于确保声明的完整性和唯一性。
    // fn calculate_match_code(&self) -> String {
    //     // 获取当事人ID并排序
    //     // 如果存在接收方，则也获取其ID
    //     // 排序是为了确保哈希值的一致性，无论当事人和接收方的顺序如何
    //     let mut ids = vec![self.declarant.id()];
    //     if let Some(ref recipient) = self.recipient {
    //         ids.push(recipient.id());
    //     }
    //     ids.sort();
    //
    //     // 组合必要内容
    //     // 将排序后的ID、内容的必要哈希值和声明类型组合成一个字符串
    //     // 这是为了确保所有相关数据都被包含在最终的哈希值中
    //     let content = format!(
    //         "{}_{}_{}",
    //         ids.join("_"),
    //         self.content.essential_hash(),
    //         self.declaration_type.to_string()
    //     );
    //
    //     use sha2::{Sha256, Digest};
    //     let mut hasher = Sha256::new();
    //     hasher.update(content);
    //     format!("{:x}", hasher.finalize())
    // }

    fn calculate_match_code(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();

        // 先收集ID到Vec中
        let mut party_ids = vec![self.declarant.id()];
        if let Some(ref recipient) = self.recipient {
            party_ids.push(recipient.id());
        }
        // 固定排序
        party_ids.sort();

        // 使用固定的分隔符连接
        let party_str = party_ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("|");  // 使用不太可能出现在其他地方的分隔符

        hasher.update(party_str.as_bytes());
        hasher.update(self.content.essential_hash().as_bytes());

        hex::encode(hasher.finalize())
    }

    /// 检查是否能够与另一个意思表示构成合同
    /// 主要用于要约和承诺的匹配
    pub fn can_form_contract_with(&self, other: &IntentDeclaration) -> bool {
        // 先检查双方意思表示是否有效
        if !self.is_valid() || !other.is_valid() {
            println!("Invalid declarations");
            return false;
        }

        // 检查实质性内容是否一致
        self.match_code == other.match_code
    }

    /// 获取内容哈希值
    pub fn match_code(&self) -> &str {
        &self.match_code
    }

    /// 检查两个意思表示是否实质性一致
    pub fn matches(&self, other: &IntentDeclaration) -> bool {
        self.match_code == other.match_code || (
            self.declaration_type == other.declaration_type
                && self.declarant.id() == other.declarant.id()
                && self.recipient.as_ref().map(|r| r.id()) == other.recipient.as_ref().map(|r| r.id())
        )
    }

    /// 判断意思表示是否仍然有效
    ///
    /// 本函数主要通过检查意思表示的状态以及其有效期来判断其是否仍然有效
    /// 如果意思表示的状态不是有效状态，或者当前时间超过了其有效期，则认为该意思表示无效
    ///
    /// # 返回 Returns
    ///
    /// * `bool` - 返回一个布尔值，表示意思表示是否有效
    pub fn is_valid(&self) -> bool {
        // 检查状态
        if self.status != DeclarationStatus::Effective {
            println!("Invalid status");
            return false;
        }

        // 检查是否在有效期内
        if let Some(valid_until) = self.valid_until {
            if Utc::now() > valid_until {
                println!("Expired");
                return false;
            }
        }
        println!("Valid");
        true
    }

    /// 验证表意人的行为能力
    pub fn validate_capacity(&self) -> FanResult<()> {
        // 检查表意人的行为能力
        if !self.declarant.has_capacity() {
            return Err(FanError::validation(
                "表意人无行为能力",
                ValidationErrorType::EntityCapacityLacking,
                "validate_capacity",
                "IntentDeclaration",
            ))
        }

        // 如果有相对人，也需要检查相对人的行为能力
        if let Some(ref recipient) = self.recipient {
            if !recipient.has_capacity() {
                return Err(FanError::validation(
                    "相对人无行为能力",
                    ValidationErrorType::EntityCapacityLacking,
                    "validate_capacity",
                    "IntentDeclaration",
                ))
            }
        }

        Ok(())
    }

    /// 撤回意思表示（在到达相对人之前）
    pub fn revoke(&mut self) -> FanResult<()> {
        if self.status != DeclarationStatus::Created {
            return Err(FanError::validation(
                "只能撤回尚未生效的意思表示",
                ValidationErrorType::IntentStatusVoid,
                "revoke",
                "IntentDeclaration",
            ))
        }
        self.status = DeclarationStatus::Revoked;
        Ok(())
    }

    /// 撤销意思表示（在到达相对人之后）
    pub fn withdraw(&mut self) -> FanResult<()> {
        if self.status != DeclarationStatus::Effective {
            return Err(FanError::validation(
                "只能撤销已经生效的意思表示",
                ValidationErrorType::IntentStatusVoid,
                "withdraw",
                "IntentDeclaration",
            ))
        }
        self.status = DeclarationStatus::Withdrawn;
        Ok(())
    }

    /// 使意思表示生效
    pub fn make_effective(&mut self) -> FanResult<()> {
        if self.status != DeclarationStatus::Created {
            return Err(FanError::validation(
                "只有新创建的意思表示才能生效",
                ValidationErrorType::IntentStatusVoid,
                "make_effective",
                "IntentDeclaration",
            ))
        }
        self.status = DeclarationStatus::Effective;
        Ok(())
    }

    /// 标记意思表示已到达相对人
    pub fn mark_as_delivered(&mut self) -> FanResult<()> {
        self.delivered_at = Some(Utc::now());
        self.status = DeclarationStatus::Effective;
        Ok(())
    }

}

impl IntentDeclaration {

    /// 获取意思表示的类型
    pub fn declaration_type(&self) -> DeclarationType {
        self.declaration_type.clone()
    }

    /// 获取意思表示的声明人
    pub fn declarant(&self) -> Arc<dyn Entity> {
        self.declarant.clone()
    }

    /// 获取意思表示的相对人
    pub fn recipient(&self) -> Option<Arc<dyn Entity>> {
        self.recipient.clone()
    }

    /// 获取意思表示的内容
    pub fn content(&self) -> IntentContent {
        self.content.clone()
    }

    /// 获取意思表示的创建时间
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// 获取意思表示的有效期
    pub fn valid_until(&self) -> Option<DateTime<Utc>> {
        self.valid_until
    }

    /// 获取意思表示的状态
    pub fn status(&self) -> DeclarationStatus {
        self.status.clone()
    }

    /// 获取意思表示的ID
    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use chrono::TimeZone;
    use crate::core::entity::{
        Entity, NaturalPerson, MentalStatus, LegalPerson, LegalPersonType, CompanyType
    };
    use std::sync::Arc;
    use rust_decimal::Decimal;
    use crate::contract::intent::content::{Quantity, QuantityUnit, SubjectMatter, SubjectMatterType};

    fn test_content() -> IntentContent {
        IntentContent::new(
            SubjectMatter::new(Uuid::new_v4(), SubjectMatterType::GenericGoods, "测试商品".to_string(), Some("商品描述".to_string())),
            Some(Quantity {
                amount: Decimal::try_from(1.0).unwrap(),
                unit: QuantityUnit::Piece,
            }),
            None,
            Some(crate::contract::intent::content::Price::new(Decimal::try_from(100.0).unwrap(), "CNY".to_string(), "现金".to_string())),
            None,
            None,
        )
    }
    #[test]
    fn test_matching_intent_declarations() {
        // 创建两个自然人作为测试主体
        let person_a = Arc::new(NaturalPerson::new(
            // 创建一个20岁的成年人
            Utc::now() - Duration::days(365 * 20),
            MentalStatus::Normal,
        ));

        let person_b = Arc::new(NaturalPerson::new(
            Utc::now() - Duration::days(365 * 20),
            MentalStatus::Normal,
        ));

        // 创建相同内容的意思表示
        let offer_content = test_content();

        // A向B发出要约
        let mut declaration_a = IntentDeclaration::new(
            DeclarationType::Offer,
            person_a.clone(),
            Some(person_b.clone()),
            offer_content.clone(),
            None,
        ).unwrap();

        declaration_a.mark_as_delivered().unwrap();

        // B向A发出要约或承诺
        let mut declaration_b = IntentDeclaration::new(
            DeclarationType::Acceptance,
            person_b.clone(),
            Some(person_a.clone()),
            offer_content.clone(),
            None,
        ).unwrap();

        declaration_b.mark_as_delivered().unwrap();

        println!("Declaration A match_code calculation:");
        println!("Content hash: {:?}", declaration_a.content.essential_hash());
        println!("Match code A: {}", declaration_a.match_code);
        println!("Valid until: {:?}", declaration_a.valid_until);
        println!("Status: {:?}", declaration_a.status);

        println!("\nDeclaration B match_code calculation:");
        println!("Content hash: {:?}", declaration_b.content.essential_hash());
        println!("Match code B: {}", declaration_b.match_code);
        println!("Valid until: {:?}", declaration_b.valid_until);
        println!("Status: {:?}", declaration_b.status);

        // 验证match_code相同
        assert_eq!(declaration_a.match_code, declaration_b.match_code);
        // 验证能否形成合同
        assert!(declaration_a.can_form_contract_with(&declaration_b));
    }

    #[test]
    fn test_intent_declaration_with_no_capacity() {
        // 创建一个无民事行为能力的自然人（成年但精神状态受损）
        let incapacitated_person = Arc::new(NaturalPerson::new(
            Utc::now() - Duration::days(365 * 30),
            MentalStatus::SeverelyImpaired,
        ));

        let normal_person = Arc::new(NaturalPerson::new(
            Utc::now() - Duration::days(365 * 30),
            MentalStatus::Normal,
        ));

        let content = test_content();

        // 无民事行为能力人不能发出意思表示
        let result = IntentDeclaration::new(
            DeclarationType::Offer,
            incapacitated_person.clone(),
            Some(normal_person.clone()),
            content.clone(),
            None,
        );

        assert!(result.is_err());
    }

        #[test]
        fn test_intent_declaration_with_entities() {
            let birthday = Utc.with_ymd_and_hms(1990, 1, 1, 0, 0, 0).unwrap();
            // 创建一个自然人作为表意人
            let declarant = Arc::new(NaturalPerson::new(birthday, MentalStatus::Normal));

            // 创建一个公司法定代表人
            let legal_representative = NaturalPerson::new(birthday, MentalStatus::Normal);

            // 创建一个法人作为相对人
            let recipient = Arc::new(
                LegalPerson::new(
                    LegalPersonType::Company(CompanyType::Limited),
                    1000000.0,
                    legal_representative.id(),
                    "北京".to_string(),
                    Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap())
            );

            // 创建意思表示
            let declaration = IntentDeclaration::new(
                DeclarationType::Offer,
                declarant,
                Some(recipient),
                IntentContent::default(),
                None,
            );

            // 验证行为能力
            assert!(declaration.unwrap().validate_capacity().is_ok());
        }
}