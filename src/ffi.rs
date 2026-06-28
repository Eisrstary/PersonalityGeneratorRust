//! C FFI 导出层 —— 对外部语言友好的 C ABI 接口。
//!
//! 所有函数返回 `FfiResult` 结构体，包含错误码和数据指针。
//! 调用方负责通过对应的 `_free` 函数释放返回的内存。
//!
//! # 使用流程
//!
//! ```c
//! // 1. 生成人格
//! FfiResult r = pg_generate(42, "B015=0.9");
//! if (r.error_code != 0) { /* handle error */ }
//!
//! // 2. 读取参数
//! double guilt = pg_get_value(r.handle, "B015");
//!
//! // 3. 获取文本
//! char* text = pg_to_roleplay(r.handle);
//! printf("%s", text);
//! pg_free_string(text);
//!
//! // 4. 释放
//! pg_free_personality(r.handle);
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::{Generator, Personality, Seed};

// ═══════════════════════════════════════════════════════════════
// 类型
// ═══════════════════════════════════════════════════════════════

/// FFI 返回结果。
#[repr(C)]
pub struct FfiResult {
    /// 人格句柄（非零表示有效）
    pub handle: *mut Personality,
    /// 错误码：0=成功
    pub error_code: i32,
}

/// 错误码常量。
pub const ERR_OK: i32 = 0;
pub const ERR_NULL_POINTER: i32 = -1;
pub const ERR_INVALID_HEX: i32 = -2;
pub const ERR_SEED_EXHAUSTED: i32 = -3;

// ═══════════════════════════════════════════════════════════════
// 生成 API
// ═══════════════════════════════════════════════════════════════

/// 从 i32 种子生成人格。
///
/// `bias` 可为 NULL（无偏向）或如 "B015=0.9,C031=-0.7"。
#[no_mangle]
pub extern "C" fn pg_generate(seed: i32, bias: *const c_char) -> FfiResult {
    let bias_str = unsafe { cstr_to_str(bias) };
    let p = Generator::from_seed(seed, bias_str);
    FfiResult { handle: Box::into_raw(Box::new(p)), error_code: ERR_OK }
}

/// 从 hex 字符串种子生成人格。
///
/// hex 必须为 2048 字符。失败时 handle=NULL，error_code!=0。
#[no_mangle]
pub extern "C" fn pg_generate_from_hex(hex: *const c_char, bias: *const c_char) -> FfiResult {
    let hex_str = match unsafe { cstr_to_str(hex) } {
        Some(s) => s,
        None => return FfiResult { handle: std::ptr::null_mut(), error_code: ERR_NULL_POINTER },
    };
    let bias_str = unsafe { cstr_to_str(bias) };
    match Generator::from_hex(hex_str, bias_str) {
        Ok(p) => FfiResult { handle: Box::into_raw(Box::new(p)), error_code: ERR_OK },
        Err(_) => FfiResult { handle: std::ptr::null_mut(), error_code: ERR_INVALID_HEX },
    }
}

/// 批量生成。
///
/// 返回的人格数组布局：[handle_count, handle_0, handle_1, ...]
/// 调用方通过 `pg_free_batch()` 释放。
#[no_mangle]
pub extern "C" fn pg_generate_batch(count: i32, bias: *const c_char) -> *mut *mut Personality {
    let bias_str = unsafe { cstr_to_str(bias) };
    let batch = Generator::generate(count.max(0) as usize, bias_str);
    let mut handles: Vec<*mut Personality> = batch.into_iter().map(|p| Box::into_raw(Box::new(p))).collect();
    handles.insert(0, handles.len() as *mut Personality); // 第一个元素存长度
    let ptr = handles.as_mut_ptr();
    std::mem::forget(handles);
    ptr
}

// ═══════════════════════════════════════════════════════════════
// 查询 API
// ═══════════════════════════════════════════════════════════════

/// 获取参数值 [0, 1]。id 如 "B015"。无效 id 返回 0.5。
#[no_mangle]
pub extern "C" fn pg_get_value(handle: *const Personality, id: *const c_char) -> f64 {
    if handle.is_null() { return 0.5; }
    let id_str = match unsafe { cstr_to_str(id) } { Some(s) => s, None => return 0.5 };
    unsafe { &*handle }.get(id_str)
}

/// 获取所有 84 个参数值。调用方需传入 f64[84] 缓冲区。
#[no_mangle]
pub extern "C" fn pg_get_all_values(handle: *const Personality, out: *mut f64) {
    if handle.is_null() || out.is_null() { return; }
    let p = unsafe { &*handle };
    let out = unsafe { std::slice::from_raw_parts_mut(out, 84) };
    out.copy_from_slice(p.values());
}

/// 获取缺失计数。
#[no_mangle]
pub extern "C" fn pg_missing_count(handle: *const Personality) -> i32 {
    if handle.is_null() { return 84; }
    unsafe { &*handle }.missing_count() as i32
}

/// 获取指纹。调用方通过 `pg_free_string()` 释放。
#[no_mangle]
pub extern "C" fn pg_fingerprint(handle: *const Personality) -> *mut c_char {
    if handle.is_null() { return std::ptr::null_mut(); }
    let fp = unsafe { &*handle }.fingerprint();
    CString::new(fp).unwrap_or_default().into_raw()
}

// ═══════════════════════════════════════════════════════════════
// 文本输出 API
// ═══════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn pg_to_roleplay(handle: *const Personality) -> *mut c_char {
    if handle.is_null() { return std::ptr::null_mut(); }
    let text = crate::textify::to_roleplay(unsafe { &*handle });
    CString::new(text).unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn pg_to_compact(handle: *const Personality) -> *mut c_char {
    if handle.is_null() { return std::ptr::null_mut(); }
    let text = crate::textify::to_compact(unsafe { &*handle });
    CString::new(text).unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn pg_to_detailed(handle: *const Personality) -> *mut c_char {
    if handle.is_null() { return std::ptr::null_mut(); }
    let text = crate::textify::to_detailed(unsafe { &*handle });
    CString::new(text).unwrap_or_default().into_raw()
}

// ═══════════════════════════════════════════════════════════════
// 种子 API
// ═══════════════════════════════════════════════════════════════

/// 生成随机种子（2048 字符 hex）。调用方通过 `pg_free_string()` 释放。
#[no_mangle]
pub extern "C" fn pg_random_seed_hex() -> *mut c_char {
    CString::new(crate::seed::random_seed().to_string()).unwrap_or_default().into_raw()
}

/// 从 i32 生成种子 hex。
#[no_mangle]
pub extern "C" fn pg_seed_from_int(seed: i32) -> *mut c_char {
    CString::new(Seed::from_i32(seed).to_string()).unwrap_or_default().into_raw()
}

// ═══════════════════════════════════════════════════════════════
// 释放 API
// ═══════════════════════════════════════════════════════════════

/// 释放单个人格。
#[no_mangle]
pub extern "C" fn pg_free_personality(handle: *mut Personality) {
    if !handle.is_null() { unsafe { drop(Box::from_raw(handle)); } }
}

/// 释放批量生成的人格数组。
#[no_mangle]
pub extern "C" fn pg_free_batch(handles: *mut *mut Personality) {
    if handles.is_null() { return; }
    let count = unsafe { *handles } as usize;
    for i in 1..=count {
        unsafe { pg_free_personality(*handles.add(i)); }
    }
    unsafe { drop(Box::from_raw(handles)); }
}

/// 释放字符串。
#[no_mangle]
pub extern "C" fn pg_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}

// ═══════════════════════════════════════════════════════════════
// 内部辅助
// ═══════════════════════════════════════════════════════════════

unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() { return None; }
    CStr::from_ptr(ptr).to_str().ok()
}
