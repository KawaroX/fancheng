//! 典型合同模块

use crate::{FanError, FanResult};
use crate::contract::Contract;

/// 典型合同特征
pub trait TypicalContract: Contract {
    /// 验证合同是否符合法定要求
    /// 典型合同必须符合法律规定的特定要求
    fn validate_legal_requirements(&self) -> FanResult<()>;
}