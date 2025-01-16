//! 意思表示模块
//! 实现合同订立过程中的意思表示相关功能
//! 包括要约、承诺等意思表示的具体实现

pub mod declaration;
pub mod content;

pub use declaration::{IntentDeclaration, DeclarationType};
pub use content::IntentContent;