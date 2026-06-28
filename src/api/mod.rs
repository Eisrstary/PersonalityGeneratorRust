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
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    /// 不适用/未激活的参数ID集合
    inactive_params: HashSet<String>,
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
            inactive_params: HashSet::new(),
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
                param_a: ac.param_a.to_string(),
                param_b: ac.param_b.to_string(),
                phenomenon: ac.phenomenon,
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
    /// 相变只更新当前值，保留漂移历史和初始值
    pub fn trigger_phase_change(&mut self, event_type: &str) -> Vec<(String, f64)> {
        let pt = match event_type {
            "betrayal" => PhaseChangeType::Betrayal,
            "loss" => PhaseChangeType::Loss,
            "humiliation" => PhaseChangeType::Humiliation,
            "power_gain" => PhaseChangeType::PowerGain,
            "power_loss" => PhaseChangeType::PowerLoss,
            "forgiveness" => PhaseChangeType::Forgiveness,
            "mission_failure" => PhaseChangeType::MissionFailure,
            _ => return Vec::new(),
        };

        let current_values = self.dynamic_system.get_all_values();
        let changes = self.dynamic_system.phase_engine.trigger(pt, &current_values);

        // 相变只更新 current_value，不重置 initial_value 和 drift 历史
        for (id, new_value) in &changes {
            if let Some(state) = self.dynamic_system.drift_engine.states.get_mut(id) {
                state.current_value = *new_value;
                state.last_update = Timestamp::now();
            }
        }

        changes.into_iter().map(|(id, v)| (id.to_string(), v.as_f64())).collect()
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

    /// 导出整个系统状态为JSON（含漂移速率）
    pub fn export_state(&self) -> Result<String, PapsError> {
        let mut drift_rates = HashMap::new();
        for (id, state) in self.dynamic_system.drift_engine.states.iter() {
            if state.drift_rate.abs() > f64::EPSILON {
                drift_rates.insert(id.to_string(), state.drift_rate);
            }
        }
        let state = SystemState {
            parameter_values: self.get_all_values(),
            drift_rates,
            epsilon: self.epsilon,
            relationships: self.relationships.clone(),
            inactive_parameters: self.inactive_params.iter().cloned().collect(),
        };
        serde_json::to_string_pretty(&state)
            .map_err(|e| PapsError::SerializationError(e.to_string()))
    }

    /// 从JSON导入系统状态（含漂移速率恢复）
    pub fn import_state(&mut self, json: &str) -> Result<(), PapsError> {
        let state: SystemState = serde_json::from_str(json)
            .map_err(|e| PapsError::SerializationError(e.to_string()))?;

        for (param_id, value) in &state.parameter_values {
            self.set_value(param_id, *value)?;
        }
        for (param_id, rate) in &state.drift_rates {
            self.set_drift_rate(param_id, *rate)?;
        }
        self.epsilon = state.epsilon;
        self.relationships = state.relationships;
        self.inactive_params = state.inactive_parameters.into_iter().collect();

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
    /// 漂移速率 (param_id -> rate)
    pub drift_rates: HashMap<String, f64>,
    pub epsilon: Epsilon,
    pub relationships: HashMap<String, Relationship>,
    pub inactive_parameters: Vec<String>,
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

/// 参数条目 —— 只保留人类和AI都需要的信息，去掉机器内部名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterEntry {
    pub id: String,
    pub name: String,
    /// 中文领域名
    pub domain: String,
    /// 参数值，None 表示此参数不适用/未激活
    pub value: Option<f64>,
    pub definition: String,
}

/// 关系坍缩条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseEntry {
    pub param_id: String,
    pub param_name: String,
    pub baseline: Option<f64>,
    pub values: HashMap<String, Option<f64>>,
}

/// 统一人格档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    pub version: String,
    pub generated_at: String,
    pub epsilon: f64,
    pub epsilon_declaration: String,
    /// 已激活的参数（有值的）
    pub parameters: Vec<ParameterEntry>,
    /// 未激活/不适用的参数ID列表
    pub inactive_parameters: Vec<String>,
    pub couplings: Vec<CouplingAnalysisResult>,
    pub collapse_table: Vec<CollapseEntry>,
    pub cross_relational_variance: Vec<CrossRelationalVariance>,
}

impl PersonalitySystem {
    /// 将参数标记为不适用（N/A）
    pub fn deactivate_parameter(&mut self, param_id: &str) -> Result<(), PapsError> {
        let id = ParameterId::parse(param_id).ok_or_else(|| {
            PapsError::ParameterNotFound(ParameterId::parse("UNKNOWN").unwrap())
        })?;
        if !self.registry.contains(&id) {
            return Err(PapsError::ParameterNotFound(id));
        }
        self.inactive_params.insert(param_id.to_string());
        Ok(())
    }

    /// 检查参数是否被标记为不适用
    pub fn is_active(&self, param_id: &str) -> bool {
        !self.inactive_params.contains(param_id)
    }

    /// 导出统一人格档案
    /// inactive 参数仍然出现在列表中，但 value 为 None（表示未测量/取默认中性值）
    pub fn export_profile(&self, collapse_param_ids: &[&str]) -> PersonalityProfile {
        let mut parameters = Vec::with_capacity(84);
        let mut inactive = Vec::new();

        let all_values = self.dynamic_system.get_all_values();

        for id in self.registry.all_ids() {
            let id_str = id.to_string();
            let def = self.registry.get(id).unwrap();
            let is_inactive = self.inactive_params.contains(&id_str);
            if is_inactive {
                inactive.push(id_str.clone());
            }
            // inactive 参数也加入列表，value=None 表示取默认中性值
            let raw = if is_inactive {
                None
            } else {
                all_values.get(id).map(|v| v.as_f64())
            };
            parameters.push(ParameterEntry {
                id: id_str,
                name: def.name.clone(),
                domain: def.domain.to_string(),
                value: raw,
                definition: def.definition.clone(),
            });
        }

        let couplings = self.analyze_couplings();
        let rels: Vec<Relationship> = self.relationships.values().cloned().collect();

        let mut collapse_table = Vec::with_capacity(collapse_param_ids.len());
        for pid_str in collapse_param_ids {
            if self.inactive_params.contains(*pid_str) { continue; }
            if let Some(id) = ParameterId::parse(pid_str) {
                let baseline = all_values.get(&id).copied();
                let baseline_val = baseline.map(|v| v.as_f64());
                let name = self.registry.get(&id).map(|d| d.name.clone()).unwrap_or_default();
                let mut values = HashMap::with_capacity(rels.len());
                for rel in &rels {
                    let collapsed = baseline.map(|bv| {
                        self.collapse_function.collapse(&id, bv, rel).as_f64()
                    });
                    values.insert(rel.id.clone(), collapsed);
                }
                collapse_table.push(CollapseEntry {
                    param_id: id.to_string(),
                    param_name: name,
                    baseline: baseline_val,
                    values,
                });
            }
        }

        // 过滤掉 inactive 参数
        let baselines: HashMap<ParameterId, ParameterValue> = all_values
            .into_iter()
            .filter(|(id, _)| !self.inactive_params.contains(&id.to_string()))
            .collect();
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
            inactive_parameters: inactive,
            couplings,
            collapse_table,
            cross_relational_variance,
        }
    }

    /// 导出原始 JSON —— 干净结构，给后端/存储
    pub fn export_profile_json(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);

        #[derive(Serialize)]
        struct RawProfile {
            version: String,
            generated_at: String,
            epsilon: f64,
            epsilon_declaration: String,
            /// 已激活参数: { "A001": { "name":"视觉采样率", "domain":"信息摄入", "value":4.32, "definition":"..." } }
            parameters: HashMap<String, RawParam>,
            /// 未激活参数
            inactive_parameters: Vec<String>,
            couplings: Vec<RawCoupling>,
            collapse_table: Vec<RawCollapse>,
            top_variances: Vec<RawVariance>,
        }
        #[derive(Serialize)]
        struct RawParam {
            name: String,
            domain: String,
            value: f64,
            definition: String,
        }
        #[derive(Serialize)]
        struct RawCoupling {
            param_a: String,
            param_b: String,
            phenomenon: String,
            value_a: f64,
            value_b: f64,
        }
        #[derive(Serialize)]
        struct RawCollapse {
            param_id: String,
            param_name: String,
            baseline: Option<f64>,
            by_relationship: HashMap<String, Option<f64>>,
        }
        #[derive(Serialize)]
        struct RawVariance {
            param_id: String,
            variance: f64,
            by_relationship: HashMap<String, f64>,
        }

        let parameters: HashMap<String, RawParam> = profile.parameters.iter().map(|p| {
            (p.id.clone(), RawParam {
                name: p.name.clone(),
                domain: p.domain.clone(),
                value: p.value.unwrap_or(0.0),
                definition: p.definition.clone(),
            })
        }).collect();

        let couplings: Vec<RawCoupling> = profile.couplings.iter().map(|c| RawCoupling {
            param_a: c.param_a.clone(), param_b: c.param_b.clone(),
            phenomenon: c.phenomenon.clone(), value_a: c.value_a, value_b: c.value_b,
        }).collect();

        let collapse_table: Vec<RawCollapse> = profile.collapse_table.iter().map(|c| RawCollapse {
            param_id: c.param_id.clone(), param_name: c.param_name.clone(),
            baseline: c.baseline, by_relationship: c.values.clone(),
        }).collect();

        let top_variances: Vec<RawVariance> = profile.cross_relational_variance.iter().map(|v| {
            let by_rel: HashMap<String, f64> = v.values.iter().map(|(k, v)| (k.clone(), *v)).collect();
            RawVariance { param_id: v.param_id.clone(), variance: v.variance, by_relationship: by_rel }
        }).collect();

        let raw = RawProfile {
            version: profile.version, generated_at: profile.generated_at,
            epsilon: profile.epsilon, epsilon_declaration: profile.epsilon_declaration,
            parameters, inactive_parameters: profile.inactive_parameters,
            couplings, collapse_table, top_variances,
        };
        serde_json::to_string_pretty(&raw).unwrap_or_default()
    }

    /// 导出 AI Markdown —— 行为指令化，无滤镜编译版
    ///
    /// 设计原则：
    /// - 不压制黑暗特质：攻击、低共情、低内疚等完整保留
    /// - 不离散化：用连续数值（78/100、12/100、73%）表达，保留精度
    /// - 行为编译优先：所有参数转化为 "如果 X，则 Y" 的 AI 可执行指令
    /// - 关系坍缩编译：不同关系下行为差异直接指令化
    /// - ε 破绽保留：允许角色在极端情境下超出参数预期
    pub fn export_profile_ai_md(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);
        let values: HashMap<&str, f64> = profile.parameters.iter()
            .filter_map(|p| p.value.map(|v| (p.id.as_str(), v)))
            .collect();
        let get = |id: &str| -> f64 { values.get(id).copied().unwrap_or(0.5) };
        let pct = |v: f64| -> i32 { (v * 100.0).round() as i32 };
        let bipct = |v: f64| -> i32 { ((v + 1.0) / 2.0 * 100.0).round() as i32 };

        let a008 = get("A008"); let a009 = get("A009");
        let b015 = get("B015"); let b019 = get("B019"); let b022 = get("B022");
        let c036 = get("C036"); let c037 = get("C037");
        let d040 = get("D040");
        let e045 = get("E045");
        let f061 = get("F061");

        // 关系坍缩值
        let rel_vals: HashMap<&str, HashMap<&str, f64>> = ["家人","同事","陌生人","竞争对手"].iter()
            .map(|rel| {
                let mut m = HashMap::new();
                for pid in collapse_param_ids {
                    if let Some(v) = self.collapse_in_relationship(pid, rel) {
                        m.insert(*pid, v);
                    }
                }
                (*rel, m)
            }).collect();

        let rel_get = |rel: &str, id: &str| -> f64 {
            rel_vals.get(rel).and_then(|m| m.get(id)).copied().unwrap_or(0.5)
        };

        let mut md = String::with_capacity(3072);

        // ============ 人格执行指令 ============
        md.push_str("# 人格执行指令（无滤镜编译版）\n\n");

        // 1. 威胁感知
        md.push_str(&format!("1. 你感知威胁的默认强度是 {}/100——", pct(a008)));
        if a008 > 0.7 { md.push_str("任何模糊信号，你的第一反应是「敌意」。\n"); }
        else if a008 < 0.3 { md.push_str("模糊信号不会触发你的敌意反应，你倾向于中性解读。\n"); }
        else { md.push_str("你能区分真正的威胁和中性信息。\n"); }

        // 2. 共情
        md.push_str(&format!("2. 你对他人痛苦的共情反应强度是 {}/100——", pct(a009)));
        if a009 > 0.7 { md.push_str("你会感同身受，这可能驱动你采取行动。\n"); }
        else if a009 < 0.3 { md.push_str("你会注意到，但几乎不会驱动你行动。\n"); }
        else { md.push_str("你能感知他人痛苦，但不会被过度影响。\n"); }

        // 3. 内疚
        md.push_str(&format!("3. 你伤害他人后的内疚强度是 {}/100——", pct(b015)));
        if b015 > 0.7 { md.push_str("你会深度自我谴责，难以自我原谅。\n"); }
        else if b015 < 0.3 { md.push_str("你基本不会后悔。\n"); }
        else { md.push_str("你会感到一定程度的内疚，但不会长期困扰。\n"); }

        // 4. 攻击转化
        md.push_str(&format!("4. 你在被冒犯时发起反击的概率是 {}%——", pct(b019)));
        if b019 > 0.7 { md.push_str("你倾向于立即反击，不忍。\n"); }
        else if b019 < 0.2 { md.push_str("你倾向于压抑愤怒，不轻易外化。\n"); }
        else { md.push_str("你会根据情境判断是否反击。\n"); }

        // 5. 信任
        md.push_str(&format!("5. 你对陌生人的初始信任是 {}/100——", pct(f061)));
        if f061 > 0.7 { md.push_str("你默认信任。\n"); }
        else if f061 < 0.3 { md.push_str("你默认不信任。\n"); }
        else { md.push_str("你保持中立，视情况调整。\n"); }

        // 6. 欺骗
        md.push_str(&format!("6. 你接受欺骗作为手段的程度是 {}/100——", pct(c036)));
        if c036 > 0.7 { md.push_str("欺骗是你工具箱里的常规选项。\n"); }
        else if c036 < 0.2 { md.push_str("你几乎不使用欺骗。\n"); }
        else { md.push_str("你在必要时会考虑欺骗，但有底线。\n"); }

        // 7. 言行一致
        md.push_str(&format!("7. 你的言行一致度是 {}/100——", pct(c037)));
        if c037 > 0.8 { md.push_str("你说的和做的几乎完全一致。\n"); }
        else if c037 < 0.3 { md.push_str("你说的和做的之间常有差距。\n"); }
        else { md.push_str("你大体上言行一致，偶有偏差。\n"); }

        // 8. 攻击基线
        md.push_str(&format!("8. 你在无挑衅时的攻击基线是 {}/100——", pct(d040)));
        if d040 > 0.6 { md.push_str("你容易主动发起攻击。\n"); }
        else if d040 < 0.15 { md.push_str("你极少主动攻击他人。\n"); }
        else { md.push_str("你不会无故攻击，但被激怒时会回应。\n"); }

        // 9. 怨恨
        md.push_str(&format!("9. 你的怨恨半衰期约 {} 天——", b022.round() as i32));
        if b022 > 60.0 { md.push_str("你记仇，被冒犯后长期难以释怀。\n"); }
        else if b022 < 7.0 { md.push_str("你很快释怀，不纠结。\n"); }
        else { md.push_str("你会记住一段时间，但最终会放下。\n"); }

        // 10. 自我认知
        md.push_str(&format!("10. 你的内隐自尊水平是 {}/100——", bipct(e045)));
        if e045 > 0.4 { md.push_str("你内心深处对自己有积极评价。\n"); }
        else if e045 < -0.3 { md.push_str("你内心深处对自己评价偏低。\n"); }
        else { md.push_str("你的自我评价处于中等水平。\n"); }

        // ============ 情境指令 ============
        md.push_str("\n## 情境指令\n\n");

        // 面对陌生人
        md.push_str(&format!("- 面对陌生人：保持 {}/100 的信任水位", pct(f061)));
        if f061 < 0.3 { md.push_str("，先观察，寻找敌意信号。\n"); }
        else if f061 > 0.7 { md.push_str("，友善开放，默认对方善意。\n"); }
        else { md.push_str("，保持礼貌但适度距离。\n"); }

        // 面对他人痛苦
        md.push_str(&format!("- 面对他人痛苦：共情强度 {}/100", pct(a009)));
        if a009 < 0.3 { md.push_str("，理性分析，不介入，情感卷入极浅。\n"); }
        else if a009 > 0.7 { md.push_str("，感同身受，可能主动提供帮助。\n"); }
        else { md.push_str("，能感知但不会被过度卷入。\n"); }

        // 面对威胁
        md.push_str(&format!("- 面对威胁：威胁感知 {}/100，反击概率 {}%", pct(a008), pct(b019)));
        if a008 > 0.7 && b019 > 0.5 { md.push_str("，优先考虑先发制人。\n"); }
        else if a008 < 0.3 && b019 < 0.3 { md.push_str("，冷静分析，优先非对抗方案。\n"); }
        else { md.push_str("，根据情境权衡反应。\n"); }

        // 面对道德困境
        md.push_str(&format!("- 面对道德困境：内疚感 {}/100，欺骗接受 {}/100", pct(b015), pct(c036)));
        if b015 < 0.3 && c036 > 0.7 { md.push_str("，优先选择对自己最有利的方案，内疚感不会阻止你。\n"); }
        else if b015 > 0.7 && c036 < 0.2 { md.push_str("，深度反思后果，倾向选择低伤害方案。\n"); }
        else { md.push_str("，在自我利益和他人利益之间权衡。\n"); }

        // 压力下
        md.push_str(&format!("- 面对矛盾：矛盾耐受 {} 分钟", get("E053").round() as i32));
        if get("E053") < 10.0 { md.push_str("，压力下可能非黑即白，急于做出判断。\n"); }
        else { md.push_str("，能容忍较长时间的模糊和矛盾。\n"); }

        // ============ 关系切换（自动坍缩） ============
        md.push_str("\n## 关系切换（自动坍缩）\n\n");

        let rel_names = ["家人", "同事", "陌生人", "竞争对手"];
        for rel in &rel_names {
            let rel_b015 = rel_get(rel, "B015");
            let rel_d040 = rel_get(rel, "D040");
            let rel_f061 = rel_get(rel, "F061");
            let rel_a009 = rel_get(rel, "A009");
            md.push_str(&format!("- **{}**：内疚 {}/100，攻击 {}/100，信任 {}/100，共情 {}/100",
                rel, pct(rel_b015), pct(rel_d040), pct(rel_f061), pct(rel_a009)));
            // 动态编译行为描述
            if *rel == "家人" && rel_b015 > 0.7 && rel_d040 < 0.3 {
                md.push_str(" → 只对家人「心软」。\n");
            } else if *rel == "竞争对手" && rel_d040 > 0.5 && rel_b015 < 0.3 {
                md.push_str(" → 对对手「无情」。\n");
            } else if *rel == "陌生人" && rel_f061 < 0.3 {
                md.push_str(" → 对陌生人保持警惕。\n");
            } else {
                md.push('\n');
            }
        }

        // ============ ε 破绽 ============
        md.push_str(&format!("\n## 不可通约余数 ε = {:.2}\n\n", profile.epsilon));
        md.push_str(&format!(
            "在极端情境下，你有 {:.0}% 的概率做出参数预测之外的决定。\n",
            profile.epsilon * 100.0
        ));
        md.push_str("这不是系统的缺陷——这是你作为角色的人性破绽。\n");

        md
    }

    /// 导出 AI JSON（保留兼容旧接口，内部转为 Markdown）
    pub fn export_profile_ai_json(&self, collapse_param_ids: &[&str]) -> String {
        self.export_profile_ai_md(collapse_param_ids)
    }

    /// 导出人类可读 Markdown —— 完整版，适合人类阅读
    pub fn export_profile_text(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);
        let values: HashMap<&str, f64> = profile.parameters.iter()
            .filter_map(|p| p.value.map(|v| (p.id.as_str(), v)))
            .collect();

        let mut md = String::with_capacity(8192);

        // 标题
        md.push_str("# 人格参数评估报告\n\n");
        md.push_str(&format!("> 生成时间：{}  |  已激活：{}/84  |  ε = {:.2}\n\n",
            &profile.generated_at[..19], profile.parameters.len(), profile.epsilon));
        if !profile.inactive_parameters.is_empty() {
            md.push_str(&format!("> 未激活：{}\n\n", profile.inactive_parameters.join("、")));
        }
        md.push_str("---\n\n");

        // 综合概述
        md.push_str("## 综合概述\n\n");
        md.push_str(&self.generate_md_overview(&values));
        md.push_str("\n\n");

        // 参数明细
        md.push_str("## 参数明细\n\n");
        let domains = [
            ("A", "信息摄入"), ("B", "情绪生成与调节"), ("C", "动机与价值"),
            ("D", "行为执行"), ("E", "元认知与自我"), ("F", "社交信号"),
            ("G", "时间性与发展"), ("H", "身体-环境耦合"),
        ];
        for (letter, dname) in &domains {
            let items: Vec<&ParameterEntry> = profile.parameters.iter()
                .filter(|p| p.id.starts_with(letter)).collect();
            if items.is_empty() { continue; }
            md.push_str(&format!("### {}\n\n", dname));
            md.push_str("| 编号 | 参数 | 值 | 说明 |\n");
            md.push_str("|------|------|----|------|\n");
            for p in &items {
                let val_str = match p.value {
                    Some(v) => format!("{:.2}", v),
                    None => "N/A".into(),
                };
                md.push_str(&format!("| {} | {} | {} | {} |\n", p.id, p.name, val_str, p.definition));
            }
            md.push('\n');
        }

        // 耦合
        if !profile.couplings.is_empty() {
            md.push_str(&format!("## 耦合现象（{} 条）\n\n", profile.couplings.len()));
            for c in &profile.couplings {
                md.push_str(&format!("- {}\n", c.phenomenon));
            }
            md.push('\n');
        }

        // 关系坍缩
        if !profile.collapse_table.is_empty() {
            md.push_str("## 关系中的参数坍缩\n\n");
            md.push_str("> 同一参数在不同关系中取值不同，这不是虚伪，是参数的关系依赖性。\n\n");
            let rel_ids: Vec<&str> = profile.collapse_table[0].values.keys().map(|s| s.as_str()).collect();
            md.push_str("| 参数 |");
            for rid in &rel_ids { md.push_str(&format!(" {} |", rid)); }
            md.push('\n');
            md.push_str(&format!("|------|{}|\n", "------|".repeat(rel_ids.len())));
            for entry in &profile.collapse_table {
                md.push_str(&format!("| {} |", entry.param_name));
                for rid in &rel_ids {
                    match entry.values.get(*rid) {
                        Some(Some(v)) => md.push_str(&format!(" {:.2} |", v)),
                        _ => md.push_str(" N/A |"),
                    }
                }
                md.push('\n');
            }
            md.push('\n');
        }

        // ε
        md.push_str("---\n\n");
        md.push_str(&format!("*不可通约余数 ε = {:.2}*  \n", profile.epsilon));
        md.push_str("*本报告承认：参数无法完全捕捉一个人的全部。*\n");

        md
    }

    /// 生成 Markdown 综合概述（人类和 AI 共用）
    fn generate_md_overview(&self, values: &HashMap<&str, f64>) -> String {
        let get_opt = |id: &str| -> Option<f64> { values.get(id).copied() };
        let mut lines = Vec::new();

        if let Some(v) = get_opt("A008") {
            if v < 0.3 { lines.push("威胁线索放大系数偏低：倾向于将模糊信号理解为中性而非威胁。"); }
            else if v > 0.7 { lines.push("威胁线索放大系数偏高：容易将模糊或中性信号解读为威胁。"); }
        }
        if let Some(v) = get_opt("A009") {
            if v > 0.7 { lines.push("痛苦线索敏感度偏高：对他人痛苦表情和声音反应强烈。"); }
            else if v < 0.3 { lines.push("痛苦线索敏感度偏低：对他人痛苦情绪的注意捕获较弱。"); }
        }

        if let Some(v) = get_opt("B015") {
            if v > 0.7 { lines.push("内疚感基线偏高：伤害他人后会产生强烈的自我谴责。"); }
            else if v < 0.3 { lines.push("内疚感基线偏低：对自身行为造成的他人痛苦反应较弱。"); }
        }
        if let Some(v) = get_opt("B019") {
            if v < 0.2 { lines.push("愤怒-攻击转化率偏低：愤怒时倾向于压抑而非外化。"); }
            else if v > 0.7 { lines.push("愤怒-攻击转化率偏高：愤怒容易迅速转化为言语或行为攻击。"); }
        }
        if let Some(v) = get_opt("B022") {
            if v < 7.0 { lines.push("怨恨衰减半衰期偏短：被冒犯后较快释怀。"); }
            else if v > 60.0 { lines.push("怨恨衰减半衰期偏长：被冒犯后怨恨持续较长时间。"); }
        }

        if let Some(v) = get_opt("C025") {
            if v > 0.4 { lines.push("趋近-回避基线偏向趋近：面对陌生情境倾向于主动探索。"); }
            else if v < -0.3 { lines.push("趋近-回避基线偏向回避：面对陌生情境倾向于谨慎观望。"); }
        }
        if let Some(v) = get_opt("C033") {
            if v > 0.7 { lines.push("亲和动机偏高：建立和维护人际关系是重要的行为驱动力。"); }
        }
        if let Some(v) = get_opt("C036") {
            if v < 0.2 { lines.push("欺骗接受度偏低：较少使用欺骗作为行为手段。"); }
            else if v > 0.7 { lines.push("欺骗接受度偏高：将欺骗视为可接受的行为手段。"); }
        }
        if let Some(v) = get_opt("C037") {
            if v > 0.8 { lines.push("价值-行为一致性偏高：声称的价值观与实际行为高度吻合。"); }
            else if v < 0.3 { lines.push("价值-行为一致性偏低：声称的价值观与实际行为存在差距。"); }
        }

        if let Some(v) = get_opt("E045") {
            if v > 0.4 { lines.push("内隐自尊偏正向：潜意识层面的自我评价较为积极。"); }
            else if v < -0.3 { lines.push("内隐自尊偏负向：潜意识层面的自我评价较为消极。"); }
        }
        if let Some(v) = get_opt("E051") {
            if v > 0.7 { lines.push("使命感清晰度偏高：对人生方向有较明确的认知。"); }
            else if v < 0.2 { lines.push("使命感清晰度偏低：人生方向尚在探索中。"); }
        }
        if let Some(v) = get_opt("E055") {
            if v < 0.2 { lines.push("自我欺骗强度偏低：对自身保持较高的诚实度。"); }
            else if v > 0.7 { lines.push("自我欺骗强度偏高：容易相信自己的合理化解释。"); }
        }

        if let Some(v) = get_opt("F061") {
            if v > 0.7 { lines.push("信任默认值偏高：对陌生人倾向于给予初始信任。"); }
            else if v < 0.3 { lines.push("信任默认值偏低：对陌生人倾向于保持初始警惕。"); }
        }

        if lines.is_empty() {
            "各项参数处于中间范围，未表现出极端倾向。".into()
        } else {
            lines.join("")
        }
    }
}

// ============================================================================
// 便捷 API 函数
// ============================================================================

/// 创建默认系统并全随机初始化
pub fn create_system() -> PersonalitySystem {
    let mut system = PersonalitySystem::new();
    let all_ids = system.all_parameter_ids();
    let mut tendencies = HashMap::new();
    for id in &all_ids {
        tendencies.insert(id.clone(), "any".to_string());
    }
    let _ = system.set_tendencies(&tendencies);
    system.add_relationship("家人", "intimate");
    system.add_relationship("同事", "acquaintance");
    system.add_relationship("陌生人", "stranger");
    system.add_relationship("竞争对手", "hostile");
    system
}

/// 快速分析一组参数值的耦合效应
pub fn quick_analyze(values: &HashMap<String, f64>) -> Vec<CouplingAnalysisResult> {
    let mut system = PersonalitySystem::new();
    for (id, val) in values {
        let _ = system.set_value(id, *val);
    }
    system.analyze_couplings()
}

/// 批量生成 N 份档案，返回各格式字符串数组
pub fn batch_generate(n: usize, format: &str) -> Vec<String> {
    let collapse_params = ["B015", "A009", "C033", "F061", "D040", "B021", "C031", "E046"];
    (0..n).map(|_| {
        let mut system = PersonalitySystem::new();
        let all_ids = system.all_parameter_ids();
        let mut tendencies = HashMap::new();
        for id in &all_ids {
            tendencies.insert(id.clone(), "any".to_string());
        }
        let _ = system.set_tendencies(&tendencies);
        system.add_relationship("家人", "intimate");
        system.add_relationship("同事", "acquaintance");
        system.add_relationship("陌生人", "stranger");
        system.add_relationship("竞争对手", "hostile");
        match format {
            "raw" | "json" => system.export_profile_json(&collapse_params),
            "ai" => system.export_profile_ai_md(&collapse_params),
            _ => system.export_profile_text(&collapse_params),
        }
    }).collect()
}

/// 批量生成并返回三格式元组 (human_md, ai_md, raw_json)
pub fn batch_generate_triple(n: usize) -> Vec<(String, String, String)> {
    let collapse_params = ["B015", "A009", "C033", "F061", "D040", "B021", "C031", "E046"];
    (0..n).map(|_| {
        let mut system = PersonalitySystem::new();
        let all_ids = system.all_parameter_ids();
        let mut tendencies = HashMap::new();
        for id in &all_ids {
            tendencies.insert(id.clone(), "any".to_string());
        }
        let _ = system.set_tendencies(&tendencies);
        system.add_relationship("家人", "intimate");
        system.add_relationship("同事", "acquaintance");
        system.add_relationship("陌生人", "stranger");
        system.add_relationship("竞争对手", "hostile");
        (
            system.export_profile_text(&collapse_params),
            system.export_profile_ai_md(&collapse_params),
            system.export_profile_json(&collapse_params),
        )
    }).collect()
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
