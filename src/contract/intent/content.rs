//! 意思表示的具体内容
//! 包括合同的标的物、数量、质量、价款等实质性内容

use std::cmp::PartialEq;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// 标的物类型
#[derive(Debug, Clone, PartialEq)]
pub enum SubjectMatterType {
    /// 特定物
    SpecificGoods,
    /// 种类物
    GenericGoods,
    /// 特定服务
    Service,
    /// 知识产权
    IntellectualProperty,
    /// 其他类型
    Other(String),
}

/// 标的物
#[derive(Debug, Clone)]
pub struct SubjectMatter {
    /// 标的物ID
    id: Uuid,
    /// 标的物类型
    subject_type: SubjectMatterType,
    /// 标的物名称
    name: String,
    /// 标的物描述
    description: Option<String>,
}

/// 数量单位
#[derive(Debug, Clone, PartialEq)]
pub enum QuantityUnit {
    Piece,    // 个
    Kilogram, // 千克
    Meter,    // 米
    Square,   // 平方米
    Cubic,    // 立方米
    Other(String),
}

/// 数量
#[derive(Debug, Clone)]
pub struct Quantity {
    /// 数值
    amount: Decimal,
    /// 单位
    unit: QuantityUnit,
}

/// 质量要求
#[derive(Debug, Clone)]
pub struct Quality {
    /// 质量标准
    standard: String,
    /// 具体要求
    requirements: Vec<String>,
    /// 质量保证期限
    warranty_period: Option<DateTime<Utc>>,
}

/// 价款或报酬
#[derive(Debug, Clone)]
pub struct Price {
    /// 金额
    amount: Decimal,
    /// 币种
    currency: String,
    /// 支付方式
    payment_method: String,
    /// 支付期限
    payment_deadline: Option<DateTime<Utc>>,
}

/// 履行地点
#[derive(Debug, Clone)]
pub struct Location {
    /// 地址
    address: String,
    /// 具体要求（如交付方式等）
    requirements: Option<String>,
}

/// 履行期限
#[derive(Debug, Clone)]
pub struct TimeLimit {
    /// 开始时间
    start_time: Option<DateTime<Utc>>,
    /// 结束时间
    end_time: DateTime<Utc>,
    /// 是否分期履行
    is_installment: bool,
    /// 分期履行的具体安排
    installment_plan: Option<Vec<DateTime<Utc>>>,
}

/// 意思表示的具体内容
#[derive(Debug, Clone)]
pub struct IntentContent {
    /// 标的物
    pub subject_matter: SubjectMatter,
    /// 数量
    pub quantity: Option<Quantity>,
    /// 质量要求
    pub quality: Option<Quality>,
    /// 价款或报酬
    pub price: Option<Price>,
    /// 履行期限
    pub time_limit: Option<TimeLimit>,
    /// 履行地点
    pub location: Option<Location>,
    /// 附随义务
    pub additional_obligations: Vec<String>,
    /// 其他条款
    pub additional_terms: HashMap<String, String>,
}

impl PartialEq for SubjectMatter {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id || (self.name == other.name && self.subject_type == other.subject_type) {
            true
        } else {
            false
        }
    }
}

impl Default for SubjectMatter {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            subject_type: SubjectMatterType::Other("".to_string()),
            name: "".to_string(),
            description: None,
        }
    }
}

impl IntentContent {
    /// 创建新的意思表示内容
    ///
    /// 该构造函数用于初始化一个意思表示内容的实例，其中包含了关于交易或协议的关键条款。
    /// 通过此方法，可以指定交易的主题、数量、质量、价格、时间限制和地点等参数。
    /// 未指定的参数将被视为未约定，可以在后续的协商中添加。
    ///
    /// # 参数 Arguments
    /// - `subject_matter`: 主题，表示交易或协议的主题内容。
    /// - `quantity`: 数量，可选参数，表示交易涉及的数量。
    /// - `quality`: 质量，可选参数，表示交易标的的质量标准。
    /// - `price`: 价格，可选参数，表示交易的价格。
    /// - `time_limit`: 时间限制，可选参数，表示交易需要在特定时间内完成。
    /// - `location`: 地点，可选参数，表示交易发生的地点。
    ///
    /// # 返回值 Returns
    /// 返回一个意思表示内容的实例，其中包含了提供的参数值，以及初始化的附加义务和条款。
    pub fn new(
        subject_matter: SubjectMatter,
        quantity: Option<Quantity>,
        quality: Option<Quality>,
        price: Option<Price>,
        time_limit: Option<TimeLimit>,
        location: Option<Location>,
    ) -> Self {
        Self {
            subject_matter,
            quantity,
            quality,
            price,
            time_limit,
            location,
            // 初始化为空的向量，用于存储附加义务
            additional_obligations: Vec::new(),
            // 初始化为空的哈希映射，用于存储额外条款
            additional_terms: HashMap::new(),
        }
    }

    /// 判断是否为实质性内容（合同的必要条款）
    pub fn is_essential(&self) -> bool {
        // 标的物是必须的
        if self.subject_matter.name.is_empty() {
            return false;
        }

        // 根据不同类型的合同，判断其他必要内容
        match self.subject_matter.subject_type {
            SubjectMatterType::SpecificGoods | SubjectMatterType::GenericGoods => {
                // 买卖合同必须有价款
                self.price.is_some()
            }
            SubjectMatterType::Service => {
                // 服务合同必须有履行期限和报酬
                self.time_limit.is_some() && self.price.is_some()
            }
            SubjectMatterType::IntellectualProperty => {
                // 知识产权合同必须有使用范围和报酬
                self.price.is_some() && !self.additional_terms.is_empty()
            }
            SubjectMatterType::Other(_) => {
                // 其他类型根据具体情况判断
                true
            }
        }
    }

    /// 添加附随义务
    ///
    /// # Parameters
    ///
    /// - `obligation`: 一个字符串，表示要添加的附随义务。
    ///
    /// # Description
    ///
    /// 此方法用于向对象中添加一个附随义务。附随义务是指在完成主要任务的同时需要额外注意或执行的事项。
    /// 通过此方法，可以将新的义务添加到对象的附随义务列表中，以便在后续操作中进行参考或处理。
    ///
    // # Examples
    //
    // ```
    // let mut obligations = Obligations::new();
    // obligations.add_obligation("Check the weather".to_string());
    // ```
    // TODO: Claude 犯病，我也不知道这个 Obligations 是什么，看看到时候是否需要设计，我感觉其实挺难的、、
    pub fn add_obligation(&mut self, obligation: String) {
        self.additional_obligations.push(obligation);
    }

    /// 添加其他条款
    pub fn add_term(&mut self, key: String, value: String) {
        self.additional_terms.insert(key, value);
    }

    /// 判断是否与另一个意思表示内容在实质性内容上一致
    pub fn matches_essential_terms(&self, other: &IntentContent) -> bool {
        // 标的物必须一致
        if self.subject_matter != other.subject_matter {
            return false;
        }

        // 价款必须一致（如果双方都指定了价款）
        if let (Some(self_price), Some(other_price)) = (&self.price, &other.price) {
            if self_price.amount != other_price.amount {
                return false;
            }
        }

        // 其他要素可以不完全一致
        true
    }
}

impl Default for IntentContent {
    fn default() -> Self {
        Self {
            subject_matter: SubjectMatter::default(),
            quantity: None,
            quality: None,
            price: None,
            time_limit: None,
            location: None,
            additional_obligations: Vec::new(),
            additional_terms: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_essential_terms() {
        // 创建一个买卖合同的意思表示内容
        let subject_matter = SubjectMatter {
            id: Uuid::new_v4(),
            subject_type: SubjectMatterType::SpecificGoods,
            name: "iPhone".to_string(),
            description: Some("iPhone 13 Pro Max".to_string()),
        };

        let price = Price {
            amount: Decimal::from(9999),
            currency: "CNY".to_string(),
            payment_method: "支付宝".to_string(),
            payment_deadline: None,
        };

        let content = IntentContent::new(
            subject_matter,
            None,
            None,
            Some(price),
            None,
            None,
        );

        // 买卖合同有标的物和价款就是实质性内容完整
        assert!(content.is_essential());
    }
}