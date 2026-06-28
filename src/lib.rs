//! # 人格原子参数系统 (PAPS)
//!
//! **无原型 · 纯参数 · 全光谱**
//!
//! 这里没有"人格类型"。没有"好人"与"坏人"。没有"原型"。
//! 这里只有参数。参数在关系中坍缩。参数在时间里漂移。参数在情境中撕裂。
//!
//! 每一个具体的人 = 所有参数在特定历史/关系/情境下的唯一一次取值。
//! 这个取值不可分类、不可复制、不可预测。
//!
//! ## 快速开始
//!
//! ```rust
//! use personality_generator::Generator;
//!
//! let gen = Generator::new();
//!
//! // 从种子生成单个人格
//! let personality = gen.generate_from_seed(42, None);
//!
//! // 带偏向生成
//! let personality = gen.generate_from_seed(42, Some("B015=0.9,C031=-0.7"));
//!
//! // 批量生成
//! let personalities = gen.generate(5, Some("A=0.3"));
//!
//! // 查询参数
//! let guilt = personality.get("B015");
//!
//! // 生成文本描述
//! let text = personality_generator::textify::to_roleplay(&personality);
//! println!("{}", text);
//! ```
//!
//! ## 模块结构
//!
//! - `seed` — 确定性伪随机数生成器（xorshift128+）与 1024 字节种子
//! - `params` — 84 个原子参数定义（A-H 八个领域）
//! - `generator` — 核心生成器、Personality、Bias
//! - `textify` — 文本生成器（角色扮演/紧凑/详细三种模式）

pub mod seed;
pub mod params;
pub mod generator;
pub mod textify;

// 重新导出常用类型
pub use generator::{Bias, Generator, Personality};
pub use params::ParamDef;
pub use seed::Seed;
