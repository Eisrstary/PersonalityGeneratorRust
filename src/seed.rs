//! 确定性伪随机数生成器与种子系统。
//!
//! 基于 xorshift128+ 算法，无外部依赖，适用于所有平台（含 WASM）。
//! 1024 字节确定性种子 → 驱动 84 个参数生成。

// ═══════════════════════════════════════════════════════════════
// XorShift128+ 随机数生成器
// ═══════════════════════════════════════════════════════════════

/// 基于 xorshift128+ 的确定性伪随机数生成器。
#[derive(Debug, Clone)]
pub struct XorShiftRng {
    s0: u64,
    s1: u64,
}

impl XorShiftRng {
    /// 用两个 u64 种子创建生成器。零种子会被替换为默认值。
    pub fn new(seed0: u64, seed1: u64) -> Self {
        Self {
            s0: if seed0 == 0 { 0xDEADBEEF12345678 } else { seed0 },
            s1: if seed1 == 0 { 0xABCDEF0987654321 } else { seed1 },
        }
    }

    /// 生成下一个 u64。
    pub fn next_u64(&mut self) -> u64 {
        let s1 = self.s0;
        let s0 = self.s1;
        self.s0 = s0;
        let s1 = s1 ^ (s1 << 23);
        self.s1 = s1 ^ s0 ^ (s1 >> 18) ^ (s0 >> 5);
        self.s1.wrapping_add(s0)
    }

    /// 生成 [0, 1) 区间的 f64，精度 53 位。
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// 生成 [0, 1) 区间的 f32，精度 24 位。
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }
}

// ═══════════════════════════════════════════════════════════════
// 种子扩展器
// ═══════════════════════════════════════════════════════════════

/// 将单个 i32 种子确定性扩展为指定长度的字节数组。
/// 使用 xorshift128+ 替代 SHA256，适用于所有平台。
pub fn expand_seed(seed: i32, target_len: usize) -> Vec<u8> {
    let mut result = vec![0u8; target_len];
    let mut rng = XorShiftRng::new(
        (seed as u64).wrapping_mul(6364136223846793005),
        (seed as u64) ^ 0x9E3779B97F4A7C15,
    );

    for chunk in result.chunks_exact_mut(8) {
        let value = rng.next_u64();
        chunk.copy_from_slice(&value.to_le_bytes());
    }

    // 处理尾部不足 8 字节的部分
    let remainder_start = (target_len / 8) * 8;
    if remainder_start < target_len {
        let value = rng.next_u64();
        let bytes = value.to_le_bytes();
        let remaining = target_len - remainder_start;
        result[remainder_start..].copy_from_slice(&bytes[..remaining]);
    }

    result
}

// ═══════════════════════════════════════════════════════════════
// Seed —— 1024 字节确定性种子
// ═══════════════════════════════════════════════════════════════

/// 人格种子：1024 字节确定性数据，驱动全部 84 个参数生成。
#[derive(Debug, Clone)]
pub struct Seed {
    data: [u8; Self::LEN],
    pos: usize,
    bit_off: u8,
}

impl Seed {
    /// 种子固定长度。
    pub const LEN: usize = 1024;

    /// 从字节数组创建种子。
    pub fn from_bytes(data: [u8; Self::LEN]) -> Self {
        Self { data, pos: 0, bit_off: 0 }
    }

    /// 从 i32 整数种子创建。
    pub fn from_int(seed: i32) -> Self {
        let expanded = expand_seed(seed, Self::LEN);
        let mut arr = [0u8; Self::LEN];
        arr.copy_from_slice(&expanded);
        Self { data: arr, pos: 0, bit_off: 0 }
    }

    /// 从十六进制字符串创建。字符串长度必须为 2048（= 1024×2）。
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        if hex.len() != Self::LEN * 2 {
            return Err(format!("十六进制字符串必须为 {} 个字符", Self::LEN * 2));
        }
        let mut data = [0u8; Self::LEN];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let high = hex_char_to_nibble(chunk[0])?;
            let low = hex_char_to_nibble(chunk[1])?;
            data[i] = (high << 4) | low;
        }
        Ok(Self { data, pos: 0, bit_off: 0 })
    }

    /// 用系统时间生成随机种子。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as i32;
        let seed = tick ^ (tick << 13) ^ (tick.wrapping_mul(1103515245));
        Self::from_int(seed)
    }

    /// WASM 兼容的随机种子生成。
    #[cfg(target_arch = "wasm32")]
    pub fn random() -> Self {
        // WASM 环境下使用固定种子 + 调用次数递增
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let seed = COUNTER.fetch_add(1, Ordering::Relaxed) as i32;
        Self::from_int(seed.wrapping_mul(1103515245))
    }

    /// 重置读取位置。
    pub fn reset(&mut self) {
        self.pos = 0;
        self.bit_off = 0;
    }

    /// 读取一个 f64（消耗 8 字节），范围 [0, 1)。
    pub fn read_f64(&mut self) -> Result<f64, &'static str> {
        if self.pos + 8 > Self::LEN {
            return Err("种子已耗尽");
        }
        let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
        let v = u64::from_le_bytes(bytes);
        self.pos += 8;
        Ok((v >> 11) as f64 / (1u64 << 53) as f64)
    }

    /// 读取一个 f32（消耗 4 字节），范围 [0, 1)。
    pub fn read_f32(&mut self) -> Result<f32, &'static str> {
        if self.pos + 4 > Self::LEN {
            return Err("种子已耗尽");
        }
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
        let v = u32::from_le_bytes(bytes);
        self.pos += 4;
        Ok(v as f32 / u32::MAX as f32)
    }

    /// 读取一个 bit。
    pub fn read_bit(&mut self) -> Result<bool, &'static str> {
        if self.pos >= Self::LEN {
            return Err("种子已耗尽");
        }
        let bit = ((self.data[self.pos] >> self.bit_off) & 1) == 1;
        self.bit_off += 1;
        if self.bit_off >= 8 {
            self.bit_off = 0;
            self.pos += 1;
        }
        Ok(bit)
    }

    /// 获取原始字节数据。
    pub fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.data
    }

    /// 转为十六进制字符串。
    pub fn to_hex(&self) -> String {
        self.data.iter().map(|b| format!("{:02X}", b)).collect()
    }
}

fn hex_char_to_nibble(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(format!("无效的十六进制字符: {}", c as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xorshift_deterministic() {
        let mut rng1 = XorShiftRng::new(42, 99);
        let mut rng2 = XorShiftRng::new(42, 99);
        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_seed_roundtrip() {
        let seed = Seed::from_int(12345);
        let hex = seed.to_hex();
        let restored = Seed::from_hex(&hex).unwrap();
        assert_eq!(seed.as_bytes(), restored.as_bytes());
    }

    #[test]
    fn test_seed_read_order() {
        let mut seed = Seed::from_int(42);
        let a = seed.read_f64().unwrap();
        let b = seed.read_f32().unwrap();
        seed.reset();
        assert!((seed.read_f64().unwrap() - a).abs() < 1e-15);
        assert!((seed.read_f32().unwrap() - b).abs() < 1e-7);
    }
}
