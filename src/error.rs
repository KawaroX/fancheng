use chrono::{DateTime, Utc};

/// 法律规范验证错误类型
#[derive(Debug)]
pub enum ValidationErrorType {
    // Entity相关错误
    EntityCapacityLacking,   // 能力不足
    EntityStatusIllegal,     // 实体状态不合法
    EntityRelationMalformed, // 实体关系不符合要求
    EntityError,             // 其他实体错误

    // Intent相关错误
    IntentContentMalformed, // 意思表示内容不合格
    IntentStatusVoid,       // 意思表示状态无效
    IntentMatchFailure,     // 意思表示匹配失败

    // Contract相关错误
    ContractElementMissing,   // 合同要素缺失
    ContractContentIllegal,   // 合同内容不合法
    ContractPartyUnqualified, // 合同当事人不适格
    ContractStatusIllegal,    // 合同状态不合法

    // Operation相关错误
    OperationUnauthorized,  // 未授权的操作
    OperationTimingWrong,   // 操作时机不当
    OperationSequenceWrong, // 操作顺序错误
}

/// 框架统一错误类型
#[derive(Debug)]
pub enum FanError {
    /// 法律规范验证错误
    ValidationError {
        message: String,
        error_type: ValidationErrorType,
        legal_reference: Option<String>,
        context: ErrorContext,
    },

    /// 程序运行错误
    SystemError {
        message: String,
        error_type: &'static str,
    },
}

/// 错误上下文
#[derive(Debug)]
pub struct ErrorContext {
    /// 执行的操作
    operation: String,
    /// 错误发生的位置
    location: String,
    /// 相关实体的ID
    entity_ids: Vec<String>,
    /// 错误发生时间
    timestamp: DateTime<Utc>,
}

impl ErrorContext {
    /// 创建一个新的 `ErrorContext`，基于给定的操作和位置信息。
    ///
    /// # 参数
    ///
    /// * `operation`: 导致错误的操作描述。
    /// * `location`: 错误发生的位置。
    ///
    /// # 返回值
    ///
    /// 返回一个包含给定值的新 `ErrorContext` 实例。
    pub fn new(operation: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            // 将给定的操作转换为字符串并赋值给 operation 字段
            operation: operation.into(),
            // 将给定的位置转换为字符串并赋值给 location 字段
            location: location.into(),
            // 初始化 entity_ids 向量为空
            entity_ids: Vec::new(),
            // 设置当前时间为错误上下文的时间戳
            timestamp: Utc::now(),
        }
    }

    pub fn add_entity_id(&mut self, id: impl Into<String>) -> &mut Self {
        self.entity_ids.push(id.into());
        self
    }
}

/// 框架统一结果类型
pub type FanResult<T> = Result<T, FanError>;

// 便捷方法
impl FanError {
    /// 创建一个验证错误实例
    ///
    /// 该函数用于生成一个特定的错误实例，用于表示在执行某个操作时发生的验证错误
    /// 它允许通过指定错误消息、错误类型、操作名称和错误位置来详细描述错误
    ///
    /// # 参数
    /// - `message`: 转换为`String`类型的错误消息，描述验证失败的具体情况
    /// - `error_type`: 错误类型，可能的值包括但不限于`RequiredFieldMissing`、`InvalidFormat`等
    /// - `operation`: 转换为`String`类型的操作名称，表示在执行哪个操作时发生了错误
    /// - `location`: 转换为`String`类型的错误位置，指示验证失败发生的具体位置
    ///
    /// # 返回值
    /// 返回一个`Self`类型的实例，其中`Self`是一个包含验证错误详细信息的类型
    /// 该实例可用于进一步的错误处理或日志记录
    pub fn validation(
        message: impl Into<String>,
        error_type: ValidationErrorType,
        operation: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            error_type,
            legal_reference: None,
            context: ErrorContext::new(operation, location),
        }
    }

    /// 创建一个带有引用的验证错误
    ///
    /// 该函数用于生成一个验证错误，其中包含了错误消息、错误类型、合法引用以及操作和位置信息
    /// 它主要用于在进行数据验证时，当遇到不符合预期的数据时创建错误实例
    ///
    /// # 参数
    /// - `message`: 错误消息，描述了验证失败的原因
    /// - `error_type`: 错误类型，指明了验证错误的种类
    /// - `legal_reference`: 合法引用，提供了正确的参考值或规则
    /// - `operation`: 操作名称，描述了正在进行的操作
    /// - `location`: 位置信息，指明了发生错误的位置
    ///
    /// # 返回
    /// 返回一个`Self`类型的实例，其中`Self`是一个包含验证错误信息的类型
    pub fn validation_with_ref(
        message: impl Into<String>,
        error_type: ValidationErrorType,
        legal_reference: impl Into<String>,
        operation: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            error_type,
            legal_reference: Some(legal_reference.into()),
            context: ErrorContext::new(operation, location),
        }
    }

    /// 创建一个系统错误实例。
    ///
    /// 此函数用于生成系统错误对象，允许用户指定错误消息和错误类型。
    /// 它主要用于系统级错误处理，以便在程序中统一处理不同类型的错误。
    ///
    /// # 参数
    /// - `message`: 转换为`String`类型的错误消息。这可以是任何实现了`Into<String>` trait的类型，
    ///              允许灵活地传递错误信息。
    /// - `error_type`: 静态字符串，表示错误类型。使用静态字符串确保错误类型是已知且不变的，
    ///                 便于错误处理和日志记录。
    ///
    /// # 返回值
    /// 返回`Self`类型的实例，具体来说是一个`SystemError`变体。
    /// 这种设计允许错误处理代码以一致和类型安全的方式访问错误信息。
    pub fn system(message: impl Into<String>, error_type: &'static str) -> Self {
        Self::SystemError {
            message: message.into(),
            error_type,
        }
    }
}
