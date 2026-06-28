//! # Personality Atomic Parameter System (PAPS)
//!
//! 人格原子参数系统 —— 无原型·纯参数·全光谱
//!
//! 这里没有"人格类型"。没有"好人"与"坏人"。没有"原型"。
//! 这里只有参数。参数在关系中坍缩。参数在时间里漂移。参数在情境中撕裂。
//! 每一个具体的人 = 所有参数在特定历史/关系/情境下的唯一一次取值。
//! 这个取值不可分类、不可复制、不可预测。

pub mod core;
pub mod parameters;
pub mod coupling;
pub mod dynamics;
pub mod relationship;
pub mod epsilon;
pub mod api;

#[cfg(feature = "ffi-export")]
pub mod ffi;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Prelude: 导出最常用的类型
pub mod prelude {
    pub use crate::core::*;
    pub use crate::parameters::*;
    pub use crate::coupling::*;
    pub use crate::dynamics::*;
    pub use crate::relationship::*;
    pub use crate::epsilon::*;
    pub use crate::api::*;
}

/// PAPS 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// 参数总数
pub const TOTAL_PARAMETERS: usize = 84;
/// 参数领域数
pub const TOTAL_DOMAINS: usize = 8;
