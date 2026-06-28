//! API 层 —— 对外暴露的统一接口
//!
//! 无论以DLL、WASM还是独立应用形式出现，
//! 对外部都是通过这个API层进行调用。

use crate::core::*;
use crate::coupling::CouplingAnalyzer;
use crate::dynamics::{DynamicSystem, PhaseChangeType};
use crate::epsilon::{Epsilon, EpsilonAcknowledgment, EpsilonFlavor};
use crate::parameters::ParameterRegistry;
use crate::relationship::{CollapseFunction, Relationship, RelationshipType};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PersonalitySystem: 人格系统总成
// ============================================================================

/// 人格原子参数系统 —— 顶层API
///
/// 这是外部调用者使用的唯一入口点。
/// 封装了参数注册表、耦合分析器、动态系统、关系坍缩和ε。
pub struct PersonalitySystem {
    /// 参数注册表
    registry: ParameterRegistry,
    /// 耦合分析器
    coupling_analyzer: CouplingAnalyzer,
    /// 动态系统
    dynamic_system: DynamicSystem,
    /// 关系坍缩函数
    collapse_function: CollapseFunction,
    /// 不可通约余数
    epsilon: Epsilon,
    /// 所有关系
    relationships: HashMap<String, Relationship>,
}

impl PersonalitySystem {
    /// 创建新的人格系统实例
    pub fn new() -> Self {
        let registry = ParameterRegistry::new();
        let coupling_analyzer = CouplingAnalyzer::new(&registry);
        let dynamic_system = DynamicSystem::new(&registry);
        let collapse_function = CollapseFunction::new(&registry);
        let epsilon = Epsilon::default();

        PersonalitySystem {
            registry,
            coupling_analyzer,
            dynamic_system,
            collapse_function,
            epsilon,
            relationships: HashMap::new(),
        }
    }

    /// 使用自定义ε创建
    pub fn with_epsilon(epsilon: Epsilon) -> Self {
        let mut system = Self::new();
        system.epsilon = epsilon;
        system
    }

    // ===== 参数查询 =====

    /// 获取参数总数
    pub fn parameter_count(&self) -> usize {
        self.registry.len()
    }

    /// 获取参数定义
    pub fn get_parameter(&self, id: &str) -> Option<&crate::parameters::ParameterDefinition> {
        self.registry.get_by_str(id)
    }

    /// 获取所有参数ID
    pub fn all_parameter_ids(&self) -> Vec<String> {
        self.registry.all_ids().iter().map(|id| id.to_string()).collect()
    }

    /// 获取某个领域的所有参数
    pub fn get_domain_parameters(&self, domain: ParameterDomain) -> Vec<String> {
        self.registry
            .get_domain_ids(&domain)
            .iter()
            .map(|id| id.to_string())
            .collect()
    }

    // ===== 参数值管理 =====

    /// 获取参数的当前值
    pub fn get_value(&self, param_id: &str) -> Option<f64> {
        let id = ParameterId::parse(param_id)?;
        self.dynamic_system.drift_engine.get_value(&id).map(|v| v.as_f64())
    }

    /// 设置参数的初始值
    pub fn set_value(&mut self, param_id: &str, value: f64) -> Result<(), PapsError> {
        let id = ParameterId::parse(param_id).ok_or_else(|| {
            PapsError::ParameterNotFound(ParameterId::parse("UNKNOWN").unwrap())
        })?;
        if !self.registry.contains(&id) {
            return Err(PapsError::ParameterNotFound(id));
        }
        let param_def = self.registry.get(&id).unwrap();
        let pv = match &param_def.spectrum {
            SpectrumType::Normalized => ParameterValue::normalized(value),
            SpectrumType::Bipolar => ParameterValue::bipolar(value),
            SpectrumType::Unbounded => ParameterValue::unbounded(value),
            _ => ParameterValue::normalized(value),
        };
        self.dynamic_system.drift_engine.set_initial_value(&id, pv);
        Ok(())
    }

    /// 批量设置参数值
    pub fn set_values(&mut self, values: &HashMap<String, f64>) -> Result<(), PapsError> {
        for (param_id, value) in values {
            self.set_value(param_id, *value)?;
        }
        Ok(())
    }

    // ===== 倾向设置（Tendency-based setting）=====

    /// 按倾向设置参数值 —— 在合理范围内随机生成
    ///
    /// tendency:
    ///   "very_low" | "low" | "medium" | "high" | "very_high"
    ///   对 bipolar 参数额外支持: "negative" | "positive"
    pub fn set_tendency(&mut self, param_id: &str, tendency: &str) -> Result<f64, PapsError> {
        use rand::Rng;

        let id = ParameterId::parse(param_id).ok_or_else(|| {
            PapsError::ParameterNotFound(ParameterId::parse("UNKNOWN").unwrap())
        })?;
        let param_def = self.registry.get(&id).ok_or_else(|| {
            PapsError::ParameterNotFound(id.clone())
        })?;

        let mut rng = rand::thread_rng();
        let value = match &param_def.spectrum {
            SpectrumType::Normalized => {
                let range = match tendency {
                    "very_low" => (0.0, 0.2),
                    "low" => (0.1, 0.4),
                    "medium" => (0.35, 0.65),
                    "high" => (0.6, 0.9),
                    "very_high" => (0.8, 1.0),
                    _ => (0.0, 1.0),
                };
                rng.gen_range(range.0..range.1)
            }
            SpectrumType::Bipolar => {
                let range = match tendency {
                    "very_negative" | "very_low" => (-1.0, -0.6),
                    "negative" | "low" => (-0.7, -0.2),
                    "neutral" | "medium" => (-0.3, 0.3),
                    "positive" | "high" => (0.2, 0.7),
                    "very_positive" | "very_high" => (0.6, 1.0),
                    _ => (-1.0, 1.0),
                };
                rng.gen_range(range.0..range.1)
            }
            SpectrumType::Unbounded => {
                // 无界参数：用参数默认值作为参考锚点
                let anchor = param_def.default_value.as_f64().max(1.0);
                let range = match tendency {
                    "very_low" => (anchor * 0.01, anchor * 0.3),
                    "low" => (anchor * 0.1, anchor * 0.6),
                    "medium" => (anchor * 0.4, anchor * 1.6),
                    "high" => (anchor * 1.5, anchor * 4.0),
                    "very_high" => (anchor * 3.0, anchor * 10.0),
                    _ => (anchor * 0.1, anchor * 2.0),
                };
                rng.gen_range(range.0..range.1)
            }
            _ => rng.gen_range(0.0..1.0),
        };

        self.set_value(param_id, value)?;
        Ok(value)
    }

    /// 批量按倾向设置参数
    ///
    /// tendencies: HashMap<param_id, tendency_string>
    /// 返回: HashMap<param_id, generated_value>
    pub fn set_tendencies(
        &mut self,
        tendencies: &HashMap<String, String>,
    ) -> Result<HashMap<String, f64>, PapsError> {
        let mut results = HashMap::new();
        for (param_id, tendency) in tendencies {
            let value = self.set_tendency(param_id, tendency)?;
            results.insert(param_id.clone(), value);
        }
        Ok(results)
    }

    /// 获取参数的光谱类型和有效范围（用于外部了解参数约束）
    pub fn get_parameter_range(&self, param_id: &str) -> Option<ParameterRange> {
        let id = ParameterId::parse(param_id)?;
        let def = self.registry.get(&id)?;
        Some(match &def.spectrum {
            SpectrumType::Normalized => ParameterRange {
                param_id: id.to_string(),
                param_name: def.name.clone(),
                kind: "normalized".into(),
                min: 0.0,
                max: 1.0,
                default: def.default_value.as_f64(),
                supported_tendencies: vec![
                    "very_low".into(), "low".into(), "medium".into(),
                    "high".into(), "very_high".into(),
                ],
            },
            SpectrumType::Bipolar => ParameterRange {
                param_id: id.to_string(),
                param_name: def.name.clone(),
                kind: "bipolar".into(),
                min: -1.0,
                max: 1.0,
                default: def.default_value.as_f64(),
                supported_tendencies: vec![
                    "very_negative".into(), "negative".into(), "neutral".into(),
                    "positive".into(), "very_positive".into(),
                ],
            },
            SpectrumType::Unbounded => ParameterRange {
                param_id: id.to_string(),
                param_name: def.name.clone(),
                kind: "unbounded".into(),
                min: 0.0,
                max: f64::INFINITY,
                default: def.default_value.as_f64(),
                supported_tendencies: vec![
                    "very_low".into(), "low".into(), "medium".into(),
                    "high".into(), "very_high".into(),
                ],
            },
            _ => ParameterRange {
                param_id: id.to_string(),
                param_name: def.name.clone(),
                kind: "normalized".into(),
                min: 0.0,
                max: 1.0,
                default: 0.5,
                supported_tendencies: vec!["medium".into()],
            },
        })
    }

    /// 获取所有参数的有效范围
    pub fn get_all_parameter_ranges(&self) -> Vec<ParameterRange> {
        self.registry
            .all_ids()
            .iter()
            .filter_map(|id| self.get_parameter_range(&id.to_string()))
            .collect()
    }

    /// 获取所有当前参数值
    pub fn get_all_values(&self) -> HashMap<String, f64> {
        self.dynamic_system
            .get_all_values()
            .into_iter()
            .map(|(id, v)| (id.to_string(), v.as_f64()))
            .collect()
    }

    // ===== 耦合分析 =====

    /// 分析当前参数值的耦合效应
    pub fn analyze_couplings(&self) -> Vec<CouplingAnalysisResult> {
        let values = self.dynamic_system.get_all_values();
        self.coupling_analyzer
            .analyze(&values)
            .into_iter()
            .map(|ac| CouplingAnalysisResult {
                param_a: ac.entry.param_a.to_string(),
                param_b: ac.entry.param_b.to_string(),
                phenomenon: ac.entry.phenomenon.clone(),
                value_a: ac.value_a.as_f64(),
                value_b: ac.value_b.as_f64(),
            })
            .collect()
    }

    /// 查找与指定参数强耦合的参数
    pub fn find_strongly_coupled(&self, param_id: &str) -> Vec<String> {
        let id = match ParameterId::parse(param_id) {
            Some(id) => id,
            None => return Vec::new(),
        };
        self.coupling_analyzer
            .find_strongly_coupled(&id)
            .into_iter()
            .map(|id| id.to_string())
            .collect()
    }

    // ===== 动态系统 =====

    /// 设置参数漂移速率
    pub fn set_drift_rate(&mut self, param_id: &str, rate: f64) -> Result<(), PapsError> {
        let id = ParameterId::parse(param_id).ok_or_else(|| {
            PapsError::ParameterNotFound(ParameterId::parse("UNKNOWN").unwrap())
        })?;
        self.dynamic_system.drift_engine.set_drift_rate(&id, rate);
        Ok(())
    }

    /// 推进时间（应用漂移）
    pub fn advance_time(&mut self, days: f64) -> HashMap<String, f64> {
        let now = Timestamp::from_ms(
            Timestamp::now().unix_ms + (days * 24.0 * 3600.0 * 1000.0) as i64,
        );
        self.dynamic_system
            .drift_engine
            .apply_all_drifts(&now)
            .into_iter()
            .map(|(id, drift)| (id.to_string(), drift))
            .collect()
    }

    /// 触发相变事件
    pub fn trigger_phase_change(&mut self, event_type: &str) -> Vec<(String, f64)> {
        let pt = match event_type {
            "betrayal" => PhaseChangeType::Betrayal,
            "loss" => PhaseChangeType::Loss,
            "humiliation" => PhaseChangeType::Humiliation,
            "power_gain" => PhaseChangeType::PowerGain,
            "power_loss" => PhaseChangeType::PowerLoss,
            "forgiveness" => PhaseChangeType::Forgiveness,
            "mission_failure" => PhaseChangeType::MissionFailure,
            _ => PhaseChangeType::Custom(0),
        };

        let current_values = self.dynamic_system.get_all_values();
        self.dynamic_system
            .phase_engine
            .trigger(pt, &current_values)
            .into_iter()
            .map(|(id, v)| {
                // Apply the phase change to the drift engine
                self.dynamic_system.drift_engine.set_initial_value(&id, v);
                (id.to_string(), v.as_f64())
            })
            .collect()
    }

    /// 检查反转状态
    pub fn is_reversed(&self, param_id: &str) -> bool {
        let id = match ParameterId::parse(param_id) {
            Some(id) => id,
            None => return false,
        };
        self.dynamic_system.reversal_engine.is_reversed(&id)
    }

    // ===== 关系管理 =====

    /// 添加关系
    pub fn add_relationship(&mut self, id: &str, rel_type: &str) {
        let rt = match rel_type {
            "intimate" => RelationshipType::Intimate,
            "acquaintance" => RelationshipType::Acquaintance,
            "stranger" => RelationshipType::Stranger,
            "hostile" => RelationshipType::Hostile,
            "superior" => RelationshipType::PowerSuperior,
            "subordinate" => RelationshipType::PowerSubordinate,
            _ => RelationshipType::Stranger,
        };
        self.relationships
            .insert(id.to_string(), Relationship::new(id.to_string(), rt));
    }

    /// 计算参数在特定关系中的坍缩值
    pub fn collapse_in_relationship(
        &self,
        param_id: &str,
        relationship_id: &str,
    ) -> Option<f64> {
        let id = ParameterId::parse(param_id)?;
        let rel = self.relationships.get(relationship_id)?;
        let baseline = self.dynamic_system.drift_engine.get_value(&id)?;
        let collapsed = self.collapse_function.collapse(&id, baseline, rel);
        Some(collapsed.as_f64())
    }

    /// 获取所有关系中参数值的差异分析
    pub fn cross_relational_analysis(&self, top_n: usize) -> Vec<CrossRelationalVariance> {
        let baselines = self.dynamic_system.get_all_values();
        let rels: Vec<Relationship> = self.relationships.values().cloned().collect();

        self.collapse_function
            .analyze_cross_relational_variance(&baselines, &rels, top_n)
            .into_iter()
            .map(|(id, variance, values)| CrossRelationalVariance {
                param_id: id.to_string(),
                variance,
                values: values.into_iter().collect(),
            })
            .collect()
    }

    // ===== ε =====

    /// 获取ε的值
    pub fn epsilon_value(&self) -> f64 {
        self.epsilon.value
    }

    /// 设置ε
    pub fn set_epsilon(&mut self, value: f64, flavor: &str) {
        let f = match flavor {
            "chaotic" => EpsilonFlavor::Chaotic,
            "emergent" => EpsilonFlavor::Emergent,
            "historical" => EpsilonFlavor::Historical,
            "free" => EpsilonFlavor::Free,
            "mysterious" => EpsilonFlavor::Mysterious,
            _ => EpsilonFlavor::Mixed,
        };
        self.epsilon = Epsilon::new(value, f);
    }

    /// 应用ε到所有参数值
    pub fn apply_epsilon(&self) -> HashMap<String, f64> {
        let values = self.dynamic_system.get_all_values();
        self.epsilon
            .apply_all(&values)
            .into_iter()
            .map(|(id, v)| (id.to_string(), v.as_f64()))
            .collect()
    }

    /// 获取ε的哲学声明
    pub fn epsilon_acknowledgment(&self) -> String {
        EpsilonAcknowledgment::default().declaration
    }

    // ===== 序列化 =====

    /// 导出整个系统状态为JSON
    pub fn export_state(&self) -> Result<String, PapsError> {
        let state = SystemState {
            parameter_values: self.get_all_values(),
            epsilon: self.epsilon,
            relationships: self.relationships.clone(),
        };
        serde_json::to_string_pretty(&state)
            .map_err(|e| PapsError::SerializationError(e.to_string()))
    }

    /// 从JSON导入系统状态
    pub fn import_state(&mut self, json: &str) -> Result<(), PapsError> {
        let state: SystemState = serde_json::from_str(json)
            .map_err(|e| PapsError::SerializationError(e.to_string()))?;

        for (param_id, value) in &state.parameter_values {
            self.set_value(param_id, *value)?;
        }
        self.epsilon = state.epsilon;
        self.relationships = state.relationships;

        Ok(())
    }

    /// 获取系统信息
    pub fn system_info(&self) -> SystemInfo {
        SystemInfo {
            version: crate::VERSION.to_string(),
            total_parameters: self.registry.len(),
            total_domains: 8,
            coupling_count: self.coupling_analyzer.matrix().len(),
            relationship_count: self.relationships.len(),
            epsilon_value: self.epsilon.value,
        }
    }
}

impl Default for PersonalitySystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// API 数据结构
// ============================================================================

/// 耦合分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingAnalysisResult {
    pub param_a: String,
    pub param_b: String,
    pub phenomenon: String,
    pub value_a: f64,
    pub value_b: f64,
}

/// 跨关系方差
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRelationalVariance {
    pub param_id: String,
    pub variance: f64,
    pub values: Vec<(String, f64)>,
}

/// 系统状态（用于序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub parameter_values: HashMap<String, f64>,
    pub epsilon: Epsilon,
    pub relationships: HashMap<String, Relationship>,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub total_parameters: usize,
    pub total_domains: usize,
    pub coupling_count: usize,
    pub relationship_count: usize,
    pub epsilon_value: f64,
}

/// 参数范围信息（用于外部了解参数约束和可用倾向）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRange {
    pub param_id: String,
    pub param_name: String,
    /// "normalized" | "bipolar" | "unbounded"
    pub kind: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub supported_tendencies: Vec<String>,
}

// ============================================================================
// PersonalityProfile: 统一人格档案 —— 唯一的数据真相源
// ============================================================================

/// 参数条目（含元信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterEntry {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub domain_label: String,
    pub value: f64,
    pub spectrum_type: String,
    pub definition: String,
}

/// 关系坍缩条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseEntry {
    pub param_id: String,
    pub param_name: String,
    pub baseline: f64,
    pub values: HashMap<String, f64>,
}

/// 统一人格档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    /// 系统版本
    pub version: String,
    /// 生成时间戳
    pub generated_at: String,
    /// ε 值
    pub epsilon: f64,
    /// ε 哲学声明
    pub epsilon_declaration: String,
    /// 所有参数及其当前值
    pub parameters: Vec<ParameterEntry>,
    /// 激活的耦合现象
    pub couplings: Vec<CouplingAnalysisResult>,
    /// 关系坍缩表
    pub collapse_table: Vec<CollapseEntry>,
    /// 跨关系方差 Top N
    pub cross_relational_variance: Vec<CrossRelationalVariance>,
}

impl PersonalitySystem {
    /// 导出统一人格档案 —— 这是所有输出格式的唯一数据源
    pub fn export_profile(&self, collapse_param_ids: &[&str]) -> PersonalityProfile {
        let mut parameters = Vec::with_capacity(84);

        for id in self.registry.all_ids() {
            let def = self.registry.get(id).unwrap();
            let value = self
                .dynamic_system
                .drift_engine
                .get_value(id)
                .map(|v| v.as_f64())
                .unwrap_or(0.0);

            let spectrum_type = match &def.spectrum {
                SpectrumType::Normalized => "normalized".to_string(),
                SpectrumType::Bipolar => "bipolar".to_string(),
                SpectrumType::Unbounded => "unbounded".to_string(),
                SpectrumType::MultiDimensional(_) => "multi_dimensional".to_string(),
                SpectrumType::Decomposable(_) => "decomposable".to_string(),
            };

            parameters.push(ParameterEntry {
                id: id.to_string(),
                name: def.name.clone(),
                domain: format!("{:?}", def.domain),
                domain_label: def.domain.to_string(),
                value,
                spectrum_type,
                definition: def.definition.clone(),
            });
        }

        // 耦合分析
        let couplings = self.analyze_couplings();

        // 关系坍缩
        let rels: Vec<Relationship> = self.relationships.values().cloned().collect();
        let mut collapse_table = Vec::new();
        for pid_str in collapse_param_ids {
            if let Some(id) = ParameterId::parse(pid_str) {
                if let Some(baseline) = self.dynamic_system.drift_engine.get_value(&id) {
                    let name = self
                        .registry
                        .get(&id)
                        .map(|d| d.name.clone())
                        .unwrap_or_default();
                    let mut values = HashMap::new();
                    for rel in &rels {
                        let collapsed = self.collapse_function.collapse(&id, baseline, rel);
                        values.insert(rel.id.clone(), collapsed.as_f64());
                    }
                    collapse_table.push(CollapseEntry {
                        param_id: id.to_string(),
                        param_name: name,
                        baseline: baseline.as_f64(),
                        values,
                    });
                }
            }
        }

        // 跨关系方差
        let baselines = self.dynamic_system.get_all_values();
        let cross_relational_variance: Vec<CrossRelationalVariance> = self
            .collapse_function
            .analyze_cross_relational_variance(&baselines, &rels, 5)
            .into_iter()
            .map(|(id, variance, values)| CrossRelationalVariance {
                param_id: id.to_string(),
                variance,
                values,
            })
            .collect();

        let now = chrono::Utc::now().to_rfc3339();

        PersonalityProfile {
            version: crate::VERSION.to_string(),
            generated_at: now,
            epsilon: self.epsilon.value,
            epsilon_declaration: EpsilonAcknowledgment::default().declaration,
            parameters,
            couplings,
            collapse_table,
            cross_relational_variance,
        }
    }

    /// 导出原始 JSON（完整数据，给后端/存储）
    pub fn export_profile_json(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);
        serde_json::to_string_pretty(&profile).unwrap_or_default()
    }

    /// 导出 AI 可读 JSON（紧凑，为角色扮演 AI 优化）
    pub fn export_profile_ai_json(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);

        #[derive(Serialize)]
        struct AiProfile {
            version: String,
            epsilon: f64,
            #[serde(rename = "param")]
            parameters: HashMap<String, f64>,
            couplings: Vec<AiCoupling>,
            #[serde(rename = "relational_collapse")]
            collapse: Vec<AiCollapse>,
            #[serde(rename = "top_variances")]
            variances: Vec<AiVariance>,
        }

        #[derive(Serialize)]
        struct AiCoupling {
            a: String,
            b: String,
            desc: String,
        }

        #[derive(Serialize)]
        struct AiCollapse {
            id: String,
            name: String,
            baseline: f64,
            #[serde(rename = "by_relationship")]
            by_rel: HashMap<String, f64>,
        }

        #[derive(Serialize)]
        struct AiVariance {
            id: String,
            variance: f64,
            #[serde(rename = "by_relationship")]
            by_rel: HashMap<String, f64>,
        }

        let parameters: HashMap<String, f64> = profile
            .parameters
            .iter()
            .map(|p| (p.id.clone(), p.value))
            .collect();

        let couplings: Vec<AiCoupling> = profile
            .couplings
            .iter()
            .map(|c| AiCoupling {
                a: c.param_a.clone(),
                b: c.param_b.clone(),
                desc: c.phenomenon.clone(),
            })
            .collect();

        let collapse: Vec<AiCollapse> = profile
            .collapse_table
            .iter()
            .map(|c| AiCollapse {
                id: c.param_id.clone(),
                name: c.param_name.clone(),
                baseline: c.baseline,
                by_rel: c.values.clone(),
            })
            .collect();

        let variances: Vec<AiVariance> = profile
            .cross_relational_variance
            .iter()
            .map(|v| {
                let by_rel: HashMap<String, f64> =
                    v.values.iter().map(|(rid, val)| (rid.clone(), *val)).collect();
                AiVariance {
                    id: v.param_id.clone(),
                    variance: v.variance,
                    by_rel,
                }
            })
            .collect();

        let ai = AiProfile {
            version: profile.version,
            epsilon: profile.epsilon,
            parameters,
            couplings,
            collapse,
            variances,
        };

        serde_json::to_string(&ai).unwrap_or_default()
    }

    /// 导出人类可读文本
    pub fn export_profile_text(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);
        let mut out = String::new();

        // 标题
        out.push_str(&format!(
            "人格原子参数档案 (PAPS v{})\n生成时间: {}\n\n",
            profile.version, profile.generated_at
        ));
        out.push_str("注意: 这不是人格类型，这是84个参数在此时此刻的取值快照。\n");
        out.push_str("这些值会在关系中坍缩、在时间里漂移、在情境中撕裂。\n");
        out.push_str(&"=".repeat(72));
        out.push_str("\n\n");

        // 按领域分组输出参数
        let domains = [
            ("A", "信息摄入 —— 世界如何进入这个系统"),
            ("B", "情绪生成与调节 —— 系统如何生成和调控情感状态"),
            ("C", "动机与价值 —— 什么驱动系统采取行动"),
            ("D", "行为执行 —— 系统如何将意图转化为行动"),
            ("E", "元认知与自我 —— 系统如何观察和定义自己"),
            ("F", "社交信号 —— 系统如何发送和接收人际信息"),
            ("G", "时间性与发展 —— 参数如何随时间变化"),
            ("H", "身体-环境耦合 —— 身体与环境如何交互影响"),
        ];

        for (domain_letter, domain_desc) in &domains {
            out.push_str(&format!(
                "--- 领域{}: {} ---\n\n",
                domain_letter, domain_desc
            ));
            for p in &profile.parameters {
                if p.id.starts_with(domain_letter) {
                    out.push_str(&format!(
                        "  {:<6} {:<22} = {:>8.4}    {}\n",
                        p.id, p.name, p.value, p.definition
                    ));
                }
            }
            out.push('\n');
        }

        // 耦合分析
        out.push_str(&format!("--- 参数耦合分析 ({} 条激活) ---\n\n", profile.couplings.len()));
        if profile.couplings.is_empty() {
            out.push_str("  (当前参数值组合未触发显著的耦合现象)\n\n");
        } else {
            for (i, c) in profile.couplings.iter().enumerate() {
                out.push_str(&format!(
                    "  [{:02}] {} + {} → {}\n",
                    i + 1,
                    c.param_a,
                    c.param_b,
                    c.phenomenon
                ));
            }
            out.push('\n');
        }

        // 关系坍缩
        if !profile.collapse_table.is_empty() {
            out.push_str("--- 关系中的参数坍缩 ---\n");
            out.push_str("同一个参数在不同关系中取不同值。这不是虚伪，是参数的关系依赖性。\n\n");

            // 表头
            let rel_ids: Vec<&str> = profile.collapse_table[0]
                .values
                .keys()
                .map(|s| s.as_str())
                .collect();
            out.push_str(&format!(
                "  {:<8} {:<20}",
                "参数", "参数名"
            ));
            for rid in &rel_ids {
                out.push_str(&format!(" | {:<10}", rid));
            }
            out.push('\n');
            out.push_str(&format!("  {}\n", "-".repeat(20 + rel_ids.len() * 13)));

            for entry in &profile.collapse_table {
                out.push_str(&format!(
                    "  {:<8} {:<20}",
                    entry.param_id, entry.param_name
                ));
                for rid in &rel_ids {
                    let val = entry.values.get(*rid).copied().unwrap_or(0.0);
                    out.push_str(&format!(" | {:>8.4}", val));
                }
                out.push('\n');
            }
            out.push('\n');
        }

        // ε
        out.push_str(&format!("--- 不可通约余数 ε = {:.4} ---\n\n", profile.epsilon));
        out.push_str(&profile.epsilon_declaration);
        out.push('\n');

        out
    }
}

// ============================================================================
// 便捷函数（用于外部API调用）
// ============================================================================

/// 创建默认的人格系统
pub fn create_system() -> PersonalitySystem {
    PersonalitySystem::new()
}

/// 快速分析一组参数值的耦合效应
pub fn quick_analyze(values: &HashMap<String, f64>) -> Vec<CouplingAnalysisResult> {
    let mut system = PersonalitySystem::new();
    for (id, val) in values {
        let _ = system.set_value(id, *val);
    }
    system.analyze_couplings()
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_creation() {
        let system = PersonalitySystem::new();
        assert_eq!(system.parameter_count(), 84);
    }

    #[test]
    fn test_set_and_get_value() {
        let mut system = PersonalitySystem::new();
        system.set_value("A001", 0.7).unwrap();
        let val = system.get_value("A001").unwrap();
        assert!((val - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_coupling_analysis() {
        let mut system = PersonalitySystem::new();
        system.set_value("A009", 0.8).unwrap();
        system.set_value("B015", 0.8).unwrap();
        let results = system.analyze_couplings();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_advance_time() {
        let mut system = PersonalitySystem::new();
        system.set_drift_rate("B015", -0.01).unwrap();
        let changes = system.advance_time(100.0);
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_phase_change() {
        let mut system = PersonalitySystem::new();
        let changes = system.trigger_phase_change("betrayal");
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_relationship_collapse() {
        let mut system = PersonalitySystem::new();
        system.set_value("B015", 0.7).unwrap();
        system.add_relationship("test_intimate", "intimate");
        system.add_relationship("test_hostile", "hostile");

        let intimate_val = system.collapse_in_relationship("B015", "test_intimate").unwrap();
        let hostile_val = system.collapse_in_relationship("B015", "test_hostile").unwrap();

        assert!(intimate_val > hostile_val);
    }

    #[test]
    fn test_epsilon() {
        let system = PersonalitySystem::new();
        assert!(system.epsilon_value() >= 0.0 && system.epsilon_value() <= 1.0);
    }

    #[test]
    fn test_export_import() {
        let mut system = PersonalitySystem::new();
        system.set_value("A001", 0.5).unwrap();

        let json = system.export_state().unwrap();

        let mut system2 = PersonalitySystem::new();
        system2.import_state(&json).unwrap();

        assert!((system2.get_value("A001").unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_system_info() {
        let system = PersonalitySystem::new();
        let info = system.system_info();
        assert_eq!(info.total_parameters, 84);
        assert_eq!(info.total_domains, 8);
    }
}
