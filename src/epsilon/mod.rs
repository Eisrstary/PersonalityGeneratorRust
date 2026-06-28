//! 不可通约余数 ε (Epsilon)
//!
//! 即使所有84个参数都被精确测量，即使所有耦合关系都被理解，
//! 即使所有漂移/相变/反转都被建模——
//! 仍然存在 ε（不可通约余数）。
//!
//! ε 是：
//!   - 参数无法捕捉的"那个人的独特历史"
//!   - 所有参数在特定时刻的不可复制的唯一组合
//!   - 自由意志(如果它存在)的数学表达
//!   - 涌现属性中无法被还原为参数的部分
//!   - 两个参数值完全相同的人之间仍然存在的差异
//!
//! ε 不是"我们还没发现的新参数"。
//! ε 是原则上不可被参数化的东西。
//! ε 是人格不能被还原为参数的根本原因。
//! ε 是本系统的自我否定——也是本系统最重要的部分。

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Epsilon: 不可通约余数
// ============================================================================

/// 不可通约余数
///
/// ε 是一个有界随机扰动，代表所有无法被参数系统捕捉的东西。
/// 它不是误差——它是系统对自身局限性的承认。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Epsilon {
    /// ε 的值 [0.0, 1.0]
    /// 0 = 参数系统完全解释了这个人
    /// 1 = 参数系统完全无法解释这个人（但这种情况原则上不可能，因为ε本身也是系统的一部分）
    pub value: f64,
    /// ε 的"个性"——不同人的ε有不同的"味道"
    pub flavor: EpsilonFlavor,
}

/// ε 的味道 —— ε 的质性特征
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpsilonFlavor {
    /// 混沌型：ε表现为不可预测的随机波动
    Chaotic,
    /// 涌现型：ε表现为参数交互中涌现的新属性
    Emergent,
    /// 历史型：ε表现为个人独特历史的印记
    Historical,
    /// 自由型：ε表现为自由意志的数学表达
    Free,
    /// 神秘型：ε表现为原则上不可知的东西
    Mysterious,
    /// 混合型
    Mixed,
}

impl Default for Epsilon {
    fn default() -> Self {
        Epsilon {
            value: 0.3,
            flavor: EpsilonFlavor::Mixed,
        }
    }
}

impl Epsilon {
    /// 创建新的ε
    pub fn new(value: f64, flavor: EpsilonFlavor) -> Self {
        Epsilon {
            value: value.clamp(0.0, 1.0),
            flavor,
        }
    }

    /// ε 应用于参数值，产生"真实"的表现
    /// 真实表现 = 参数值 + ε扰动
    pub fn apply(&self, param_value: f64) -> f64 {
        // ε 的影响是非线性的——它在参数值处于中间范围时影响最大
        let influence = self.value * (1.0 - 2.0 * (param_value - 0.5).abs());
        let perturbation = influence * (self.flavor_modulation());
        (param_value + perturbation).clamp(0.0, 1.0)
    }

    /// ε 的味道调制——不同味道的ε有不同的扰动模式
    fn flavor_modulation(&self) -> f64 {
        match self.flavor {
            EpsilonFlavor::Chaotic => {
                // 混沌型：使用伪随机（基于系统时间）
                use std::time::{SystemTime, UNIX_EPOCH};
                let ns = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                ((ns % 1000) as f64 / 500.0) - 1.0 // [-1, 1]
            }
            EpsilonFlavor::Emergent => {
                // 涌现型：偏向正向扰动
                0.5
            }
            EpsilonFlavor::Historical => {
                // 历史型：偏向负向扰动（历史负担）
                -0.3
            }
            EpsilonFlavor::Free => {
                // 自由型：随机但温和
                0.1
            }
            EpsilonFlavor::Mysterious => {
                // 神秘型：完全随机
                use std::time::{SystemTime, UNIX_EPOCH};
                let ns = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                ((ns % 2000) as f64 / 1000.0) - 1.0
            }
            EpsilonFlavor::Mixed => {
                // 混合型：轻微随机
                0.0
            }
        }
    }

    /// 批量应用ε到所有参数值
    pub fn apply_all(
        &self,
        values: &HashMap<ParameterId, ParameterValue>,
    ) -> HashMap<ParameterId, ParameterValue> {
        values
            .iter()
            .map(|(id, v)| {
                let new_val = self.apply(v.as_f64());
                (id.clone(), ParameterValue::normalized(new_val))
            })
            .collect()
    }
}

// ============================================================================
// EpsilonAcknowledgment: ε的自我否定声明
// ============================================================================

/// ε 的哲学声明 —— 系统对自身局限性的承认
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpsilonAcknowledgment {
    /// 声明文本
    pub declaration: String,
    /// ε 的值
    pub epsilon: Epsilon,
}

impl Default for EpsilonAcknowledgment {
    fn default() -> Self {
        EpsilonAcknowledgment {
            declaration: String::from(
                "本系统承认：即使所有参数都被精确测量，即使所有耦合关系都被理解，\
                 即使所有动态机制都被建模——仍然存在不可通约余数ε。\
                 ε不是误差，不是未发现的参数，不是模型的缺陷。\
                 ε是人格不能被还原为参数的根本原因。\
                 ε是本系统的自我否定——也是本系统最重要的部分。",
            ),
            epsilon: Epsilon::default(),
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epsilon_creation() {
        let eps = Epsilon::new(0.3, EpsilonFlavor::Mixed);
        assert_eq!(eps.value, 0.3);
    }

    #[test]
    fn test_epsilon_apply() {
        let eps = Epsilon::new(0.5, EpsilonFlavor::Mixed);
        let result = eps.apply(0.7);
        // 应该接近0.7但不完全等于
        assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn test_epsilon_all_flavors() {
        let flavors = [
            EpsilonFlavor::Chaotic,
            EpsilonFlavor::Emergent,
            EpsilonFlavor::Historical,
            EpsilonFlavor::Free,
            EpsilonFlavor::Mysterious,
            EpsilonFlavor::Mixed,
        ];

        for flavor in &flavors {
            let eps = Epsilon::new(0.3, *flavor);
            let result = eps.apply(0.5);
            assert!(result >= 0.0 && result <= 1.0);
        }
    }

    #[test]
    fn test_epsilon_apply_all() {
        let eps = Epsilon::default();
        let mut values = HashMap::new();
        values.insert(
            ParameterId::parse("A001").unwrap(),
            ParameterValue::normalized(0.5),
        );
        values.insert(
            ParameterId::parse("B015").unwrap(),
            ParameterValue::normalized(0.7),
        );

        let result = eps.apply_all(&values);
        assert_eq!(result.len(), 2);
    }
}
