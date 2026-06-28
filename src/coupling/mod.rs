//! 参数耦合矩阵 —— 不是"类型"，是"如果…可能…"
//!
//! 定义参数间的高强度耦合关系。这不是"人格类型"——
//! 只是一个参数的值如何影响另一个参数的表现。
//! 任何具体的人都可能位于这些耦合之外的任何位置。

use crate::core::*;
use crate::parameters::{ParameterRegistry, ValueCondition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CouplingMatrix: 耦合矩阵
// ============================================================================

/// 耦合强度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouplingStrength {
    /// 弱耦合：两个参数偶尔相互影响
    Weak,
    /// 中等耦合：两个参数经常相互影响
    Moderate,
    /// 强耦合：两个参数几乎总是相互影响
    Strong,
    /// 锁死耦合：两个参数高度绑定，几乎无法独立变化
    Locked,
}

/// 耦合方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouplingDirection {
    /// A影响B
    AtoB,
    /// B影响A
    BtoA,
    /// 双向影响
    Bidirectional,
}

/// 耦合关系条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouplingEntry {
    /// 参数A的ID
    pub param_a: ParameterId,
    /// 参数A的值条件
    pub condition_a: ValueCondition,
    /// 参数B的ID
    pub param_b: ParameterId,
    /// 参数B的值条件
    pub condition_b: ValueCondition,
    /// 现象学描述
    pub phenomenon: String,
    /// 耦合强度
    pub strength: CouplingStrength,
    /// 耦合方向
    pub direction: CouplingDirection,
}

/// 耦合矩阵 —— 所有参数间耦合关系的完整图谱
#[derive(Debug, Clone)]
pub struct CouplingMatrix {
    /// 所有耦合关系
    entries: Vec<CouplingEntry>,
    /// 按参数索引：param_id -> 涉及该参数的所有耦合条目索引
    index: HashMap<ParameterId, Vec<usize>>,
    /// 耦合对索引：(param_a, param_b) -> entry indices
    pair_index: HashMap<(ParameterId, ParameterId), Vec<usize>>,
}

impl CouplingMatrix {
    /// 从参数注册表构建耦合矩阵
    pub fn build(registry: &ParameterRegistry) -> Self {
        let mut entries = Vec::new();
        let mut index: HashMap<ParameterId, Vec<usize>> = HashMap::new();
        let mut pair_index: HashMap<(ParameterId, ParameterId), Vec<usize>> = HashMap::new();

        for param in registry.iter() {
            for coupling in &param.couplings {
                let entry = CouplingEntry {
                    param_a: param.id.clone(),
                    condition_a: ValueCondition::High,
                    param_b: coupling.parameter.clone(),
                    condition_b: coupling.condition,
                    phenomenon: coupling.phenomenon.clone(),
                    strength: CouplingStrength::Moderate,
                    direction: CouplingDirection::AtoB,
                };

                let idx = entries.len();
                entries.push(entry);

                index.entry(param.id.clone()).or_default().push(idx);
                index.entry(coupling.parameter.clone()).or_default().push(idx);

                // 用数字比较代替字符串比较
                let pair = if (param.id.domain, param.id.number, param.id.sub)
                    < (coupling.parameter.domain, coupling.parameter.number, coupling.parameter.sub)
                {
                    (param.id.clone(), coupling.parameter.clone())
                } else {
                    (coupling.parameter.clone(), param.id.clone())
                };
                pair_index.entry(pair).or_default().push(idx);
            }
        }

        CouplingMatrix {
            entries,
            index,
            pair_index,
        }
    }

    /// 获取涉及某个参数的所有耦合关系
    pub fn get_couplings_for(&self, param_id: &ParameterId) -> Vec<&CouplingEntry> {
        self.index
            .get(param_id)
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    /// 获取两个参数之间的所有耦合关系
    pub fn get_couplings_between(
        &self,
        a: &ParameterId,
        b: &ParameterId,
    ) -> Vec<&CouplingEntry> {
        let pair = if a.to_string() < b.to_string() {
            (a.clone(), b.clone())
        } else {
            (b.clone(), a.clone())
        };
        self.pair_index
            .get(&pair)
            .map(|indices| indices.iter().map(|&i| &self.entries[i]).collect())
            .unwrap_or_default()
    }

    /// 耦合关系总数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 遍历所有耦合关系
    pub fn iter(&self) -> impl Iterator<Item = &CouplingEntry> {
        self.entries.iter()
    }

    /// 分析给定参数值集合的耦合效应
    /// 返回所有被激活的耦合现象描述
    pub fn analyze(
        &self,
        param_values: &HashMap<ParameterId, ParameterValue>,
    ) -> Vec<ActivatedCoupling> {
        let mut activated = Vec::new();

        for entry in &self.entries {
            if let (Some(val_a), Some(val_b)) =
                (param_values.get(&entry.param_a), param_values.get(&entry.param_b))
            {
                let a_matches = Self::check_condition(val_a, &entry.condition_a);
                let b_matches = Self::check_condition(val_b, &entry.condition_b);

                if a_matches && b_matches {
                    activated.push(ActivatedCoupling {
                        param_a: entry.param_a.clone(),
                        param_b: entry.param_b.clone(),
                        phenomenon: entry.phenomenon.clone(),
                        value_a: *val_a,
                        value_b: *val_b,
                    });
                }
            }
        }

        activated
    }

    /// 检查参数值是否满足条件
    fn check_condition(value: &ParameterValue, condition: &ValueCondition) -> bool {
        match condition {
            ValueCondition::High => value.as_f64() > 0.6,
            ValueCondition::Low => value.as_f64() < 0.4,
            ValueCondition::Range(lo, hi) => {
                let v = value.as_f64();
                v >= *lo && v <= *hi
            }
            ValueCondition::DirectionBipolar(target) => {
                if *target > 0.0 {
                    value.as_f64() > 0.3
                } else if *target < 0.0 {
                    value.as_f64() < -0.3
                } else {
                    value.as_f64().abs() < 0.3
                }
            }
        }
    }

    /// 获取耦合统计信息
    pub fn stats(&self) -> CouplingStats {
        let total = self.entries.len();
        let mut strong = 0;
        let mut moderate = 0;
        let mut weak = 0;
        let mut locked = 0;

        for entry in &self.entries {
            match entry.strength {
                CouplingStrength::Strong => strong += 1,
                CouplingStrength::Moderate => moderate += 1,
                CouplingStrength::Weak => weak += 1,
                CouplingStrength::Locked => locked += 1,
            }
        }

        CouplingStats {
            total,
            strong,
            moderate,
            weak,
            locked,
        }
    }
}

/// 激活的耦合关系（引用版，避免 clone）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatedCoupling {
    pub param_a: ParameterId,
    pub param_b: ParameterId,
    pub phenomenon: String,
    pub value_a: ParameterValue,
    pub value_b: ParameterValue,
}

/// 耦合统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingStats {
    pub total: usize,
    pub strong: usize,
    pub moderate: usize,
    pub weak: usize,
    pub locked: usize,
}

// ============================================================================
// CouplingAnalyzer: 耦合分析器
// ============================================================================

/// 耦合分析器 —— 分析参数间的相互作用模式
pub struct CouplingAnalyzer {
    matrix: CouplingMatrix,
}

impl CouplingAnalyzer {
    pub fn new(registry: &ParameterRegistry) -> Self {
        CouplingAnalyzer {
            matrix: CouplingMatrix::build(registry),
        }
    }

    /// 分析参数值集合，返回所有激活的耦合现象
    pub fn analyze(
        &self,
        values: &HashMap<ParameterId, ParameterValue>,
    ) -> Vec<ActivatedCoupling> {
        self.matrix.analyze(values)
    }

    /// 查找与给定参数强耦合的所有参数
    pub fn find_strongly_coupled(&self, param_id: &ParameterId) -> Vec<ParameterId> {
        self.matrix
            .get_couplings_for(param_id)
            .into_iter()
            .filter(|e| e.strength == CouplingStrength::Strong || e.strength == CouplingStrength::Locked)
            .map(|e| {
                if e.param_a == *param_id {
                    e.param_b.clone()
                } else {
                    e.param_a.clone()
                }
            })
            .collect()
    }

    /// 获取耦合矩阵的引用
    pub fn matrix(&self) -> &CouplingMatrix {
        &self.matrix
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling_matrix_build() {
        let registry = ParameterRegistry::new();
        let matrix = CouplingMatrix::build(&registry);
        assert!(matrix.len() > 0, "Coupling matrix should have entries");
    }

    #[test]
    fn test_find_couplings() {
        let registry = ParameterRegistry::new();
        let matrix = CouplingMatrix::build(&registry);
        let couplings = matrix.get_couplings_for(&ParameterId::parse("B015").unwrap());
        assert!(!couplings.is_empty(), "B015 should have couplings");
    }

    #[test]
    fn test_coupling_analysis() {
        let registry = ParameterRegistry::new();
        let matrix = CouplingMatrix::build(&registry);

        let mut values = HashMap::new();
        values.insert(
            ParameterId::parse("A009").unwrap(),
            ParameterValue::normalized(0.8),
        );
        values.insert(
            ParameterId::parse("B015").unwrap(),
            ParameterValue::normalized(0.8),
        );

        let activated = matrix.analyze(&values);
        // A009↑ + B015↑ should activate the "伤害他人后自我折磨" coupling
        assert!(!activated.is_empty());
    }
}
