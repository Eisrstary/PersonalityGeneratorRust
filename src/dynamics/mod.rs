//! 参数动态系统 —— 不是"轨迹"，是"可能性空间"
//!
//! 实现参数漂移、相变(崩塌)、反转三大动态机制。
//! 每个参数都有独立的漂移函数。漂移受时间、经验、事件、训练、关系影响。

use crate::core::*;
use crate::parameters::{ParameterRegistry, DriftDirection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// DriftEngine: 漂移引擎
// ============================================================================

/// 漂移状态 —— 记录参数漂移的历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftState {
    /// 参数ID
    pub param_id: ParameterId,
    /// 初始值
    pub initial_value: ParameterValue,
    /// 当前值
    pub current_value: ParameterValue,
    /// 漂移开始时间
    pub start_time: Timestamp,
    /// 最后更新时间
    pub last_update: Timestamp,
    /// 漂移速率（单位：值变化/天）
    pub drift_rate: f64,
    /// 漂移方向
    pub direction: DriftDirection,
    /// 累积漂移量
    pub accumulated_drift: f64,
}

/// 漂移引擎 —— 管理所有参数的漂移
pub struct DriftEngine {
    states: HashMap<ParameterId, DriftState>,
    registry: ParameterRegistry,
}

impl DriftEngine {
    /// 从参数注册表创建漂移引擎
    pub fn new(registry: &ParameterRegistry) -> Self {
        let mut states = HashMap::with_capacity(registry.len());

        for param in registry.iter() {
            let state = DriftState {
                param_id: param.id.clone(),
                initial_value: param.default_value,
                current_value: param.default_value,
                start_time: Timestamp::now(),
                last_update: Timestamp::now(),
                drift_rate: 0.0,
                direction: DriftDirection::Variable,
                accumulated_drift: 0.0,
            };
            states.insert(param.id.clone(), state);
        }

        DriftEngine {
            states,
            registry: registry.clone(),
        }
    }

    /// 获取参数的当前值
    pub fn get_value(&self, param_id: &ParameterId) -> Option<ParameterValue> {
        self.states.get(param_id).map(|s| s.current_value)
    }

    /// 设置参数的初始值
    pub fn set_initial_value(&mut self, param_id: &ParameterId, value: ParameterValue) {
        if let Some(state) = self.states.get_mut(param_id) {
            state.initial_value = value;
            state.current_value = value;
            state.start_time = Timestamp::now();
            state.last_update = Timestamp::now();
            state.accumulated_drift = 0.0;
        }
    }

    /// 设置参数的漂移速率（值变化/天）
    pub fn set_drift_rate(&mut self, param_id: &ParameterId, rate: f64) {
        if let Some(state) = self.states.get_mut(param_id) {
            state.drift_rate = rate;
            if rate > 0.0 {
                state.direction = DriftDirection::Increasing;
            } else if rate < 0.0 {
                state.direction = DriftDirection::Decreasing;
            } else {
                state.direction = DriftDirection::Variable;
            }
        }
    }

    /// 应用时间漂移 —— 将参数从上次更新时间推移到当前时间
    pub fn apply_time_drift(&mut self, param_id: &ParameterId, now: &Timestamp) -> Option<f64> {
        let state = self.states.get_mut(param_id)?;
        let days_elapsed = now.days_between(&state.last_update);

        if days_elapsed <= 0.0 || state.drift_rate.abs() < f64::EPSILON {
            return Some(0.0);
        }

        let drift_amount = state.drift_rate * days_elapsed;
        state.accumulated_drift += drift_amount;

        let new_value = state.current_value.as_f64() + drift_amount;

        // 根据参数类型钳制
        let param_def = self.registry.get(param_id)?;
        let clamped = match &param_def.spectrum {
            SpectrumType::Normalized => ParameterValue::normalized(new_value),
            SpectrumType::Bipolar => ParameterValue::bipolar(new_value),
            SpectrumType::Unbounded => ParameterValue::unbounded(new_value),
            _ => ParameterValue::normalized(new_value.clamp(0.0, 1.0)),
        };

        state.current_value = clamped;
        state.last_update = *now;

        Some(drift_amount)
    }

    /// 批量应用所有参数的时间漂移
    pub fn apply_all_drifts(&mut self, now: &Timestamp) -> HashMap<ParameterId, f64> {
        let ids: Vec<ParameterId> = self.states.keys().cloned().collect();
        let mut changes = HashMap::new();

        for id in ids {
            if let Some(drift) = self.apply_time_drift(&id, now) {
                if drift.abs() > f64::EPSILON {
                    changes.insert(id, drift);
                }
            }
        }

        changes
    }

    /// 获取所有漂移状态
    pub fn all_states(&self) -> &HashMap<ParameterId, DriftState> {
        &self.states
    }
}

// ============================================================================
// PhaseChangeEngine: 相变引擎
// ============================================================================

/// 相变事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseChangeEvent {
    /// 事件类型
    pub event_type: PhaseChangeType,
    /// 发生时间
    pub timestamp: Timestamp,
    /// 影响的参数及跳变值
    pub parameter_changes: Vec<(ParameterId, ParameterValue)>,
    /// 事件描述
    pub description: String,
}

/// 相变类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseChangeType {
    /// 背叛
    Betrayal,
    /// 丧失
    Loss,
    /// 羞辱
    Humiliation,
    /// 获得权力
    PowerGain,
    /// 失去权力
    PowerLoss,
    /// 原谅
    Forgiveness,
    /// 见证极端事件
    WitnessExtreme,
    /// 使命幻灭
    MissionFailure,
    /// 自定义
    Custom(u32),
}

/// 相变引擎 —— 管理参数的突然跳变
pub struct PhaseChangeEngine {
    history: Vec<PhaseChangeEvent>,
    #[allow(dead_code)]
    registry: ParameterRegistry,
}

impl PhaseChangeEngine {
    pub fn new(registry: &ParameterRegistry) -> Self {
        PhaseChangeEngine {
            history: Vec::new(),
            registry: registry.clone(),
        }
    }

    /// 触发相变事件
    pub fn trigger(
        &mut self,
        event_type: PhaseChangeType,
        current_values: &HashMap<ParameterId, ParameterValue>,
    ) -> Vec<(ParameterId, ParameterValue)> {
        let mut changes = Vec::new();

        match event_type {
            PhaseChangeType::Betrayal => {
                // 被信任者严重伤害 → F061↓(信任崩塌) + A008↑(威胁放大)
                changes.push((
                    ParameterId::parse("F061").unwrap(),
                    ParameterValue::normalized(0.1),
                ));
                changes.push((
                    ParameterId::parse("A008").unwrap(),
                    ParameterValue::normalized(0.8),
                ));
                changes.push((
                    ParameterId::parse("C033").unwrap(),
                    ParameterValue::normalized(0.1),
                ));
            }
            PhaseChangeType::Loss => {
                // 失去重要他人 → C026↑(意义寻求)
                if let Some(v) = current_values.get(&ParameterId::parse("C026").unwrap()) {
                    changes.push((
                        ParameterId::parse("C026").unwrap(),
                        ParameterValue::unbounded(v.as_f64() + 30.0),
                    ));
                }
            }
            PhaseChangeType::Humiliation => {
                // 公开被羞辱 → B017可能↑或↓ + E046↓
                changes.push((
                    ParameterId::parse("E046").unwrap(),
                    ParameterValue::normalized(0.2),
                ));
            }
            PhaseChangeType::PowerGain => {
                // 突然获得权力 → C031↑ + A010可能从+1跳变到-1
                changes.push((
                    ParameterId::parse("C031").unwrap(),
                    ParameterValue::bipolar(0.8),
                ));
                changes.push((
                    ParameterId::parse("A010").unwrap(),
                    ParameterValue::bipolar(-0.5),
                ));
            }
            PhaseChangeType::PowerLoss => {
                changes.push((
                    ParameterId::parse("C031").unwrap(),
                    ParameterValue::bipolar(-0.6),
                ));
            }
            PhaseChangeType::Forgiveness => {
                // 被受害者原谅 → B015可能从零跳变到极高(延迟内疚涌现)
                if let Some(v) = current_values.get(&ParameterId::parse("B015").unwrap()) {
                    if v.as_f64() < 0.3 {
                        changes.push((
                            ParameterId::parse("B015").unwrap(),
                            ParameterValue::normalized(0.9),
                        ));
                    }
                }
            }
            PhaseChangeType::MissionFailure => {
                // 使命幻灭 → E051↓
                changes.push((
                    ParameterId::parse("E051").unwrap(),
                    ParameterValue::normalized(0.05),
                ));
                changes.push((
                    ParameterId::parse("C026").unwrap(),
                    ParameterValue::unbounded(50.0),
                ));
            }
            PhaseChangeType::WitnessExtreme | PhaseChangeType::Custom(_) => {
                // 默认：多个参数可能同时跳变
            }
        }

        let event = PhaseChangeEvent {
            event_type,
            timestamp: Timestamp::now(),
            parameter_changes: changes.clone(),
            description: format!("{:?}", event_type),
        };
        self.history.push(event);

        changes
    }

    /// 获取相变历史
    pub fn history(&self) -> &[PhaseChangeEvent] {
        &self.history
    }
}

// ============================================================================
// ReversalEngine: 反转引擎
// ============================================================================

/// 反转状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReversalState {
    /// 正常状态（未反转）
    Normal,
    /// 已反转
    Reversed,
}

/// 反转引擎 —— 管理参数的意义翻转
pub struct ReversalEngine {
    states: HashMap<ParameterId, ReversalState>,
    registry: ParameterRegistry,
}

impl ReversalEngine {
    pub fn new(registry: &ParameterRegistry) -> Self {
        let mut states = HashMap::with_capacity(registry.len());
        for param in registry.iter() {
            states.insert(param.id.clone(), ReversalState::Normal);
        }
        ReversalEngine {
            states,
            registry: registry.clone(),
        }
    }

    /// 检查反转条件并触发反转
    pub fn check_and_trigger(
        &mut self,
        param_id: &ParameterId,
        situation: &Situation,
    ) -> Option<ReversalState> {
        let param_def = self.registry.get(param_id)?;

        for reversal in &param_def.reversals {
            let should_reverse = match reversal.trigger.as_str() {
                "极度恐惧" => matches!(situation.situation_type, SituationType::Threat)
                    && situation.intensity > 0.9,
                "极度抑郁" => false, // 需要额外的抑郁状态追踪
                "权力导致重大负面后果" => false,
                "地位上升" => matches!(situation.situation_type, SituationType::Power),
                "过度自我监控" => false,
                "极端情况" => situation.intensity > 0.95,
                "规则制定者背叛系统" => false,
                "使命幻灭" => false,
                "极度疲劳" => matches!(situation.situation_type, SituationType::Fatigue)
                    && situation.intensity > 0.9,
                "极度安全" => matches!(situation.situation_type, SituationType::Safe)
                    && situation.intensity > 0.9,
                _ => false,
            };

            if should_reverse {
                let current = self.states.get(param_id).copied().unwrap_or(ReversalState::Normal);
                let new_state = match current {
                    ReversalState::Normal => ReversalState::Reversed,
                    ReversalState::Reversed => ReversalState::Normal,
                };
                self.states.insert(param_id.clone(), new_state);
                return Some(new_state);
            }
        }

        None
    }

    /// 获取反转状态
    pub fn get_state(&self, param_id: &ParameterId) -> ReversalState {
        self.states.get(param_id).copied().unwrap_or(ReversalState::Normal)
    }

    /// 是否已反转
    pub fn is_reversed(&self, param_id: &ParameterId) -> bool {
        self.get_state(param_id) == ReversalState::Reversed
    }
}

// ============================================================================
// DynamicSystem: 动态系统总成
// ============================================================================

/// 动态系统 —— 整合漂移、相变、反转三大机制
pub struct DynamicSystem {
    pub drift_engine: DriftEngine,
    pub phase_engine: PhaseChangeEngine,
    pub reversal_engine: ReversalEngine,
    registry: ParameterRegistry,
}

impl DynamicSystem {
    pub fn new(registry: &ParameterRegistry) -> Self {
        DynamicSystem {
            drift_engine: DriftEngine::new(registry),
            phase_engine: PhaseChangeEngine::new(registry),
            reversal_engine: ReversalEngine::new(registry),
            registry: registry.clone(),
        }
    }

    /// 获取当前所有参数值
    pub fn get_all_values(&self) -> HashMap<ParameterId, ParameterValue> {
        self.drift_engine
            .all_states()
            .iter()
            .map(|(id, state)| (id.clone(), state.current_value))
            .collect()
    }

    /// 推进时间一步（应用漂移 + 检查情境触发的相变/反转）
    pub fn step(
        &mut self,
        now: &Timestamp,
        situation: Option<&Situation>,
    ) -> DynamicStepResult {
        // 1. 应用时间漂移
        let drift_changes = self.drift_engine.apply_all_drifts(now);

        // 2. 检查反转条件
        let mut reversal_changes = Vec::new();
        if let Some(sit) = situation {
            for id in self.registry.all_ids() {
                if let Some(state) = self.reversal_engine.check_and_trigger(id, sit) {
                    reversal_changes.push((id.clone(), state));
                }
            }
        }

        DynamicStepResult {
            drift_changes,
            reversal_changes,
        }
    }
}

/// 动态步骤结果
#[derive(Debug, Clone)]
pub struct DynamicStepResult {
    pub drift_changes: HashMap<ParameterId, f64>,
    pub reversal_changes: Vec<(ParameterId, ReversalState)>,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_engine_creation() {
        let registry = ParameterRegistry::new();
        let engine = DriftEngine::new(&registry);
        assert_eq!(engine.states.len(), 84);
    }

    #[test]
    fn test_drift_application() {
        let registry = ParameterRegistry::new();
        let mut engine = DriftEngine::new(&registry);

        // Set drift rate for B015 (内疚感)
        engine.set_drift_rate(&ParameterId::parse("B015").unwrap(), -0.01);

        // Advance by 100 days
        let future = Timestamp::from_ms(
            Timestamp::now().unix_ms + (100i64 * 24 * 3600 * 1000),
        );
        let changes = engine.apply_all_drifts(&future);

        assert!(!changes.is_empty());
    }

    #[test]
    fn test_phase_change_betrayal() {
        let registry = ParameterRegistry::new();
        let mut engine = PhaseChangeEngine::new(&registry);

        let values = HashMap::new();
        let changes = engine.trigger(PhaseChangeType::Betrayal, &values);

        assert!(!changes.is_empty());
        // Should include F061↓ and A008↑
        assert!(changes.iter().any(|(id, _)| id.to_string() == "F061"));
        assert!(changes.iter().any(|(id, _)| id.to_string() == "A008"));
    }

    #[test]
    fn test_dynamic_system_step() {
        let registry = ParameterRegistry::new();
        let mut system = DynamicSystem::new(&registry);

        let now = Timestamp::now();
        let situation = Situation {
            situation_type: SituationType::Threat,
            intensity: 1.0,
            duration_seconds: 60.0,
            description: "极度恐惧".into(),
        };

        let result = system.step(&now, Some(&situation));
        // Should have some effects
        // Should have some effects (at minimum, no panics)
        let _ = result.reversal_changes.len();
    }
}
