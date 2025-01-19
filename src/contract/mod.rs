//! 合同模块
//! 实现合同相关的核心功能，包括意思表示、合同订立等

pub mod base;
pub mod intent;
pub mod types;
pub mod typical;

// 重导出常用类型
pub use base::{BaseContract, Contract};
pub use intent::content::IntentContent;
pub use intent::declaration::{DeclarationType, IntentDeclaration};
pub use typical::TypicalContract;
