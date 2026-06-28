//! 核心生成器 —— 从种子生成人格参数快照。
//!
//! # 架构
//!
//! - `Personality`：84 个归一化值 [0,1] + 缺失标记 + 指纹
//! - `Bias`：参数偏向配置（Builder 模式）
//! - `Generator`：唯一公共入口

use crate::params::{self, ParamDef, Domain, PARAMS, PARAM_COUNT};
use crate::seed::Seed;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// Personality
// ═══════════════════════════════════════════════════════════════

/// 一次人格生成的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct Personality {
    /// 84 个归一化值 [0, 1]
    values: [f64; PARAM_COUNT],
    /// 缺失标记
    missing: [bool; PARAM_COUNT],
    /// 指纹（前 4 个非缺失参数的 4 位小数值）
    fingerprint: String,
}

impl Personality {
    /// 按 ID 获取归一化值。
    pub fn get(&self, id: &str) -> f64 {
        params::by_id(id)
            .map(|p| self.values[p.index as usize])
            .unwrap_or(0.5)
    }

    /// 按索引获取归一化值。
    pub fn get_index(&self, idx: usize) -> Option<f64> {
        if idx < PARAM_COUNT { Some(self.values[idx]) } else { None }
    }

    /// 所有归一化值的引用。
    pub fn values(&self) -> &[f64; PARAM_COUNT] { &self.values }

    /// 缺失标记。
    pub fn missing(&self) -> &[bool; PARAM_COUNT] { &self.missing }

    /// 缺失计数。
    pub fn missing_count(&self) -> usize {
        self.missing.iter().filter(|&&m| m).count()
    }

    /// 指纹。
    pub fn fingerprint(&self) -> &str { &self.fingerprint }
}

// ═══════════════════════════════════════════════════════════════
// Bias
// ═══════════════════════════════════════════════════════════════

/// 偏向配置。
///
/// 用 Builder 模式链式调用：
/// ```ignore
/// Bias::new().set("B015", 0.9).set_domain(Domain::Emotion, 0.5).strength(0.7)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Bias {
    biases: [f64; PARAM_COUNT],
    strength: f64,
}

impl Default for Bias {
    fn default() -> Self {
        Self { biases: [0.0; PARAM_COUNT], strength: 0.7 }
    }
}

impl Bias {
    pub fn new() -> Self { Self::default() }

    pub fn set(mut self, id: &str, v: f64) -> Self {
        if v.is_finite() {
            if let Some(p) = params::by_id(id) {
                self.biases[p.index as usize] = v.clamp(-1.0, 1.0);
            }
        }
        self
    }

    pub fn set_domain(mut self, domain: Domain, v: f64) -> Self {
        if v.is_finite() {
            let v = v.clamp(-1.0, 1.0);
            for p in params::by_domain(domain) {
                self.biases[p.index as usize] = v;
            }
        }
        self
    }

    pub fn strength(mut self, s: f64) -> Self {
        if s.is_finite() {
            self.strength = s.clamp(0.0, 1.0);
        }
        self
    }

    /// 获取当前偏向强度值。
    pub fn strength_value(&self) -> f64 { self.strength }

    /// 从字符串解析。格式："B015=0.9,C031=-0.7,STRENGTH=0.5"
    pub fn parse(spec: &str) -> Self {
        let mut bias = Self::new();
        for part in spec.split(&[',', ';'][..]) {
            let part = part.trim();
            if part.is_empty() { continue; }
            let mut kv = part.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim().to_uppercase();
            let val_str = kv.next().unwrap_or("").trim();
            let val: f64 = match val_str.parse::<f64>() {
                Ok(v) if v.is_finite() => v,
                _ => continue,
            };
            if key.len() == 1 {
                if let Some(dom) = Domain::from_code(key.chars().next().unwrap()) {
                    bias = bias.set_domain(dom, val);
                    continue;
                }
            }
            match key.as_str() {
                "STRENGTH" | "S" | "STR" => bias = bias.strength(val),
                _ => bias = bias.set(&key, val),
            }
        }
        bias
    }
}

// ═══════════════════════════════════════════════════════════════
// Generator
// ═══════════════════════════════════════════════════════════════

pub struct Generator {
    #[allow(dead_code)]
    id_to_idx: HashMap<&'static str, usize>,
}

impl Generator {
    pub fn new() -> Self {
        Self { id_to_idx: params::index_map() }
    }

    /// 批量生成。`count` 为 0 时返回空。
    pub fn generate(&self, count: usize, bias: Option<&str>) -> Vec<Personality> {
        let bias = Bias::parse(bias.unwrap_or(""));
        let base = crate::seed::random_seed();
        // 用 base 种子的前 4 字节作为批次基础种子
        let mut tmp = base.clone();
        let base_i32 = tmp.read_f32().map(|v| (v * i32::MAX as f32) as i32).unwrap_or(0);

        (0..count)
            .map(|i| Self::generate_one(&mut Seed::from_i32(base_i32.wrapping_add(i as i32)), &bias))
            .collect()
    }

    /// 从整数种子生成。
    pub fn from_seed(&self, seed: i32, bias: Option<&str>) -> Personality {
        let bias = Bias::parse(bias.unwrap_or(""));
        Self::generate_one(&mut Seed::from_i32(seed), &bias)
    }

    /// 从 hex 种子生成。
    pub fn from_hex(&self, hex: &str, bias: Option<&str>) -> Result<Personality, crate::seed::SeedError> {
        let seed = Seed::from_hex(hex)?;
        let bias = Bias::parse(bias.unwrap_or(""));
        Ok(Self::generate_one(&mut seed.clone(), &bias))
    }

    fn generate_one(seed: &mut Seed, bias: &Bias) -> Personality {
        seed.reset();
        let mut values = [0.5f64; PARAM_COUNT];
        let mut missing = [false; PARAM_COUNT];

        for i in 0..PARAM_COUNT {
            let def = &PARAMS[i];

            // 缺失判断：15%
            let is_missing = seed.read_f32().map(|v| v < 0.15).unwrap_or(true);

            if is_missing {
                values[i] = 0.5;
                missing[i] = true;
                continue;
            }

            // 生成随机值
            let rnd = match seed.read_f64() {
                Ok(v) if def.bipolar => v * 2.0 - 1.0, // [-1, 1)
                Ok(v) => v,                              // [0, 1)
                Err(_) => { values[i] = 0.5; missing[i] = true; continue; }
            };

            // 应用偏向
            let rnd = apply_bias(rnd, def, bias, i);

            // 映射到原始值范围
            let raw = if def.bipolar {
                def.range.mid() + rnd * def.range.span() / 2.0
            } else {
                def.range.min + rnd * def.range.span()
            };

            // 裁剪并归一化
            values[i] = def.range.normalize(def.range.clamp(raw));
        }

        let fingerprint = build_fingerprint(&values, &missing);
        Personality { values, missing, fingerprint }
    }
}

impl Default for Generator {
    fn default() -> Self { Self::new() }
}

// ═══════════════════════════════════════════════════════════════
// 内部函数
// ═══════════════════════════════════════════════════════════════

/// 非线性偏向拉力：effective_pull = strength × √|bias|
fn apply_bias(rnd: f64, def: &ParamDef, bias: &Bias, idx: usize) -> f64 {
    if bias.strength <= 0.0 { return rnd; }
    let eb = bias.biases[idx];
    if eb.abs() < 0.001 { return rnd; }

    let pull = bias.strength * eb.abs().sqrt();

    if def.bipolar {
        (rnd + (eb - rnd) * pull).clamp(-1.0, 1.0)
    } else {
        let target = if eb > 0.0 { 1.0 } else { 0.0 };
        (rnd + (target - rnd) * pull).clamp(0.0, 1.0)
    }
}

/// 指纹：前 4 个非缺失参数的 4 位小数值，用 | 分隔。
fn build_fingerprint(values: &[f64; PARAM_COUNT], missing: &[bool; PARAM_COUNT]) -> String {
    let parts: Vec<String> = values.iter()
        .zip(missing.iter())
        .filter(|(_, &m)| !m)
        .take(4)
        .map(|(v, _)| format!("{v:.4}"))
        .collect();
    if parts.is_empty() { String::from("ALL_MISSING") } else { parts.join("|") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let gen = Generator::new();
        let a = gen.from_seed(42, None);
        let b = gen.from_seed(42, None);
        assert_eq!(a.values, b.values);
        assert_eq!(a.fingerprint, b.fingerprint);
    }

    #[test]
    fn different_seeds_differ() {
        let gen = Generator::new();
        assert_ne!(gen.from_seed(1, None).fingerprint, gen.from_seed(99999, None).fingerprint);
    }

    #[test]
    fn bias_monotonic() {
        let gen = Generator::new();
        let none = gen.from_seed(42, None).get("B015");
        let high = gen.from_seed(42, Some("B015=1.0,STRENGTH=1.0")).get("B015");
        assert!(high > none, "Bias should increase value");
    }

    #[test]
    fn bias_nan_rejected() {
        let bias = Bias::parse("B015=NaN,STRENGTH=NaN");
        assert_eq!(bias.strength, 0.7); // default
    }

    #[test]
    fn values_in_range() {
        let gen = Generator::new();
        for seed in 0..100 {
            let p = gen.from_seed(seed, None);
            for (i, &v) in p.values.iter().enumerate() {
                assert!(v.is_finite() && (0.0..=1.0).contains(&v),
                    "seed={seed} param={} value={v}", PARAMS[i].id);
            }
        }
    }

    #[test]
    fn fingerprint_format() {
        let gen = Generator::new();
        let p = gen.from_seed(42, None);
        if p.fingerprint != "ALL_MISSING" {
            for part in p.fingerprint.split('|') {
                let _: f64 = part.parse().unwrap();
            }
        }
    }
}
