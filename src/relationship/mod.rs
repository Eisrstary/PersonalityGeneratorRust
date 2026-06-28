//! 关系中的参数坍缩 —— 参数如何在具体关系中取值
//!
//! 一个参数不是"一个值"——它在不同关系中取不同值。
//! 这不是"虚伪"——这是参数的关系依赖性。所有参数都有此属性。
//!
//! Parameter(relationship) = Baseline × RelationModifier(relationship_type)

use crate::core::*;
use crate::parameters::ParameterRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Relationship: 关系定义
// ============================================================================

/// 关系类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// 亲密关系（家人/伴侣/挚友）
    Intimate,
    /// 熟人关系
    Acquaintance,
    /// 陌生关系
    Stranger,
    /// 敌对关系
    Hostile,
    /// 权力不对等（高位）
    PowerSuperior,
    /// 权力不对等（低位）
    PowerSubordinate,
    /// 依赖关系（依赖对方）
    Dependent,
    /// 被依赖关系（对方依赖我）
    DependedUpon,
    /// 自定义
    Custom(u32),
}

/// 关系历史事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipEvent {
    TrustBuilt { timestamp: Timestamp },
    TrustBetrayed { timestamp: Timestamp },
    HarmInflicted { timestamp: Timestamp, severity: f64 },
    HarmReceived { timestamp: Timestamp, severity: f64 },
    ForgivenessGiven { timestamp: Timestamp },
    ForgivenessReceived { timestamp: Timestamp },
    Repair { timestamp: Timestamp },
    Custom { timestamp: Timestamp, description: String },
}

/// 关系定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// 关系ID
    pub id: String,
    /// 关系类型
    pub relationship_type: RelationshipType,
    /// 关系建立时间
    pub established: Timestamp,
    /// 关系历史事件
    pub history: Vec<RelationshipEvent>,
    /// 关系预期持续时间（天，0=未知）
    pub expected_duration_days: f64,
    /// 是否有求于对方
    pub needs_from_other: f64, // 0.0 ~ 1.0
    /// 是否感受到对方的威胁
    pub threat_from_other: f64, // 0.0 ~ 1.0
    /// 关系亲密度
    pub closeness: f64, // 0.0 ~ 1.0
}

impl Relationship {
    pub fn new(id: String, relationship_type: RelationshipType) -> Self {
        Relationship {
            id,
            relationship_type,
            established: Timestamp::now(),
            history: Vec::new(),
            expected_duration_days: 0.0,
            needs_from_other: 0.0,
            threat_from_other: 0.0,
            closeness: 0.0,
        }
    }

    /// 计算关系修饰因子
    /// 这个因子决定了参数基线值在特定关系中的坍缩程度
    pub fn modifier(&self) -> f64 {
        let mut modifier = 1.0;

        // 关系类型的基础修饰
        match self.relationship_type {
            RelationshipType::Intimate => modifier *= 1.3,
            RelationshipType::Acquaintance => modifier *= 0.9,
            RelationshipType::Stranger => modifier *= 0.5,
            RelationshipType::Hostile => modifier *= 0.3,
            RelationshipType::PowerSuperior => modifier *= 0.8,
            RelationshipType::PowerSubordinate => modifier *= 0.7,
            RelationshipType::Dependent => modifier *= 1.1,
            RelationshipType::DependedUpon => modifier *= 0.9,
            RelationshipType::Custom(_) => {}
        }

        // 亲密度影响
        modifier *= 1.0 + self.closeness * 0.5;

        // 威胁感知降低修饰
        modifier *= 1.0 - self.threat_from_other * 0.3;

        // 需求增加修饰
        modifier *= 1.0 + self.needs_from_other * 0.2;

        // 历史事件影响
        for event in &self.history {
            match event {
                RelationshipEvent::TrustBetrayed { .. } => modifier *= 0.5,
                RelationshipEvent::HarmReceived { severity, .. } => {
                    modifier *= 1.0 - severity * 0.3;
                }
                RelationshipEvent::ForgivenessReceived { .. } => modifier *= 1.2,
                RelationshipEvent::Repair { .. } => modifier *= 1.1,
                _ => {}
            }
        }

        modifier.clamp(0.1, 2.0)
    }
}

// ============================================================================
// RelationshipContext: 关系语境中的参数取值
// ============================================================================

/// 关系语境 —— 参数在特定关系中的坍缩值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipContext {
    /// 关系
    pub relationship: Relationship,
    /// 参数在该关系中的取值映射
    pub parameter_values: HashMap<ParameterId, ParameterValue>,
}

// ============================================================================
// CollapseFunction: 坍缩函数
// ============================================================================

/// 关系坍缩函数
/// Parameter(relationship) = Baseline × RelationModifier(relationship_type)
pub struct CollapseFunction {
    registry: ParameterRegistry,
}

/// 跨关系方差条目
pub type CrossRelationalEntry = (ParameterId, f64, Vec<(String, f64)>);

impl CollapseFunction {
    pub fn new(registry: &ParameterRegistry) -> Self {
        CollapseFunction {
            registry: registry.clone(),
        }
    }

    /// 计算参数在特定关系中的坍缩值
    pub fn collapse(
        &self,
        param_id: &ParameterId,
        baseline: ParameterValue,
        relationship: &Relationship,
    ) -> ParameterValue {
        let modifier = relationship.modifier();
        let raw_value = baseline.as_f64() * modifier;

        // 根据参数类型钳制
        if let Some(param_def) = self.registry.get(param_id) {
            match &param_def.spectrum {
                SpectrumType::Normalized => ParameterValue::normalized(raw_value),
                SpectrumType::Bipolar => ParameterValue::bipolar(raw_value),
                SpectrumType::Unbounded => ParameterValue::unbounded(raw_value),
                _ => ParameterValue::normalized(raw_value.clamp(0.0, 1.0)),
            }
        } else {
            ParameterValue::normalized(raw_value.clamp(0.0, 1.0))
        }
    }

    /// 批量计算所有参数在特定关系中的坍缩值
    pub fn collapse_all(
        &self,
        baselines: &HashMap<ParameterId, ParameterValue>,
        relationship: &Relationship,
    ) -> HashMap<ParameterId, ParameterValue> {
        let modifier = relationship.modifier();

        baselines
            .iter()
            .map(|(id, baseline)| {
                let raw = baseline.as_f64() * modifier;
                let value = if let Some(param_def) = self.registry.get(id) {
                    match &param_def.spectrum {
                        SpectrumType::Normalized => ParameterValue::normalized(raw),
                        SpectrumType::Bipolar => ParameterValue::bipolar(raw),
                        SpectrumType::Unbounded => ParameterValue::unbounded(raw),
                        _ => ParameterValue::normalized(raw.clamp(0.0, 1.0)),
                    }
                } else {
                    ParameterValue::normalized(raw.clamp(0.0, 1.0))
                };
                (id.clone(), value)
            })
            .collect()
    }

    /// 分析参数在不同关系中的取值差异
    /// 返回差异最大的前N个参数
    pub fn analyze_cross_relational_variance(
        &self,
        baselines: &HashMap<ParameterId, ParameterValue>,
        relationships: &[Relationship],
        top_n: usize,
    ) -> Vec<CrossRelationalEntry> {
        let mut variances: Vec<CrossRelationalEntry> = Vec::new();

        for (param_id, baseline) in baselines {
            let mut values: Vec<(String, f64)> = relationships
                .iter()
                .map(|rel| {
                    let collapsed = self.collapse(param_id, *baseline, rel);
                    (rel.id.clone(), collapsed.as_f64())
                })
                .collect();

            if values.len() >= 2 {
                let min = values.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
                let max = values.iter().map(|(_, v)| *v).fold(f64::NEG_INFINITY, f64::max);
                let variance = max - min;

                values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                if variance > 0.01 {
                    variances.push((param_id.clone(), variance, values));
                }
            }
        }

        variances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        variances.truncate(top_n);
        variances
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_modifier() {
        let rel = Relationship::new("test".into(), RelationshipType::Intimate);
        let modifier = rel.modifier();
        assert!(modifier > 1.0, "Intimate relationship should amplify");

        let rel2 = Relationship::new("test2".into(), RelationshipType::Hostile);
        let modifier2 = rel2.modifier();
        assert!(modifier2 < 1.0, "Hostile relationship should attenuate");
    }

    #[test]
    fn test_collapse_function() {
        let registry = ParameterRegistry::new();
        let cf = CollapseFunction::new(&registry);

        let baseline = ParameterValue::normalized(0.7);
        let rel = Relationship::new("intimate".into(), RelationshipType::Intimate);
        let collapsed = cf.collapse(&ParameterId::parse("B015").unwrap(), baseline, &rel);

        // 亲密关系中内疚感应该被放大
        assert!(collapsed.as_f64() > baseline.as_f64());
    }

    #[test]
    fn test_cross_relational_variance() {
        let registry = ParameterRegistry::new();
        let cf = CollapseFunction::new(&registry);

        let mut baselines = HashMap::new();
        baselines.insert(
            ParameterId::parse("B015").unwrap(),
            ParameterValue::normalized(0.7),
        );

        let relationships = vec![
            Relationship::new("intimate".into(), RelationshipType::Intimate),
            Relationship::new("hostile".into(), RelationshipType::Hostile),
            Relationship::new("stranger".into(), RelationshipType::Stranger),
        ];

        let variances = cf.analyze_cross_relational_variance(&baselines, &relationships, 5);
        assert!(!variances.is_empty());
    }
}
