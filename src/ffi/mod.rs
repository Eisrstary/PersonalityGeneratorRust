//! FFI 模块 —— C语言接口，用于生成DLL
//!
//! 提供C ABI兼容的函数，允许其他语言（C/C++/C#/Python等）调用PAPS系统。

use crate::api::PersonalitySystem;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

// 全局系统实例（线程安全）
static SYSTEM: once_cell::sync::Lazy<Mutex<PersonalitySystem>> =
    once_cell::sync::Lazy::new(|| Mutex::new(PersonalitySystem::new()));

// ============================================================================
// 辅助函数
// ============================================================================

/// 将C字符串转换为Rust字符串
unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

/// 将Rust字符串转换为C字符串（调用者负责释放）
fn string_to_c_str(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

/// 释放C字符串
unsafe fn free_c_str(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// ============================================================================
// FFI 函数
// ============================================================================

/// 初始化系统
#[no_mangle]
pub extern "C" fn paps_init() -> i32 {
    let mut system = SYSTEM.lock().unwrap();
    *system = PersonalitySystem::new();
    0
}

/// 获取参数总数
#[no_mangle]
pub extern "C" fn paps_parameter_count() -> i32 {
    let system = SYSTEM.lock().unwrap();
    system.parameter_count() as i32
}

/// 获取参数值
/// 返回: 参数值（f64），如果参数不存在返回 -1.0
#[no_mangle]
pub extern "C" fn paps_get_value(param_id: *const c_char) -> f64 {
    let id = unsafe { c_str_to_string(param_id) };
    let system = SYSTEM.lock().unwrap();
    system.get_value(&id).unwrap_or(-1.0)
}

/// 设置参数值
/// 返回: 0=成功, -1=失败
#[no_mangle]
pub extern "C" fn paps_set_value(param_id: *const c_char, value: f64) -> i32 {
    let id = unsafe { c_str_to_string(param_id) };
    let mut system = SYSTEM.lock().unwrap();
    match system.set_value(&id, value) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// 获取所有参数值（JSON格式）
/// 调用者需要调用 paps_free_string 释放返回的字符串
#[no_mangle]
pub extern "C" fn paps_get_all_values_json() -> *mut c_char {
    let system = SYSTEM.lock().unwrap();
    let values = system.get_all_values();
    let json = serde_json::to_string(&values).unwrap_or_default();
    string_to_c_str(json)
}

/// 分析耦合效应（JSON格式）
#[no_mangle]
pub extern "C" fn paps_analyze_couplings_json() -> *mut c_char {
    let system = SYSTEM.lock().unwrap();
    let results = system.analyze_couplings();
    let json = serde_json::to_string(&results).unwrap_or_default();
    string_to_c_str(json)
}

/// 推进时间
/// 返回: 变化的参数数量
#[no_mangle]
pub extern "C" fn paps_advance_time(days: f64) -> i32 {
    let mut system = SYSTEM.lock().unwrap();
    let changes = system.advance_time(days);
    changes.len() as i32
}

/// 触发相变事件
/// event_type: "betrayal", "loss", "humiliation", "power_gain", "power_loss", "forgiveness", "mission_failure"
/// 返回: 受影响的参数数量
#[no_mangle]
pub extern "C" fn paps_trigger_phase_change(event_type: *const c_char) -> i32 {
    let et = unsafe { c_str_to_string(event_type) };
    let mut system = SYSTEM.lock().unwrap();
    let changes = system.trigger_phase_change(&et);
    changes.len() as i32
}

/// 设置漂移速率
#[no_mangle]
pub extern "C" fn paps_set_drift_rate(param_id: *const c_char, rate: f64) -> i32 {
    let id = unsafe { c_str_to_string(param_id) };
    let mut system = SYSTEM.lock().unwrap();
    match system.set_drift_rate(&id, rate) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// 添加关系
#[no_mangle]
pub extern "C" fn paps_add_relationship(rel_id: *const c_char, rel_type: *const c_char) {
    let rid = unsafe { c_str_to_string(rel_id) };
    let rt = unsafe { c_str_to_string(rel_type) };
    let mut system = SYSTEM.lock().unwrap();
    system.add_relationship(&rid, &rt);
}

/// 计算关系坍缩值
#[no_mangle]
pub extern "C" fn paps_collapse_in_relationship(
    param_id: *const c_char,
    rel_id: *const c_char,
) -> f64 {
    let pid = unsafe { c_str_to_string(param_id) };
    let rid = unsafe { c_str_to_string(rel_id) };
    let system = SYSTEM.lock().unwrap();
    system.collapse_in_relationship(&pid, &rid).unwrap_or(-1.0)
}

/// 获取ε值
#[no_mangle]
pub extern "C" fn paps_epsilon_value() -> f64 {
    let system = SYSTEM.lock().unwrap();
    system.epsilon_value()
}

/// 设置ε
#[no_mangle]
pub extern "C" fn paps_set_epsilon(value: f64, flavor: *const c_char) {
    let f = unsafe { c_str_to_string(flavor) };
    let mut system = SYSTEM.lock().unwrap();
    system.set_epsilon(value, &f);
}

/// 获取系统信息（JSON格式）
#[no_mangle]
pub extern "C" fn paps_system_info_json() -> *mut c_char {
    let system = SYSTEM.lock().unwrap();
    let info = system.system_info();
    let json = serde_json::to_string(&info).unwrap_or_default();
    string_to_c_str(json)
}

/// 导出系统状态（JSON格式）
#[no_mangle]
pub extern "C" fn paps_export_state_json() -> *mut c_char {
    let system = SYSTEM.lock().unwrap();
    let json = system.export_state().unwrap_or_default();
    string_to_c_str(json)
}

/// 导入系统状态（JSON格式）
#[no_mangle]
pub extern "C" fn paps_import_state_json(json: *const c_char) -> i32 {
    let j = unsafe { c_str_to_string(json) };
    let mut system = SYSTEM.lock().unwrap();
    match system.import_state(&j) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// 释放由PAPS分配的字符串
#[no_mangle]
pub extern "C" fn paps_free_string(ptr: *mut c_char) {
    unsafe { free_c_str(ptr) }
}

/// 获取版本号
#[no_mangle]
pub extern "C" fn paps_version() -> *mut c_char {
    string_to_c_str(crate::VERSION.to_string())
}
