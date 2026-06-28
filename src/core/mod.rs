//! 核心类型定义 —— 整个PAPS系统的基石
//!
//! 定义了参数值、光谱、领域、情境、时间戳等基础类型。
//! 所有类型都经过精心设计，确保零逻辑漏洞。

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

// ============================================================================
// ParameterValue: 参数值 —— 系统的原子单位
// ============================================================================

/// 参数值 —— 一个心理功能在特定时刻的取值
///
/// 使用 f64 以保证精度，但语义上永远在 [0.0, 1.0] 或 [-1.0, 1.0] 范围内。
/// 这不是"特质"，而是"参数在特定关系/情境/时刻的坍缩值"。
///
/// # Invariants (由构造函数保证)
/// - `Normalized` 变体: value ∈ [0.0, 1.0]
/// - `Bipolar` 变体: value ∈ [-1.0, 1.0]
/// - `Unbounded` 变体: value ∈ ℝ (用于半衰期、时间等)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ParameterValue {
    /// 归一化值 [0.0, 1.0]，用于大多数参数
    Normalized(f64),
    /// 双极值 [-1.0, 1.0]，用于有方向性的参数（如趋近-回避）
    Bipolar(f64),
    /// 无界值，用于时间/频率类参数（如半衰期天数、采样率Hz）
    Unbounded(f64),
}

impl ParameterValue {
    /// 创建归一化值，自动钳制到 [0.0, 1.0]
    #[inline]
    pub fn normalized(v: f64) -> Self {
        debug_assert!(
            v.is_finite(),
            "ParameterValue::normalized called with non-finite value"
        );
        ParameterValue::Normalized(v.clamp(0.0, 1.0))
    }

    /// 创建双极值，自动钳制到 [-1.0, 1.0]
    #[inline]
    pub fn bipolar(v: f64) -> Self {
        debug_assert!(
            v.is_finite(),
            "ParameterValue::bipolar called with non-finite value"
        );
        ParameterValue::Bipolar(v.clamp(-1.0, 1.0))
    }

    /// 创建无界值
    #[inline]
    pub fn unbounded(v: f64) -> Self {
        debug_assert!(
            v.is_finite(),
            "ParameterValue::unbounded called with non-finite value"
        );
        ParameterValue::Unbounded(v.max(0.0))
    }

    /// 获取原始 f64 值
    #[inline]
    pub fn as_f64(&self) -> f64 {
        match self {
            ParameterValue::Normalized(v) | ParameterValue::Bipolar(v) | ParameterValue::Unbounded(v) => *v,
        }
    }

    /// 是否为零值
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.as_f64().abs() < f64::EPSILON
    }

    /// 钳制到合法范围
    #[inline]
    pub fn clamp_to_range(&mut self) {
        match self {
            ParameterValue::Normalized(v) => *v = v.clamp(0.0, 1.0),
            ParameterValue::Bipolar(v) => *v = v.clamp(-1.0, 1.0),
            ParameterValue::Unbounded(v) => *v = v.max(0.0),
        }
    }
}

impl fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterValue::Normalized(v) => write!(f, "{:.4}", v),
            ParameterValue::Bipolar(v) => write!(f, "{:+.4}", v),
            ParameterValue::Unbounded(v) => write!(f, "{:.4}", v),
        }
    }
}

// 序列化支持
impl Serialize for ParameterValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ParameterValue", 2)?;
        match self {
            ParameterValue::Normalized(v) => {
                s.serialize_field("type", "normalized")?;
                s.serialize_field("value", v)?;
            }
            ParameterValue::Bipolar(v) => {
                s.serialize_field("type", "bipolar")?;
                s.serialize_field("value", v)?;
            }
            ParameterValue::Unbounded(v) => {
                s.serialize_field("type", "unbounded")?;
                s.serialize_field("value", v)?;
            }
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for ParameterValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(rename = "type")]
            type_: String,
            value: f64,
        }
        let h = Helper::deserialize(deserializer)?;
        match h.type_.as_str() {
            "normalized" => Ok(ParameterValue::normalized(h.value)),
            "bipolar" => Ok(ParameterValue::bipolar(h.value)),
            "unbounded" => Ok(ParameterValue::unbounded(h.value)),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown ParameterValue type: {}",
                h.type_
            ))),
        }
    }
}

// ============================================================================
// ParameterSpectrum: 参数光谱 —— 定义参数的取值空间
// ============================================================================

/// 光谱类型 —— 参数取值空间的数学描述
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpectrumType {
    /// 单维归一化 [0, 1]
    Normalized,
    /// 双极 [-1, 1]
    Bipolar,
    /// 无界正实数 [0, ∞)
    Unbounded,
    /// 多维独立光谱（如四维情绪阈值）
    MultiDimensional(Vec<SpectrumDimension>),
    /// 可拆分光谱（子参数各有独立光谱）
    Decomposable(Vec<SubParameter>),
}

/// 光谱维度描述
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectrumDimension {
    pub name: String,
    pub low_label: String,
    pub high_label: String,
    pub spectrum_type: SpectrumType,
}

/// 子参数定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubParameter {
    pub code: String,
    pub name: String,
    pub spectrum: SpectrumType,
}

// ============================================================================
// ParameterDomain: 参数领域
// ============================================================================

/// 参数所属领域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParameterDomain {
    /// A: 信息摄入 —— 世界如何进入这个系统
    InformationIntake,
    /// B: 情绪生成与调节 —— 系统如何生成和调控情感状态
    EmotionGeneration,
    /// C: 动机与价值 —— 什么驱动系统采取行动
    MotivationValue,
    /// D: 行为执行 —— 系统如何将意图转化为行动
    BehaviorExecution,
    /// E: 元认知与自我 —— 系统如何观察和定义自己
    MetacognitionSelf,
    /// F: 社交信号 —— 系统如何发送和接收人际信息
    SocialSignal,
    /// G: 时间性与发展 —— 参数如何随时间变化
    TemporalityDevelopment,
    /// H: 身体-环境耦合 —— 身体与环境如何交互影响
    BodyEnvironmentCoupling,
}

impl fmt::Display for ParameterDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParameterDomain::InformationIntake => write!(f, "信息摄入"),
            ParameterDomain::EmotionGeneration => write!(f, "情绪生成与调节"),
            ParameterDomain::MotivationValue => write!(f, "动机与价值"),
            ParameterDomain::BehaviorExecution => write!(f, "行为执行"),
            ParameterDomain::MetacognitionSelf => write!(f, "元认知与自我"),
            ParameterDomain::SocialSignal => write!(f, "社交信号"),
            ParameterDomain::TemporalityDevelopment => write!(f, "时间性与发展"),
            ParameterDomain::BodyEnvironmentCoupling => write!(f, "身体-环境耦合"),
        }
    }
}

// ============================================================================
// ParameterGranularity: 参数粒度
// ============================================================================

/// 参数粒度 —— 参数是否可进一步拆分
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterGranularity {
    /// 原子级：当前不可拆
    Atomic,
    /// 分子级：存在内部结构但当前作为整体处理
    Molecular,
    /// 可拆：可进一步拆分为子参数
    Decomposable(Vec<String>),
}

// ============================================================================
// Situation: 情境 —— 参数坍缩的语境
// ============================================================================

/// 情境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SituationType {
    /// 安全情境
    Safe,
    /// 威胁情境
    Threat,
    /// 社交情境
    Social,
    /// 独处情境
    Alone,
    /// 压力情境
    Stress,
    /// 疲劳情境
    Fatigue,
    /// 权力情境（拥有权力）
    Power,
    /// 无权情境（失去权力）
    Powerless,
    /// 自定义情境
    Custom(u32),
}

/// 情境 —— 参数取值依赖的语境
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Situation {
    pub situation_type: SituationType,
    pub intensity: f64, // 0.0 ~ 1.0
    pub duration_seconds: f64,
    pub description: String,
}

impl Default for Situation {
    fn default() -> Self {
        Situation {
            situation_type: SituationType::Safe,
            intensity: 0.5,
            duration_seconds: 0.0,
            description: String::new(),
        }
    }
}

// ============================================================================
// Timestamp: 时间戳 —— 参数的时间标定
// ============================================================================

/// 时间戳 —— 参数取值的时间锚点
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    /// Unix 时间戳（毫秒）
    pub unix_ms: i64,
}

impl Timestamp {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        Timestamp { unix_ms: ms }
    }

    pub fn from_ms(ms: i64) -> Self {
        Timestamp { unix_ms: ms }
    }

    /// 两个时间戳之间的天数差
    pub fn days_between(&self, other: &Timestamp) -> f64 {
        (self.unix_ms - other.unix_ms) as f64 / (1000.0 * 60.0 * 60.0 * 24.0)
    }
}

// ============================================================================
// ParameterId: 参数标识符
// ============================================================================

/// 参数唯一标识符（如 A001, B015f, G063）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParameterId {
    /// 领域字母 A-H
    pub domain: char,
    /// 编号 001-084
    pub number: u16,
    /// 子参数标识（如 'a', 'b', 'c'）
    pub sub: Option<char>,
}

impl ParameterId {
    pub fn new(domain: char, number: u16, sub: Option<char>) -> Self {
        ParameterId {
            domain,
            number,
            sub,
        }
    }

    /// 从字符串解析，如 "A001", "B015f", "G063"
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.len() < 4 {
            return None;
        }
        let domain = s.chars().next()?;
        if !domain.is_ascii_uppercase() {
            return None;
        }
        let num_str: String = s[1..].chars().take_while(|c| c.is_ascii_digit()).collect();
        let number: u16 = num_str.parse().ok()?;
        let remaining = &s[1 + num_str.len()..];
        let sub = if remaining.is_empty() {
            None
        } else {
            remaining.chars().next()
        };
        Some(ParameterId { domain, number, sub })
    }

    /// 所属领域
    pub fn domain_enum(&self) -> ParameterDomain {
        match self.domain {
            'A' => ParameterDomain::InformationIntake,
            'B' => ParameterDomain::EmotionGeneration,
            'C' => ParameterDomain::MotivationValue,
            'D' => ParameterDomain::BehaviorExecution,
            'E' => ParameterDomain::MetacognitionSelf,
            'F' => ParameterDomain::SocialSignal,
            'G' => ParameterDomain::TemporalityDevelopment,
            'H' => ParameterDomain::BodyEnvironmentCoupling,
            _ => panic!("Invalid domain: {}", self.domain),
        }
    }
}

impl fmt::Display for ParameterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sub {
            Some(sub) => write!(f, "{}{:03}{}", self.domain, self.number, sub),
            None => write!(f, "{}{:03}", self.domain, self.number),
        }
    }
}

// ============================================================================
// 错误类型
// ============================================================================

/// PAPS 系统错误类型
#[derive(Debug, thiserror::Error)]
pub enum PapsError {
    #[error("参数 {0} 不存在")]
    ParameterNotFound(ParameterId),

    #[error("参数值越界: {0}")]
    ValueOutOfBounds(String),

    #[error("耦合关系无效: {0} 与 {1}")]
    InvalidCoupling(ParameterId, ParameterId),

    #[error("关系不存在: {0}")]
    RelationshipNotFound(String),

    #[error("相变条件不满足: {0}")]
    PhaseChangeConditionNotMet(String),

    #[error("序列化错误: {0}")]
    SerializationError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_value_normalized() {
        let v = ParameterValue::normalized(0.7);
        assert_eq!(v.as_f64(), 0.7);
        let v = ParameterValue::normalized(1.5);
        assert_eq!(v.as_f64(), 1.0);
        let v = ParameterValue::normalized(-0.5);
        assert_eq!(v.as_f64(), 0.0);
    }

    #[test]
    fn test_parameter_value_bipolar() {
        let v = ParameterValue::bipolar(-0.5);
        assert_eq!(v.as_f64(), -0.5);
        let v = ParameterValue::bipolar(2.0);
        assert_eq!(v.as_f64(), 1.0);
    }

    #[test]
    fn test_parameter_id_parse() {
        let id = ParameterId::parse("A001").unwrap();
        assert_eq!(id.domain, 'A');
        assert_eq!(id.number, 1);

        let id = ParameterId::parse("B015f").unwrap();
        assert_eq!(id.domain, 'B');
        assert_eq!(id.number, 15);
        assert_eq!(id.sub, Some('f'));

        assert!(ParameterId::parse("invalid").is_none());
    }

    #[test]
    fn test_parameter_value_serialization() {
        let v = ParameterValue::normalized(0.5);
        let json = serde_json::to_string(&v).unwrap();
        let v2: ParameterValue = serde_json::from_str(&json).unwrap();
        assert!((v.as_f64() - v2.as_f64()).abs() < 1e-10);
    }
}
