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
