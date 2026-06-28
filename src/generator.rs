//! 核心生成器 —— 从种子生成人格参数。
//!
//! 包含 Personality（生成结果）、Bias（偏向配置）、Generator（生成入口）。

use std::collections::HashMap;

use crate::params::{self, ParamDef, ALL_PARAMS};
use crate::seed::Seed;

// ═══════════════════════════════════════════════════════════════
// Personality —— 一次性生成结果
// ═══════════════════════════════════════════════════════════════

/// 一次人格生成的结果：84 个归一化值 [0,1] + 缺失标记 + 指纹。
#[derive(Debug, Clone)]
pub struct Personality {
    /// 84 个归一化值，范围 [0, 1]
    pub values: [f64; 84],
    /// 84 个缺失标记
    pub missing: [bool; 84],
    /// 指纹字符串
    pub fingerprint: String,
}

impl Personality {
    /// 内部构造：从原始值和缺失标记创建。
    pub(crate) fn new(values: [f64; 84], missing: [bool; 84]) -> Self {
        let fingerprint = Self::build_fingerprint(&values, &missing);
        Self { values, missing, fingerprint }
    }

    /// 根据参数 ID 获取归一化值 [0, 1]。
    pub fn get(&self, id: &str) -> f64 {
        params::param_index(id)
            .map(|i| self.values[i])
            .unwrap_or(0.5)
    }

    /// 缺失参数计数。
    pub fn missing_count(&self) -> usize {
        self.missing.iter().filter(|&&m| m).count()
    }

    fn build_fingerprint(values: &[f64; 84], missing: &[bool; 84]) -> String {
        let parts: Vec<String> = values
            .iter()
            .zip(missing.iter())
            .filter(|(_, &m)| !m)
            .take(4)
            .map(|(v, _)| format!("{:.4}", v))
            .collect();

        if parts.is_empty() {
            "ALL_MISSING".to_string()
        } else {
            parts.join("|")
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Bias —— 偏向配置
// ═══════════════════════════════════════════════════════════════

/// 偏向配置：可对特定参数或整个领域施加拉动力。
#[derive(Debug, Clone)]
pub struct Bias {
    /// 每个参数的偏向值 [-1, 1]
    biases: [f64; 84],
    /// 全局偏向强度 [0, 1]
    strength: f64,
}

impl Default for Bias {
    fn default() -> Self {
        Self {
            biases: [0.0; 84],
            strength: 0.7,
        }
    }
}

impl Bias {
    /// 创建空偏向。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置单个参数的偏向值。
    pub fn set(mut self, id: &str, v: f64) -> Self {
        if let Some(i) = params::param_index(id) {
            self.biases[i] = v.clamp(-1.0, 1.0);
        }
        self
    }

    /// 设置整个领域的偏向值。
    pub fn set_domain(mut self, domain: char, v: f64) -> Self {
        let v = v.clamp(-1.0, 1.0);
        for (i, p) in ALL_PARAMS.iter().enumerate() {
            if p.domain == domain {
                self.biases[i] = v;
            }
        }
        self
    }

    /// 设置全局偏向强度。
    pub fn set_strength(mut self, s: f64) -> Self {
        self.strength = s.clamp(0.0, 1.0);
        self
    }

    /// 从规范字符串解析偏向。
    ///
    /// 格式：`"B015=0.9,C031=-0.7,STRENGTH=0.5"` 或 `"A=0.3,B015=1.0"`
    pub fn parse(spec: &str) -> Self {
        let mut bias = Self::new();
        if spec.is_empty() {
            return bias;
        }

        for part in spec.split(&[',', ';'][..]) {
            let part = part.trim();
            let mut kv = part.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim().to_uppercase();
            let val_str = kv.next().unwrap_or("").trim();

            let val: f64 = match val_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            if key.len() == 1 {
                let ch = key.chars().next().unwrap();
                if ('A'..='H').contains(&ch) {
                    bias = bias.set_domain(ch, val);
                    continue;
                }
            }

            if key == "STRENGTH" || key == "S" || key == "STR" {
                bias = bias.set_strength(val);
            } else {
                bias = bias.set(&key, val);
            }
        }

        bias
    }

    /// 获取内部偏向数组的引用（供 Generator 使用）。
    pub(crate) fn biases(&self) -> &[f64; 84] {
        &self.biases
    }

    /// 获取偏向强度。
    pub fn strength(&self) -> f64 {
        self.strength
    }
}

// ═══════════════════════════════════════════════════════════════
// Generator —— 核心生成器
// ═══════════════════════════════════════════════════════════════

/// 人格生成器。唯一的公共入口。
pub struct Generator {
    #[allow(dead_code)]
    index_map: HashMap<&'static str, usize>,
}

impl Generator {
    /// 创建生成器实例，构建 ID→索引 查找表。
    pub fn new() -> Self {
        Self {
            index_map: params::build_index_map(),
        }
    }

    /// 批量生成人格。
    ///
    /// - `count`: 生成数量
    /// - `bias_spec`: 偏向规范字符串，如 `"B015=0.9,C031=-0.7"`
    pub fn generate(&self, count: usize, bias_spec: Option<&str>) -> Vec<Personality> {
        let count = count.max(1);
        let bias = Bias::parse(bias_spec.unwrap_or(""));

        // 用系统时间作为基础种子，确保每次调用产生不同结果
        #[cfg(not(target_arch = "wasm32"))]
        let base_seed = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos() as i32
        };
        #[cfg(target_arch = "wasm32")]
        let base_seed = {
            use std::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            COUNTER.fetch_add(1, Ordering::Relaxed) as i32
        };

        (0..count)
            .map(|i| self.generate_one(&mut Seed::from_int(base_seed + i as i32), &bias))
            .collect()
    }

    /// 从整数种子生成单个人格。
    pub fn generate_from_seed(&self, seed: i32, bias_spec: Option<&str>) -> Personality {
        let bias = Bias::parse(bias_spec.unwrap_or(""));
        self.generate_one(&mut Seed::from_int(seed), &bias)
    }

    /// 从十六进制种子生成单个人格。
    pub fn generate_from_hex(&self, hex: &str, bias_spec: Option<&str>) -> Result<Personality, String> {
        let seed = Seed::from_hex(hex)?;
        let bias = Bias::parse(bias_spec.unwrap_or(""));
        Ok(self.generate_one(&mut seed.clone(), &bias))
    }

    /// 核心生成逻辑：从种子流式读取，生成 84 个参数。
    fn generate_one(&self, seed: &mut Seed, bias: &Bias) -> Personality {
        seed.reset();
        let mut values = [0.5f64; 84];
        let mut missing = [false; 84];

        for i in 0..84 {
            let def = &ALL_PARAMS[i];

            // 缺失判断：15% 概率标记为缺失
            let is_missing = match seed.read_f32() {
                Ok(v) => v < 0.15,
                Err(_) => true,
            };

            if is_missing {
                // 缺失时使用默认值（中点）
                values[i] = if def.bipolar {
                    0.5 // 双极参数中点是 0.5（对应 raw=0）
                } else {
                    0.5 // 非双极参数中点也是 0.5
                };
                missing[i] = true;
            } else {
                // 生成随机数
                let rnd = match if def.bipolar {
                    seed.read_f64().map(|v| v * 2.0 - 1.0) // [-1, 1)
                } else {
                    seed.read_f64() // [0, 1)
                } {
                    Ok(v) => v,
                    Err(_) => {
                        values[i] = if def.bipolar { 0.5 } else { 0.5 };
                        missing[i] = true;
                        continue;
                    }
                };

                // 应用偏向拉力
                let rnd = apply_bias(rnd, def, bias, i);

                // 从归一化随机值计算原始值
                let raw = if def.bipolar {
                    (def.min + def.max) / 2.0 + rnd * (def.max - def.min) / 2.0
                } else {
                    def.min + rnd * (def.max - def.min)
                };

                // 裁剪到合法范围
                let raw = raw.clamp(def.min, def.max);

                // 归一化到 [0, 1]
                values[i] = if def.max > def.min {
                    (raw - def.min) / (def.max - def.min)
                } else {
                    0.5
                };
            }
        }

        Personality::new(values, missing)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

/// 对随机值应用偏向拉力。
///
/// 使用非线性拉力：有效拉力 = strength × √|bias|。
/// 平方根使极端值(1.0)拉力=1.0，温和值(0.25)拉力=0.5。
fn apply_bias(mut rnd: f64, def: &ParamDef, bias: &Bias, idx: usize) -> f64 {
    if bias.strength() <= 0.0 {
        return rnd;
    }

    let eb = bias.biases()[idx];
    if eb.abs() < 0.001 {
        return rnd;
    }

    let effective_pull = bias.strength() * eb.abs().sqrt();

    if def.bipolar {
        // 双极参数：目标就是偏向值本身
        rnd += (eb - rnd) * effective_pull;
        rnd.clamp(-1.0, 1.0)
    } else {
        // 非双极参数：eb>0 → 目标=1.0, eb<0 → 目标=0.0
        let target = if eb > 0.0 { 1.0 } else { 0.0 };
        rnd += (target - rnd) * effective_pull;
        rnd.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_deterministic() {
        let gen = Generator::new();
        let p1 = gen.generate_from_seed(42, None);
        let p2 = gen.generate_from_seed(42, None);
        // 相同种子应产生相同结果
        assert_eq!(p1.fingerprint, p2.fingerprint);
        assert_eq!(p1.values, p2.values);
    }

    #[test]
    fn test_generate_different_seeds() {
        let gen = Generator::new();
        let p1 = gen.generate_from_seed(1, None);
        let p2 = gen.generate_from_seed(2, None);
        // 不同种子大概率不同
        assert_ne!(p1.fingerprint, p2.fingerprint);
    }

    #[test]
    fn test_bias_parse() {
        let bias = Bias::parse("B015=0.9,C031=-0.7,STRENGTH=0.5");
        assert_eq!(bias.strength(), 0.5);
        // B015 是索引 14（0-based）
        let idx_b015 = params::param_index("B015").unwrap();
        assert!((bias.biases()[idx_b015] - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_bias_domain() {
        let bias = Bias::new().set_domain('A', 0.8);
        for (i, p) in ALL_PARAMS.iter().enumerate() {
            if p.domain == 'A' {
                assert!((bias.biases()[i] - 0.8).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_personality_get() {
        let gen = Generator::new();
        let p = gen.generate_from_seed(123, None);
        let v = p.get("A001");
        assert!((0.0..=1.0).contains(&v));
        assert_eq!(p.get("NONEXISTENT"), 0.5);
    }

    #[test]
    fn test_missing_count() {
        let gen = Generator::new();
        let p = gen.generate_from_seed(999, None);
        // 缺失应在 0~84 之间
        assert!(p.missing_count() <= 84);
    }
}
