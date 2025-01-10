pub mod core;
pub mod validate;
pub mod contract;

// 库的版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// 全局错误类型
#[derive(Debug)]
pub enum FanError {
    ValidationError(String),
    // 后续可以添加更多错误类型
}

// 结果类型别名，方便使用
pub type FanResult<T> = Result<T, FanError>;

fn main() {
    println!("The version of this program is: {}", VERSION);
}