//! 合同的基础定义
//! 包括合同的基本特征和通用结构

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;

use crate::{FanError, FanResult};
use crate::core::entity::Entity;
use super::intent::declaration::{IntentDeclaration, DeclarationType};

/// 合同条款
#[derive(Debug, Clone)]
pub struct ContractTerm {
    /// 条款序号
    number: u32,
    /// 条款内容
    content: String,
}

/// 合同状态
#[derive(Debug, Clone, PartialEq)]
pub enum ContractStatus {
    /// 合同已订立但尚未生效
    Created,
    /// 合同已经生效
    Effective,
    /// 合同正在履行中
    InProgress,
    /// 合同已经履行完毕
    Completed,
    /// 合同被解除
    Terminated,
    /// 合同被撤销
    Revoked,
    /// 合同无效
    Invalid,
}

/// 合同基本特征
pub trait Contract {
    /// 获取合同ID
    fn id(&self) -> Uuid;

    /// 获取合同当事人
    fn parties(&self) -> &[Arc<dyn Entity>];

    /// 获取合同创建时间
    fn created_at(&self) -> DateTime<Utc>;

    /// 获取合同状态
    fn status(&self) -> ContractStatus;

    /// 验证合同的有效性
    fn validate(&self) -> FanResult<()>;

    /// 使合同生效
    fn make_effective(&mut self) -> FanResult<()>;

    /// 解除合同
    fn terminate(&mut self) -> FanResult<()>;
}

// /// 合同类型
// #[derive(Debug, Clone, PartialEq)]
// pub enum ContractType {
//     /// 有名合同
//     Sale,                   // 买卖合同
//     Lease,                  // 租赁合同
//     Guarantee,              // 保证合同
//     Construction,           // 建设工程合同
//     Technology,             // 技术合同
//     Storage,                // 仓储合同
//     Commission,             // 委托合同
//     // ... 其他典型合同类型
//
//     /// 非典型合同（带名称）
//     Atypical(String),
// }

/// 基础合同结构
#[derive(Debug)]
pub struct BaseContract {
    /// 合同ID
    id: Uuid,
    /// 合同当事人
    parties: Vec<Arc<dyn Entity>>,
    /// 订立合同过程中的意思表示
    intent_declarations: Vec<IntentDeclaration>,
    /// 合同条款
    terms: Vec<ContractTerm>,
    /// 创建时间
    created_at: DateTime<Utc>,
    /// 生效时间
    effective_at: Option<DateTime<Utc>>,
    /// 履行期限
    time_limit: Option<DateTime<Utc>>,
    /// 合同状态
    status: ContractStatus,
}

impl BaseContract {
    /// 创建新的合同
    ///
    /// # 参数 Parameters
    ///
    /// - `parties`: 合同参与方的列表，每个参与方都是一个实现了`Entity` trait的对象
    /// - `intent_declarations`: 意图声明的列表，明确了合同中各方的意图
    /// - `terms`: 合同条款的列表，详细说明了合同的条款内容
    /// - `time_limit`: 合同的时间限制，为一个可选的`DateTime`对象，表示合同的截止时间
    ///
    /// # 返回 Returns
    ///
    /// 返回一个`Contract`实例，表示新创建的合同
    ///
    /// # Description
    ///
    /// 此函数负责初始化并返回一个新的合同对象它接受合同参与方、意图声明、条款以及时间限制作为参数，并使用这些参数来构建一个唯一的合同对象
    /// 合同的`id`字段被赋予一个新的UUID，以确保其唯一性`created_at`字段被设置为当前时间，以记录合同的创建时间
    /// 合同的`status`字段被设置为`Created`，表示合同已创建但尚未生效
    pub fn new(
        parties: Vec<Arc<dyn Entity>>,
        intent_declarations: Vec<IntentDeclaration>,
        terms: Vec<ContractTerm>,
        time_limit: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            parties,
            intent_declarations,
            terms,
            created_at: Utc::now(),
            effective_at: None,
            time_limit,
            status: ContractStatus::Created,
        }
    }

    /// 检查当事人的主体资格
    fn validate_parties(&self) -> FanResult<()> {
        // 检查当事人数量
        if self.parties.is_empty() {
            return Err(FanError::ValidationError("合同当事人不能为空".to_string()));
        }

        // 检查每个当事人的行为能力
        for party in &self.parties {
            if !party.has_capacity() {
                return Err(FanError::ValidationError("当事人缺乏必要的行为能力".to_string()));
            }
        }

        Ok(())
    }

    /// 验证意思表示的一致性
    fn validate_declarations(&self) -> FanResult<()> {
        // 要约
        let offer = self.intent_declarations.iter()
            .find(|d| matches!(d.declaration_type(), DeclarationType::Offer))
            .ok_or_else(|| FanError::ValidationError("缺少要约".to_string()))?;

        // 承诺
        let acceptance = self.intent_declarations.iter()
            .find(|d| matches!(d.declaration_type(), DeclarationType::Acceptance))
            .ok_or_else(|| FanError::ValidationError("缺少承诺".to_string()))?;

        // 检查要约和承诺的实质性内容是否一致
        if !offer.content().matches_essential_terms(&acceptance.content()) {
            return Err(FanError::ValidationError("要约和承诺的实质性内容不一致".to_string()));
        }

        Ok(())
    }
}

impl Contract for BaseContract {
    fn id(&self) -> Uuid {
        self.id
    }

    fn parties(&self) -> &[Arc<dyn Entity>] {
        &self.parties
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn status(&self) -> ContractStatus {
        self.status.clone()
    }

    fn validate(&self) -> FanResult<()> {
        // 验证当事人
        self.validate_parties()?;

        // 验证意思表示
        self.validate_declarations()?;

        Ok(())
    }

    fn make_effective(&mut self) -> FanResult<()> {
        // 验证合同
        self.validate()?;

        // 更新状态
        self.status = ContractStatus::Effective;
        self.effective_at = Some(Utc::now());

        Ok(())
    }

    fn terminate(&mut self) -> FanResult<()> {
        // 检查是否可以解除
        if self.status != ContractStatus::Effective && self.status != ContractStatus::InProgress {
            return Err(FanError::ValidationError("只有生效的合同才能解除".to_string()));
        }

        // 更新状态
        self.status = ContractStatus::Terminated;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_lifecycle() {
        // 创建测试数据
        let mut contract = BaseContract::new(
            vec![], // 需要添加测试用的当事人
            vec![], // 需要添加测试用的意思表示
            vec![], // 需要添加测试用的合同条款
            None,
        );

        // 测试合同状态变化
        assert_eq!(contract.status(), ContractStatus::Created);

        // TODO: 添加更多具体的测试用例
    }
}