pub mod contract;
pub mod core;
pub use core::*;
pub mod error;

pub mod validate;
pub use error::*;

// 库的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
//
// /// 框架统一错误类型
// #[derive(Debug)]
// pub enum FanError {
//     /// 验证错误：当法律规范验证失败时
//     ValidationError(&'static str),
//
//     /// 锁错误：在并发操作中发生的锁相关错误
//     LockError(String),
//
//     /// 参数错误：传入参数不符合要求
//     InvalidArgument(String),
//
//     /// 状态错误：实体状态不允许某操作
//     StateError(String),
//
//     /// 权限错误：没有权限执行某操作
//     PermissionError(String),
//
//     /// 不支持的操作
//     UnsupportedOperation(String),
// }
//
// /// 框架统一结果类型
// pub type FanResult<T> = Result<T, FanError>;
