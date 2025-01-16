//! 合同模块
//! 实现合同相关的核心功能，包括意思表示、合同订立等

pub mod intent;
pub mod base;
pub mod typical;
pub mod types;

// 重导出常用类型
pub use base::{Contract, BaseContract};
pub use typical::TypicalContract;
pub use intent::declaration::{IntentDeclaration, DeclarationType};
pub use intent::content::IntentContent;