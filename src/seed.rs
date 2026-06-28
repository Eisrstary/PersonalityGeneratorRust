//! 确定性熵源 —— 将整数种子展开为 1024 字节确定性字节流。
//!
//! # 架构
//!
//! - `EntropySource` trait：可插拔的熵源抽象
//! - `XorShift128Plus`：xorshift128+ 实现（周期 2¹²⁸-1）
//! - `Seed`：1024 字节确定性种子，流式读取 f64/f32/bit
//!
//! 每个参数消耗：1×f32（缺失判断 4B）+ 1×f64（随机值 8B）= 12B。
//! 84 参数 × 12B = 1008B < 1024B，容量充足。

use core::fmt;

// ═══════════════════════════════════════════════════════════════
// EntropySource trait
// ═══════════════════════════════════════════════════════════════

/// 确定性伪随机数生成器的统一接口。
/// 可替换实现（如 ChaCha20）而不影响下游代码。
pub trait EntropySource {
    fn next_u64(&mut self) -> u64;

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_exact_mut(8) {
            chunk.copy_from_slice(&self.next_u64().to_le_bytes());
        }
        let rem = buf.len() % 8;
        if rem > 0 {
            let start = buf.len() - rem;
            buf[start..].copy_from_slice(&self.next_u64().to_le_bytes()[..rem]);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// XorShift128Plus
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct XorShift128Plus {
    state: [u64; 2],
}

impl XorShift128Plus {
    pub const fn from_seed_pair(a: u64, b: u64) -> Self {
        Self {
            state: [
                if a == 0 { 0xDEAD_BEEF_1234_5678 } else { a },
                if b == 0 { 0xABCD_EF09_8765_4321 } else { b },
            ],
        }
    }

    pub fn from_i32(seed: i32) -> Self {
        let s = seed as u64;
        Self::from_seed_pair(
            s.wrapping_mul(0x5851_F42D_4C95_7F2D),
            s ^ 0x9E37_79B9_7F4A_7C15,
        )
    }
}

impl EntropySource for XorShift128Plus {
    fn next_u64(&mut self) -> u64 {
        let [s0, s1] = self.state;
        self.state[0] = s1;
        let t = s0 ^ (s0 << 23);
        self.state[1] = t ^ s1 ^ (t >> 18) ^ (s1 >> 5);
        self.state[1].wrapping_add(s1)
    }
}

// ═══════════════════════════════════════════════════════════════
// Seed
// ═══════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct Seed {
    data: [u8; Self::SIZE],
    pos: u16,
    bit: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedExhausted;

impl fmt::Display for SeedExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "种子已耗尽")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedError {
    InvalidHexLength(usize),
    InvalidHexChar(char),
}

impl fmt::Display for SeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexLength(n) => write!(f, "hex 长度必须为 {}，实际 {n}", Seed::SIZE * 2),
            Self::InvalidHexChar(c) => write!(f, "无效 hex 字符: '{c}'"),
        }
    }
}

impl Seed {
    pub const SIZE: usize = 1024;

    /// 从 i32 种子展开为 1024 字节。
    pub fn from_i32(seed: i32) -> Self {
        let mut data = [0u8; Self::SIZE];
        XorShift128Plus::from_i32(seed).fill_bytes(&mut data);
        Self { data, pos: 0, bit: 0 }
    }

    /// 从原始字节创建（不展开）。
    pub fn from_raw(data: [u8; Self::SIZE]) -> Self {
        Self { data, pos: 0, bit: 0 }
    }

    /// 从 hex 字符串反序列化。
    pub fn from_hex(hex: &str) -> Result<Self, SeedError> {
        let hex = hex.as_bytes();
        if hex.len() != Self::SIZE * 2 {
            return Err(SeedError::InvalidHexLength(hex.len()));
        }
        let mut data = [0u8; Self::SIZE];
        for (i, chunk) in hex.chunks(2).enumerate() {
            if chunk.len() < 2 {
                return Err(SeedError::InvalidHexLength(hex.len()));
            }
            let hi = hex_nibble(chunk[0]).ok_or(SeedError::InvalidHexChar(chunk[0] as char))?;
            let lo = hex_nibble(chunk[1]).ok_or(SeedError::InvalidHexChar(chunk[1] as char))?;
            data[i] = (hi << 4) | lo;
        }
        Ok(Self { data, pos: 0, bit: 0 })
    }

    pub fn reset(&mut self) { self.pos = 0; self.bit = 0; }

    pub fn read_f64(&mut self) -> Result<f64, SeedExhausted> {
        let p = self.pos as usize;
        if p + 8 > Self::SIZE { return Err(SeedExhausted); }
        let raw = u64::from_le_bytes(self.data[p..p + 8].try_into().unwrap());
        self.pos += 8;
        Ok((raw >> 11) as f64 / (1u64 << 53) as f64)
    }

    pub fn read_f32(&mut self) -> Result<f32, SeedExhausted> {
        let p = self.pos as usize;
        if p + 4 > Self::SIZE { return Err(SeedExhausted); }
        let raw = u32::from_le_bytes(self.data[p..p + 4].try_into().unwrap());
        self.pos += 4;
        // 用 f64 做除法避免精度损失，然后转为 f32
        Ok((raw as f64 / u32::MAX as f64) as f32)
    }

    pub fn read_bit(&mut self) -> Result<bool, SeedExhausted> {
        let p = self.pos as usize;
        if p >= Self::SIZE { return Err(SeedExhausted); }
        let bit = (self.data[p] >> self.bit) & 1;
        self.bit += 1;
        if self.bit >= 8 { self.bit = 0; self.pos += 1; }
        Ok(bit == 1)
    }

    pub fn as_bytes(&self) -> &[u8; Self::SIZE] { &self.data }
    pub fn position(&self) -> usize { self.pos as usize }
    pub fn remaining(&self) -> usize { Self::SIZE.saturating_sub(self.pos as usize) }
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Seed(pos={})", self.pos)
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.data { write!(f, "{b:02X}")?; }
        Ok(())
    }
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

/// 生成非确定性随机种子（混合 nanosecond + microsecond 精度）。
#[cfg(not(target_arch = "wasm32"))]
pub fn random_seed() -> Seed {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    // 混合秒、毫秒、纳秒，产生更好的熵分布
    let mixed = (d.as_secs() as u32)
        .wrapping_mul(1_103_515_245)
        .wrapping_add(d.subsec_millis())
        .wrapping_mul(1_103_515_245)
        .wrapping_add(d.subsec_nanos());
    Seed::from_i32(mixed as i32)
}

#[cfg(target_arch = "wasm32")]
pub fn random_seed() -> Seed {
    use core::sync::atomic::{AtomicU32, Ordering};
    static C: AtomicU32 = AtomicU32::new(0);
    Seed::from_i32(C.fetch_add(1, Ordering::Relaxed).wrapping_mul(1_103_515_245) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = XorShift128Plus::from_seed_pair(42, 99);
        let mut b = XorShift128Plus::from_seed_pair(42, 99);
        for _ in 0..100 { assert_eq!(a.next_u64(), b.next_u64()); }
    }

    #[test]
    fn seed_hex_roundtrip() {
        let s = Seed::from_i32(12345);
        let r = Seed::from_hex(&s.to_string()).unwrap();
        assert_eq!(s.as_bytes(), r.as_bytes());
    }

    #[test]
    fn seed_exhausted() {
        let mut s = Seed::from_i32(1);
        for _ in 0..128 { s.read_f64().ok(); }
        assert!(s.read_f64().is_err());
    }

    #[test]
    fn seed_reproducible() {
        let mut s = Seed::from_i32(42);
        let a = s.read_f64().unwrap();
        s.reset();
        assert!((s.read_f64().unwrap() - a).abs() < 1e-15);
    }
}
