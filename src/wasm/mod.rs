//! WASM 模块 —— WebAssembly接口
//!
//! 通过 wasm-bindgen 提供JavaScript可调用的API。
//! 编译为 .wasm 文件后可在浏览器或Node.js中使用。

use crate::api::PersonalitySystem;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// 全局系统实例（WASM是单线程的）
static mut SYSTEM: Option<PersonalitySystem> = None;

fn get_system() -> &'static mut PersonalitySystem {
    unsafe {
        if SYSTEM.is_none() {
            SYSTEM = Some(PersonalitySystem::new());
        }
        SYSTEM.as_mut().unwrap()
    }
}

/// PAPS WASM 接口
#[wasm_bindgen]
pub struct PapsWasm;

#[wasm_bindgen]
impl PapsWasm {
    /// 创建新的PAPS实例
    #[wasm_bindgen(constructor)]
    pub fn new() -> PapsWasm {
        PapsWasm
    }

    /// 初始化系统
    pub fn init(&mut self) {
        let _ = get_system();
    }

    /// 获取参数总数
    pub fn parameter_count(&self) -> usize {
        get_system().parameter_count()
    }

    /// 获取参数值
    pub fn get_value(&self, param_id: &str) -> f64 {
        get_system().get_value(param_id).unwrap_or(-1.0)
    }

    /// 设置参数值
    pub fn set_value(&mut self, param_id: &str, value: f64) -> bool {
        get_system().set_value(param_id, value).is_ok()
    }

    /// 批量设置参数值（JSON）
    pub fn set_values_json(&mut self, json: &str) -> bool {
        let values: std::collections::HashMap<String, f64> =
            match serde_json::from_str(json) {
                Ok(v) => v,
                Err(_) => return false,
            };
        get_system().set_values(&values).is_ok()
    }

    /// 获取所有参数值（JSON）
    pub fn get_all_values_json(&self) -> String {
        let values = get_system().get_all_values();
        serde_json::to_string(&values).unwrap_or_default()
    }

    /// 分析耦合效应（JSON）
    pub fn analyze_couplings_json(&self) -> String {
        let results = get_system().analyze_couplings();
        serde_json::to_string(&results).unwrap_or_default()
    }

    /// 查找强耦合参数（JSON数组）
    pub fn find_strongly_coupled_json(&self, param_id: &str) -> String {
        let results = get_system().find_strongly_coupled(param_id);
        serde_json::to_string(&results).unwrap_or_default()
    }

    /// 推进时间（天数）
    /// 返回变化的参数（JSON）
    pub fn advance_time(&mut self, days: f64) -> String {
        let changes = get_system().advance_time(days);
        serde_json::to_string(&changes).unwrap_or_default()
    }

    /// 触发相变事件
    /// 返回受影响的参数（JSON）
    pub fn trigger_phase_change(&mut self, event_type: &str) -> String {
        let changes = get_system().trigger_phase_change(event_type);
        serde_json::to_string(&changes).unwrap_or_default()
    }

    /// 设置漂移速率
    pub fn set_drift_rate(&mut self, param_id: &str, rate: f64) -> bool {
        get_system().set_drift_rate(param_id, rate).is_ok()
    }

    /// 检查参数是否已反转
    pub fn is_reversed(&self, param_id: &str) -> bool {
        get_system().is_reversed(param_id)
    }

    /// 添加关系
    pub fn add_relationship(&mut self, rel_id: &str, rel_type: &str) {
        get_system().add_relationship(rel_id, rel_type);
    }

    /// 计算关系坍缩值
    pub fn collapse_in_relationship(&self, param_id: &str, rel_id: &str) -> f64 {
        get_system().collapse_in_relationship(param_id, rel_id).unwrap_or(-1.0)
    }

    /// 跨关系分析（JSON）
    pub fn cross_relational_analysis_json(&self, top_n: usize) -> String {
        let results = get_system().cross_relational_analysis(top_n);
        serde_json::to_string(&results).unwrap_or_default()
    }

    /// 获取ε值
    pub fn epsilon_value(&self) -> f64 {
        get_system().epsilon_value()
    }

    /// 设置ε
    pub fn set_epsilon(&mut self, value: f64, flavor: &str) {
        get_system().set_epsilon(value, flavor);
    }

    /// 应用ε到所有参数（JSON）
    pub fn apply_epsilon_json(&self) -> String {
        let results = get_system().apply_epsilon();
        serde_json::to_string(&results).unwrap_or_default()
    }

    /// 获取ε的哲学声明
    pub fn epsilon_acknowledgment(&self) -> String {
        get_system().epsilon_acknowledgment()
    }

    /// 获取系统信息（JSON）
    pub fn system_info_json(&self) -> String {
        let info = get_system().system_info();
        serde_json::to_string(&info).unwrap_or_default()
    }

    /// 导出系统状态（JSON）
    pub fn export_state_json(&self) -> String {
        get_system().export_state().unwrap_or_default()
    }

    /// 导入系统状态（JSON）
    pub fn import_state_json(&mut self, json: &str) -> bool {
        get_system().import_state(json).is_ok()
    }

    /// 获取所有参数ID列表（JSON数组）
    pub fn all_parameter_ids_json(&self) -> String {
        let ids = get_system().all_parameter_ids();
        serde_json::to_string(&ids).unwrap_or_default()
    }

    /// 获取版本号
    pub fn version(&self) -> String {
        crate::VERSION.to_string()
    }
}

impl Default for PapsWasm {
    fn default() -> Self {
        Self::new()
    }
}
