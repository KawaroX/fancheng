//! 买卖合同的具体实现

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;

use crate::{FanError, FanResult, ValidationErrorType};
use crate::core::entity::Entity;
use super::super::base::{Contract, BaseContract, ContractStatus};
use super::super::typical::TypicalContract;

/// 标的物
#[derive(Debug, Clone)]
pub struct SubjectMatter {
    /// 标的物名称
    name: String,
    /// 标的物描述
    description: Option<String>,
    /// 数量
    quantity: f64,
    /// 单位
    unit: String,
    /// 质量要求
    quality_requirements: Vec<String>,
}

/// 价款
#[derive(Debug, Clone)]
pub struct Price {
    /// 金额
    amount: f64,
    /// 币种
    currency: String,
    /// 支付方式
    payment_method: String,
    /// 支付期限
    payment_deadline: Option<DateTime<Utc>>,
}

/// 买卖合同
#[derive(Debug)]
pub struct SaleContract {
    /// 基础合同内容
    base: BaseContract,
    /// 标的物
    subject: SubjectMatter,
    /// 价款
    price: Price,
    /// 交付时间
    delivery_time: Option<DateTime<Utc>>,
    /// 交付地点
    delivery_location: Option<String>,
}

impl SaleContract {
    /// 创建新的买卖合同
    pub fn new(
        base: BaseContract,
        subject: SubjectMatter,
        price: Price,
        delivery_time: Option<DateTime<Utc>>,
        delivery_location: Option<String>,
    ) -> Self {
        Self {
            base,
            subject,
            price,
            delivery_time,
            delivery_location,
        }
    }

    /// 获取标的物信息
    pub fn subject(&self) -> &SubjectMatter {
        &self.subject
    }

    /// 获取价款信息
    pub fn price(&self) -> &Price {
        &self.price
    }

    /// 获取交付时间
    pub fn delivery_time(&self) -> Option<DateTime<Utc>> {
        self.delivery_time
    }

    /// 获取交付地点
    pub fn delivery_location(&self) -> Option<&String> {
        self.delivery_location.as_ref()
    }
}

impl Contract for SaleContract {
    fn id(&self) -> Uuid {
        self.base.id()
    }

    fn parties(&self) -> &[Arc<dyn Entity>] {
        self.base.parties()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at()
    }

    fn status(&self) -> ContractStatus {
        self.base.status()
    }

    fn validate(&self) -> FanResult<()> {
        // 首先验证基础合同要求
        self.base.validate()?;
        // 然后验证买卖合同特定要求
        self.validate_legal_requirements()
    }

    fn make_effective(&mut self) -> FanResult<()> {
        self.base.make_effective()
    }

    fn terminate(&mut self) -> FanResult<()> {
        self.base.terminate()
    }
}

impl TypicalContract for SaleContract {
    fn validate_legal_requirements(&self) -> FanResult<()> {
        // 验证标的物
        if self.subject.name.is_empty() {
            return Err(FanError::validation(
                "标的物名称不能为空",
                ValidationErrorType::ContractElementMissing,
                "validate_legal_requirements",
                "SaleContract",
            ));
        }

        if self.subject.quantity <= 0.0 {
            return Err(FanError::validation(
                "标的物数量必须大于0",
                ValidationErrorType::ContractContentIllegal,
                "validate_legal_requirements",
                "SaleContract",
            ));
        }

        // 验证价款
        if self.price.amount <= 0.0 {
            return Err(FanError::validation(
                "价款必须大于0",
                ValidationErrorType::ContractContentIllegal,
                "validate_legal_requirements",
                "SaleContract",
            ));
        }

        // 验证当事人身份
        if self.base.parties().len() != 2 {
            return Err(FanError::validation(
                "买卖合同必须有且仅有两个当事人",
                ValidationErrorType::ContractPartyUnqualified,
                "validate_legal_requirements",
                "SaleContract",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sale_contract_validation() {
        // TODO: 实现具体的测试用例
    }
}