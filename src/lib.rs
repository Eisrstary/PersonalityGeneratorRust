//! # 人格原子参数系统 (PAPS)
//!
//! **无原型 · 纯参数 · 全光谱**
//!
//! 没有"人格类型"。没有"好人"与"坏人"。没有"原型"。
//! 只有参数。参数在关系中坍缩。参数在时间里漂移。参数在情境中撕裂。
//!
//! 每一个具体的人 = 所有参数在特定历史/关系/情境下的唯一一次取值。
//! 这个取值不可分类、不可复制、不可预测。
//!
//! ## 快速开始
//!
//! ```
//! use personality_generator::Generator;
//!
//! let gen = Generator::new();
//! let p = gen.from_seed(42, None);
//!
//! println!("指纹: {}", p.fingerprint());
//! println!("内疚感: {:.2}", p.get("B015"));
//! println!("缺失: {}/84", p.missing_count());
//!
//! // 带偏向生成
//! let p = gen.from_seed(42, Some("B015=0.9,C031=-0.7"));
//!
//! // 文本输出
//! println!("{}", personality_generator::textify::to_roleplay(&p));
//! ```
//!
//! ## 模块
//!
//! | 模块 | 职责 |
//! |------|------|
//! | `seed` | 确定性熵源（xorshift128+）、Seed、EntropySource trait |
//! | `params` | 84 个原子参数定义、Domain 枚举、Range 类型 |
//! | `generator` | Personality、Bias、Generator |
//! | `textify` | 三种文本输出模式 |

pub mod seed;
pub mod params;
pub mod generator;
pub mod textify;

pub use generator::{Bias, Generator, Personality};
pub use params::{Domain, ParamDef, Range, PARAMS};
pub use seed::Seed;
