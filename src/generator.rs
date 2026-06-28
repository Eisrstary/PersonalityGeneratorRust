//! 核心生成器 —— 从种子生成人格参数快照。
//!
//! `Generator` 是无状态零大小类型，所有方法为关联函数。
//! 热路径零堆分配。

use crate::params::{self, ParamDef, Domain, PARAMS, PARAM_COUNT};
use crate::seed::Seed;

// ═══════════════════════════════════════════════════════════════
// Personality
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub struct Personality {
    values: [f64; PARAM_COUNT],
    missing: [bool; PARAM_COUNT],
    fingerprint: String,
}

impl Personality {
    #[inline] pub fn get(&self, id: &str) -> f64 { params::by_id(id).map(|p| self.values[p.index as usize]).unwrap_or(0.5) }
    #[inline] pub fn values(&self) -> &[f64; PARAM_COUNT] { &self.values }
    #[inline] pub fn missing(&self) -> &[bool; PARAM_COUNT] { &self.missing }
    #[inline] pub fn missing_count(&self) -> usize { self.missing.iter().filter(|&&m| m).count() }
    #[inline] pub fn fingerprint(&self) -> &str { &self.fingerprint }
}

// ═══════════════════════════════════════════════════════════════
// Bias
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub struct Bias {
    biases: [f64; PARAM_COUNT],
    strength: f64,
}

impl Default for Bias { fn default() -> Self { Self { biases: [0.0; PARAM_COUNT], strength: 0.7 } } }

impl Bias {
    #[inline] pub fn new() -> Self { Self::default() }

    #[inline]
    pub fn set(mut self, id: &str, v: f64) -> Self {
        if v.is_finite() { if let Some(p) = params::by_id(id) { self.biases[p.index as usize] = v.clamp(-1.0, 1.0); } }
        self
    }

    #[inline]
    pub fn set_domain(mut self, domain: Domain, v: f64) -> Self {
        if v.is_finite() { let v = v.clamp(-1.0, 1.0); for p in params::by_domain(domain) { self.biases[p.index as usize] = v; } }
        self
    }

    #[inline] pub fn strength(mut self, s: f64) -> Self { if s.is_finite() { self.strength = s.clamp(0.0, 1.0); } self }
    #[inline] pub fn strength_value(&self) -> f64 { self.strength }

    pub fn parse(spec: &str) -> Self {
        let mut bias = Self::new();
        for part in spec.split(&[',', ';'][..]) {
            let part = part.trim();
            if part.is_empty() { continue; }
            let (key, val_str) = match part.split_once('=') {
                Some((k, v)) => (k.trim().to_uppercase(), v.trim()),
                None => continue,
            };
            let val: f64 = match val_str.parse::<f64>() { Ok(v) if v.is_finite() => v, _ => continue };
            if key.len() == 1 {
                if let Some(dom) = Domain::from_code(key.chars().next().unwrap()) { bias = bias.set_domain(dom, val); continue; }
            }
            match key.as_str() { "STRENGTH" | "S" | "STR" => bias = bias.strength(val), _ => bias = bias.set(&key, val) }
        }
        bias
    }
}

// ═══════════════════════════════════════════════════════════════
// Generator —— 零大小类型
// ═══════════════════════════════════════════════════════════════

pub struct Generator;

impl Generator {
    #[inline] pub const fn new() -> Self { Self }

    pub fn generate(count: usize, bias: Option<&str>) -> Vec<Personality> {
        let bias = Bias::parse(bias.unwrap_or(""));
        let mut tmp = crate::seed::random_seed();
        let base_i32 = tmp.read_f32().map(|v| (v * i32::MAX as f32) as i32).unwrap_or(0);
        (0..count).map(|i| Self::generate_one(&mut Seed::from_i32(base_i32.wrapping_add(i as i32)), &bias)).collect()
    }

    #[inline] pub fn from_seed(seed: i32, bias: Option<&str>) -> Personality { Self::generate_one(&mut Seed::from_i32(seed), &Bias::parse(bias.unwrap_or(""))) }
    #[inline] pub fn from_hex(hex: &str, bias: Option<&str>) -> Result<Personality, crate::seed::SeedError> { Ok(Self::generate_one(&mut Seed::from_hex(hex)?, &Bias::parse(bias.unwrap_or("")))) }

    #[inline]
    fn generate_one(seed: &mut Seed, bias: &Bias) -> Personality {
        seed.reset();
        let mut values = [0.5f64; PARAM_COUNT];
        let mut missing = [false; PARAM_COUNT];
        for i in 0..PARAM_COUNT {
            let def = &PARAMS[i];
            if seed.read_f32().map(|v| v < 0.15).unwrap_or(true) { missing[i] = true; continue; }
            let rnd = match seed.read_f64() {
                Ok(v) if def.bipolar => v * 2.0 - 1.0, Ok(v) => v,
                Err(_) => { missing[i] = true; continue; }
            };
            let rnd = apply_bias(rnd, def, bias, i);
            let raw = if def.bipolar { def.range.mid() + rnd * def.range.span() * 0.5 } else { def.range.min + rnd * def.range.span() };
            values[i] = def.range.normalize(def.range.clamp(raw));
        }
        Personality { fingerprint: fingerprint(&values, &missing), values, missing }
    }
}

impl Default for Generator { #[inline] fn default() -> Self { Self } }

// ═══════════════════════════════════════════════════════════════
// 内部热路径
// ═══════════════════════════════════════════════════════════════

#[inline]
fn apply_bias(rnd: f64, def: &ParamDef, bias: &Bias, idx: usize) -> f64 {
    if bias.strength <= 0.0 { return rnd; }
    let eb = bias.biases[idx];
    if eb.abs() < 0.001 { return rnd; }
    let pull = bias.strength * eb.abs().sqrt();
    if def.bipolar { (rnd + (eb - rnd) * pull).clamp(-1.0, 1.0) } else { let t = if eb > 0.0 { 1.0 } else { 0.0 }; (rnd + (t - rnd) * pull).clamp(0.0, 1.0) }
}

fn fingerprint(values: &[f64; PARAM_COUNT], missing: &[bool; PARAM_COUNT]) -> String {
    let mut n = 0;
    let mut s = String::with_capacity(32);
    for i in 0..PARAM_COUNT {
        if n >= 4 { break; }
        if !missing[i] {
            if n > 0 { s.push('|'); }
            let _ = std::fmt::write(&mut s, format_args!("{:.4}", values[i]));
            n += 1;
        }
    }
    if n == 0 { String::from("ALL_MISSING") } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn deterministic() { let a = Generator::from_seed(42, None); let b = Generator::from_seed(42, None); assert_eq!(a.values, b.values); assert_eq!(a.fingerprint, b.fingerprint); }
    #[test] fn different_seeds_differ() { assert_ne!(Generator::from_seed(1, None).fingerprint, Generator::from_seed(99999, None).fingerprint); }
    #[test] fn bias_monotonic() { assert!(Generator::from_seed(42, Some("B015=1.0,STRENGTH=1.0")).get("B015") > Generator::from_seed(42, None).get("B015")); }
    #[test] fn bias_nan_rejected() { assert_eq!(Bias::parse("B015=NaN,STRENGTH=NaN").strength_value(), 0.7); }
    #[test] fn values_in_range() { for seed in 0..100 { let p = Generator::from_seed(seed, None); for (i, &v) in p.values.iter().enumerate() { assert!(v.is_finite() && (0.0..=1.0).contains(&v), "seed={seed} param={} value={v}", PARAMS[i].id); } } }
    #[test] fn fingerprint_format() { let p = Generator::from_seed(42, None); if p.fingerprint != "ALL_MISSING" { for part in p.fingerprint.split('|') { let _: f64 = part.parse().unwrap(); } } }
}
