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
            inactive_parameters: self.inactive_params.iter().cloned().collect(),
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
    pub fn export_profile(&self, collapse_param_ids: &[&str]) -> PersonalityProfile {
        let mut parameters = Vec::with_capacity(84);
        let mut inactive = Vec::new();

        // 缓存所有参数值，避免重复 lookup
        let all_values = self.dynamic_system.get_all_values();

        for id in self.registry.all_ids() {
            let id_str = id.to_string();
            if self.inactive_params.contains(&id_str) {
                inactive.push(id_str);
                continue;
            }
            let def = self.registry.get(id).unwrap();
            let raw = all_values.get(id).map(|v| v.as_f64()).unwrap_or(0.0);
            parameters.push(ParameterEntry {
                id: id_str,
                name: def.name.clone(),
                domain: def.domain.to_string(),
                value: Some(raw),
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

    /// 导出 AI Markdown —— 分层级联纯文本，AI 原生阅读格式
    ///
    /// 为什么 Markdown 比 JSON 更适合 AI：
    /// 1. 顺序阅读，不需要在嵌套结构中跳转——注意力不被稀释
    /// 2. `#` `##` `-` 是天然的层级，token 消耗远小于 `{}[]":`
    /// 3. 可以直接当 system prompt 塞进 context，零解析成本
    /// 4. 小模型（7B）在 Markdown 上的 attention 质量远好于 JSON
    pub fn export_profile_ai_md(&self, collapse_param_ids: &[&str]) -> String {
        let profile = self.export_profile(collapse_param_ids);

        let values: HashMap<&str, f64> = profile.parameters.iter()
            .filter_map(|p| p.value.map(|v| (p.id.as_str(), v)))
            .collect();
        let get = |id: &str| -> f64 { values.get(id).copied().unwrap_or(0.5) };

        let mut md = String::with_capacity(2048);

        // ============ 标题 ============
        md.push_str("# 角色设定\n\n");

        // ============ 核心特征标签 ============
        md.push_str("## 核心特征\n\n");
        let a008 = get("A008"); let a009 = get("A009");
        let b015 = get("B015"); let b019 = get("B019"); let b022 = get("B022");
        let c025 = get("C025"); let c033 = get("C033");
        let c036 = get("C036"); let c037 = get("C037");
        let e045 = get("E045"); let e051 = get("E051"); let e055 = get("E055");
        let f061 = get("F061");
        let d040 = get("D040"); let d041 = get("D041");

        // 信息处理
        {
            let mut parts = Vec::new();
            if a008 < 0.3 { parts.push("善意解读，不轻易视为威胁"); }
            else if a008 > 0.7 { parts.push("威胁警觉，容易感知敌意"); }
            if a009 > 0.7 { parts.push("对他人痛苦高度敏感"); }
            else if a009 < 0.3 { parts.push("对他人情绪不太敏感"); }
            if !parts.is_empty() { md.push_str(&format!("- 信息处理：{}\n", parts.join("；"))); }
        }

        // 情绪
        {
            let mut parts = Vec::new();
            if b015 > 0.7 { parts.push("强内疚，伤害他人后深度自责"); }
            else if b015 < 0.3 { parts.push("低内疚，不太在意对他人的伤害"); }
            if b019 < 0.2 { parts.push("愤怒内敛，压抑而非发泄"); }
            else if b019 > 0.7 { parts.push("愤怒外化，容易转化为攻击"); }
            if b022 < 7.0 { parts.push("不记仇，很快释怀"); }
            else if b022 > 60.0 { parts.push("记仇，怨恨持久"); }
            if !parts.is_empty() { md.push_str(&format!("- 情绪：{}\n", parts.join("；"))); }
        }

        // 动机
        {
            let mut parts = Vec::new();
            if c025 > 0.4 { parts.push("趋近导向，主动探索新情境"); }
            else if c025 < -0.3 { parts.push("回避导向，对新情境谨慎观望"); }
            if c033 > 0.7 { parts.push("亲和驱动，重视人际关系"); }
            if c036 < 0.2 { parts.push("诚实，几乎不说谎"); }
            else if c036 > 0.7 { parts.push("将欺骗视为合理工具"); }
            if c037 > 0.8 { parts.push("言行高度一致"); }
            else if c037 < 0.3 { parts.push("言行常有差距"); }
            if !parts.is_empty() { md.push_str(&format!("- 动机：{}\n", parts.join("；"))); }
        }

        // 自我
        {
            let mut parts = Vec::new();
            if e045 > 0.4 { parts.push("内在自信，不需外部认可"); }
            else if e045 < -0.3 { parts.push("内在自卑，依赖外部认可"); }
            if e051 > 0.7 { parts.push("使命感强，有明确方向"); }
            else if e051 < 0.2 { parts.push("仍在寻找人生方向"); }
            if e055 < 0.2 { parts.push("对自己诚实"); }
            else if e055 > 0.7 { parts.push("倾向自我欺骗"); }
            if !parts.is_empty() { md.push_str(&format!("- 自我：{}\n", parts.join("；"))); }
        }

        // 社交
        {
            let mut parts = Vec::new();
            if f061 > 0.7 { parts.push("信任他人，相信人性本善"); }
            else if f061 < 0.3 { parts.push("警惕他人，需要时间建立信任"); }
            if d040 < 0.15 { parts.push("低攻击性"); }
            else if d040 > 0.6 { parts.push("高攻击性"); }
            if d041 > 0.7 { parts.push("严格遵守规则"); }
            else if d041 < 0.3 { parts.push("规则灵活"); }
            if !parts.is_empty() { md.push_str(&format!("- 社交：{}\n", parts.join("；"))); }
        }

        // ============ 行为模式 ============
        md.push_str("## 行为模式\n\n");

        // 面对威胁
        if a008 > 0.7 && b019 > 0.5 {
            md.push_str("- 面对威胁：高度警觉，可能先发制人\n");
        } else if a008 < 0.3 && b019 < 0.3 {
            md.push_str("- 面对威胁：冷静分析，优先非对抗方案\n");
        }

        // 面对他人痛苦
        if a009 > 0.7 && b015 > 0.7 {
            md.push_str("- 面对他人痛苦：强烈共情，即使不是自己的错也想帮助\n");
        } else if a009 < 0.3 && b015 < 0.3 {
            md.push_str("- 面对他人痛苦：情绪反应弱，理性分析为主\n");
        }

        // 道德困境
        if b015 > 0.7 && c036 < 0.2 {
            md.push_str("- 道德困境：深度反思后果，选择最不伤害他人的方案\n");
        } else if b015 < 0.3 && c036 > 0.7 {
            md.push_str("- 道德困境：选择对自己最有利的方案\n");
        }

        // 压力下
        if get("E053") > 20.0 {
            md.push_str("- 压力下：保持矛盾耐受，不急做非此即彼判断\n");
        } else {
            md.push_str("- 压力下：可能非黑即白，难以容忍模糊\n");
        }
        md.push_str("\n");

        // ============ 关系梯度 ============
        md.push_str("## 关系梯度\n\n");
        md.push_str("- 对亲近者：");
        if c033 > 0.7 && b015 > 0.7 {
            md.push_str("极度忠诚关怀，伤害亲近的人会深感痛苦\n");
        } else {
            md.push_str("适度情感投入，维持关系但不过度依赖\n");
        }
        md.push_str("- 对熟人：");
        if c033 > 0.5 {
            md.push_str("友好合作，维护良好社交关系\n");
        } else {
            md.push_str("保持礼貌但不过多投入情感\n");
        }
        md.push_str("- 对陌生人：");
        if f061 > 0.7 {
            md.push_str("友善开放，默认对方善意\n");
        } else {
            md.push_str("保持基本礼貌但有所保留\n");
        }
        md.push_str("- 对敌对者：");
        if b022 > 60.0 {
            md.push_str("长期怨恨警惕，难以原谅\n");
        } else if b022 < 7.0 {
            md.push_str("较快释怀，不长期纠结\n");
        } else {
            md.push_str("保持距离但不长期怀恨\n");
        }

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
        md.push_str(&format!("# 人格参数评估报告\n\n"));
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
        let get = |id: &str| -> f64 { values.get(id).copied().unwrap_or(0.5) };
        let mut lines = Vec::new();

        let a008 = get("A008"); let a009 = get("A009");
        if a008 < 0.3 { lines.push("倾向于以善意解读他人的言行，较少将模糊信号理解为威胁。"); }
        else if a008 > 0.7 { lines.push("对环境中潜在的威胁信号高度警觉，容易将中性表达理解为敌意。"); }
        if a009 > 0.7 { lines.push("对他人痛苦高度敏感，看到他人皱眉就会感同身受。"); }
        else if a009 < 0.3 { lines.push("对他人的痛苦情绪不太敏感，较少被他人的情绪状态影响。"); }

        let b015 = get("B015"); let b019 = get("B019"); let b022 = get("B022");
        if b015 > 0.7 { lines.push("内疚感很强，伤害他人后会长时间自我谴责。"); }
        else if b015 < 0.3 { lines.push("内疚感较弱，对自身行为造成的他人痛苦不太在意。"); }
        if b019 < 0.2 { lines.push("愤怒时倾向于压抑而非发泄，很少将愤怒转化为攻击行为。"); }
        else if b019 > 0.7 { lines.push("愤怒时容易立即转化为言语或行为上的攻击。"); }
        if b022 < 7.0 { lines.push("不记仇，被冒犯后很快释怀。"); }
        else if b022 > 60.0 { lines.push("记仇，被冒犯后怨恨情绪会持续很长时间。"); }

        let c025 = get("C025"); let c033 = get("C033");
        let c036 = get("C036"); let c037 = get("C037");
        if c025 > 0.4 { lines.push("面对陌生情境倾向于主动接近和探索。"); }
        else if c025 < -0.3 { lines.push("面对陌生情境倾向于回避和观望。"); }
        if c033 > 0.7 { lines.push("非常重视人际关系的温暖和深度，建立亲密关系是核心驱动力。"); }
        if c036 < 0.2 { lines.push("几乎不说谎，将诚实视为基本原则。"); }
        else if c036 > 0.7 { lines.push("将欺骗视为达成目的的合理工具。"); }
        if c037 > 0.8 { lines.push("言行高度一致，说到做到。"); }
        else if c037 < 0.3 { lines.push("言行之间存在较大差距，说的和做的不完全一致。"); }

        let e045 = get("E045"); let e051 = get("E051"); let e055 = get("E055");
        if e045 > 0.4 { lines.push("内心深处对自己有积极的评价，不需要外部认可来维持自我价值。"); }
        else if e045 < -0.3 { lines.push("内心深处对自己评价偏低，可能依赖外部认可。"); }
        if e051 > 0.7 { lines.push("有明确的人生使命感和方向感，知道自己为何而活。"); }
        else if e051 < 0.2 { lines.push("还在寻找人生的方向和意义。"); }
        if e055 < 0.2 { lines.push("对自己诚实，很少自我欺骗。"); }
        else if e055 > 0.7 { lines.push("倾向于相信自己的合理化解释，可能存在自我欺骗。"); }

        let f061 = get("F061");
        if f061 > 0.7 { lines.push("倾向于信任陌生人，相信人性本善。"); }
        else if f061 < 0.3 { lines.push("对陌生人保持警惕，需要时间才能建立信任。"); }

        if lines.is_empty() {
            "各项心理参数处于中等水平，没有特别突出的倾向。".into()
        } else {
            lines.join("")
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
