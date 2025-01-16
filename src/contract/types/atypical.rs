//! 非典型合同模块
//! 实现非典型合同相关的功能，包括创建、验证、生效等

use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use crate::{FanError, FanResult};
use crate::contract::IntentDeclaration;
use crate::entity::Entity;
use super::super::base::{Contract, BaseContract, ContractStatus};

/// 非典型合同
/// 用于处理法律未规定具体类型的合同关系
#[derive(Debug)]
pub struct AtypicalContract {
    /// 基础合同结构
    base: BaseContract,
    /// 合同名称（用于标识合同类型）
    name: String,
}

impl AtypicalContract {
    /// 创建新的非典型合同
    pub fn new(
        name: String,
        parties: Vec<Arc<dyn Entity>>,
        intent_declarations: Vec<IntentDeclaration>,
        time_limit: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            base: BaseContract::new(parties, intent_declarations, Vec::new(), time_limit),
            name,
        }
    }

    /// 获取合同名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Contract for AtypicalContract {
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
        // 非典型合同只需要验证基本的合同要素
        self.base.validate()
    }

    fn make_effective(&mut self) -> FanResult<()> {
        self.base.make_effective()
    }

    fn terminate(&mut self) -> FanResult<()> {
        self.base.terminate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atypical_contract() {
        // TODO: 添加测试用例
    }
}