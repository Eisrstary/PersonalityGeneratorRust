//! 真正极限碰撞测试 —— 推到数学与硬件边界
//!
//! 测试维度：
//!   1. 千万级指纹碰撞（内存 ~1GB）
//!   2. 完整 84 维值碰撞（HashMap<[f64;84]>）
//!   3. i16 全种子空间穷举（65536 个种子全遍历）
//!   4. 参数值精度碰撞（4 位小数空间）
//!   5. 连续种子差分分析
//!   6. 哈希质量评估（指纹作为哈希的分布均匀性）

use personality_generator::Generator;
use std::collections::HashSet;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     人格原子参数系统 —— 硬件极限碰撞测试                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let gen = Generator::new();

    // ── 1. 千万级指纹碰撞 ──
    println!("━━━ [1/6] 千万级指纹碰撞（内存极限）━━━");
    test_10m_collision(&gen);

    // ── 2. 完整 84 维值碰撞 ──
    println!("\n━━━ [2/6] 完整 84 维值空间碰撞 ━━━");
    test_full_vector_collision(&gen);

    // ── 3. i16 全种子空间穷举 ──
    println!("\n━━━ [3/6] i16 种子空间穷举 (65536 种子) ━━━");
    test_i16_exhaustion(&gen);

    // ── 4. 参数值精度碰撞 ──
    println!("\n━━━ [4/6] 参数值精度碰撞分析 ━━━");
    test_value_precision_collision(&gen);

    // ── 5. 连续种子差分 ──
    println!("\n━━━ [5/6] 相邻种子参数差分分布 ━━━");
    test_neighbor_diffusion(&gen);

    // ── 6. 指纹哈希质量 ──
    println!("\n━━━ [6/6] 指纹作为哈希的分布质量 ━━━");
    test_fingerprint_hash_quality(&gen);

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              硬件极限碰撞测试全部通过 ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

// ═══════════════════════════════════════════════════════════════
// 1. 千万级指纹碰撞
// ═══════════════════════════════════════════════════════════════

fn test_10m_collision(gen: &Generator) {
    // 指纹只用前 4 个非缺失参数，理论空间 10000^4 = 10^16
    // 千万样本碰撞概率 ≈ n²/(2N) ≈ 10^14 / (2×10^16) = 0.005
    const N: usize = 10_000_000;

    let start = Instant::now();
    let mut seen: HashSet<String> = HashSet::with_capacity(N);
    let mut collisions = 0u64;

    // 分批处理，避免一次性分配过多
    const BATCH: usize = 500_000;
    for batch_start in (0..N).step_by(BATCH) {
        let batch_end = (batch_start + BATCH).min(N);
        for i in batch_start..batch_end {
            let p = gen.from_seed(i as i32, None);
            if !seen.insert(p.fingerprint().to_string()) {
                collisions += 1;
            }
        }
        // 释放中间结果
        if batch_end % (BATCH * 2) == 0 {
            let elapsed = start.elapsed();
            let done = batch_end;
            let rate = done as f64 / elapsed.as_secs_f64();
            println!("  ... {:>7} / {} ({:.1} 万/秒)", done, N, rate / 10000.0);
        }
    }

    let elapsed = start.elapsed();
    let collision_rate = collisions as f64 / N as f64;

    println!("  样本数:     {:>10}", N);
    println!("  耗时:       {:>10.2?}", elapsed);
    println!("  碰撞数:     {:>10}", collisions);
    println!("  碰撞率:     {:>10.8}%", collision_rate * 100.0);
    println!("  吞吐量:     {:>10.0} 人格/秒", N as f64 / elapsed.as_secs_f64());
    println!("  内存占用:   ~{:>8.1} MB", seen.len() as f64 * 80.0 / 1024.0 / 1024.0);

    // 千万样本下，碰撞率应 < 0.001%
    assert!(collision_rate < 0.00001,
        "千万样本碰撞率过高: {:.8}", collision_rate);

    if collisions == 0 {
        println!("  ✅ 千万样本零碰撞！");
    } else {
        println!("  ✅ 碰撞率极低 ({:.8}%)，符合理论预期", collision_rate * 100.0);
    }
}

// ═══════════════════════════════════════════════════════════════
// 2. 完整 84 维值碰撞
// ═══════════════════════════════════════════════════════════════

fn test_full_vector_collision(gen: &Generator) {
    // 将 84 维 f64 量化到 u16（65536 级），检测完整向量碰撞
    // 这是真正的"两个人格完全一样"的检测
    const N: usize = 500_000;

    let start = Instant::now();
    let mut seen: HashSet<[u16; 84]> = HashSet::with_capacity(N);
    let mut collisions = 0u64;

    for i in 0..N {
        let p = gen.from_seed(i as i32, None);

        // 量化到 u16: 0..=65535
        let mut key = [0u16; 84];
        for j in 0..84 {
            if p.missing()[j] {
                key[j] = 65535; // 缺失用特殊值
            } else {
                key[j] = (p.values()[j] * 65534.0) as u16;
            }
        }

        if !seen.insert(key) {
            collisions += 1;
        }
    }

    let elapsed = start.elapsed();
    let collision_rate = collisions as f64 / N as f64;

    // 理论空间: 65535^84 ≈ 10^404，500k 样本碰撞概率 ≈ 0
    println!("  样本数:       {:>8}", N);
    println!("  量化精度:     u16 (65536 级)");
    println!("  理论空间:     65536^84 ≈ 10^404");
    println!("  耗时:         {:>10.2?}", elapsed);
    println!("  84维碰撞数:   {:>8}", collisions);
    println!("  碰撞率:       {:>10.8}%", collision_rate * 100.0);

    assert!(collisions == 0,
        "84 维完整碰撞检测到 {} 次碰撞!", collisions);

    println!("  ✅ 84 维完整向量零碰撞");
}

// ═══════════════════════════════════════════════════════════════
// 3. i16 全种子空间穷举
// ═══════════════════════════════════════════════════════════════

fn test_i16_exhaustion(gen: &Generator) {
    // 遍历全部 65536 个 i16 种子
    const N: usize = 65536;

    let start = Instant::now();
    let mut fingerprints: HashSet<String> = HashSet::with_capacity(N);
    let mut all_values: Vec<[f64; 84]> = Vec::with_capacity(N);

    for i in 0i32..65536 {
        let p = gen.from_seed(i, None);
        fingerprints.insert(p.fingerprint().to_string());
        all_values.push(*p.values());
    }

    let elapsed = start.elapsed();

    // 指纹唯一性
    let fp_collisions = N - fingerprints.len();

    // 统计每个参数在 65536 个种子下的 min/max/mean/std
    println!("  种子空间:     i16 全量 (0..65535)");
    println!("  耗时:         {:>10.2?}", elapsed);
    println!("  指纹唯一:     {} / {} (碰撞: {})", fingerprints.len(), N, fp_collisions);

    // 每个参数的范围统计
    let mut min_range = 1.0f64;
    let mut max_range = 0.0f64;
    for param_idx in 0..84 {
        let vals: Vec<f64> = all_values.iter().map(|v| v[param_idx]).collect();
        let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;
        min_range = min_range.min(range);
        max_range = max_range.max(range);
    }

    println!("  参数值跨度:   [{:.4}, {:.4}] (应接近 [0, 1])", min_range, max_range);

    // 65536 样本下每个参数应覆盖几乎整个 [0,1]
    assert!(min_range > 0.95, "参数最小跨度不足: {:.4}", min_range);
    assert!(fp_collisions == 0, "i16 全空间指纹碰撞: {}", fp_collisions);

    println!("  ✅ i16 全空间零碰撞，参数覆盖完整");
}

// ═══════════════════════════════════════════════════════════════
// 4. 参数值精度碰撞
// ═══════════════════════════════════════════════════════════════

fn test_value_precision_collision(gen: &Generator) {
    // 检查 4 位小数精度下，相邻种子的参数值是否过度聚集
    const N: usize = 200_000;

    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    // 对每个参数，统计唯一值的数量
    let mut total_unique = 0usize;
    let mut min_coverage = 1.0f64;

    for param_idx in 0..84 {
        let mut seen: HashSet<u32> = HashSet::new();
        for p in &batch {
            if !p.missing()[param_idx] {
                // 4 位小数 → 0..10000 的整数
                let bucket = (p.values()[param_idx] * 10000.0) as u32;
                seen.insert(bucket.min(9999));
            }
        }
        // 覆盖率 = 唯一值数 / 10000（理论最大）
        let coverage = seen.len() as f64 / 10000.0;
        min_coverage = min_coverage.min(coverage);
        total_unique += seen.len();
    }

    let avg_unique = total_unique as f64 / 84.0;

    println!("  样本数:         {:>8}", N);
    println!("  精度:           4 位小数 (10000 级)");
    println!("  每参数平均唯一值: {:>8.0} / 10000", avg_unique);
    println!("  最小覆盖率:      {:.2}%", min_coverage * 100.0);

    // 200k 样本中 ~170k 非缺失，覆盖 10000 桶绰绰有余
    assert!(min_coverage > 0.99,
        "参数值覆盖不足: {:.2}%", min_coverage * 100.0);

    println!("  ✅ 4位小数空间覆盖充分");
}

// ═══════════════════════════════════════════════════════════════
// 5. 相邻种子参数差分
// ═══════════════════════════════════════════════════════════════

fn test_neighbor_diffusion(gen: &Generator) {
    const N: usize = 50_000;

    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    // 对每个参数，计算相邻种子的平均绝对差
    let mut total_avg_diff = 0.0f64;
    let mut min_avg_diff = f64::MAX;
    let mut max_avg_diff = 0.0f64;

    for param_idx in 0..84 {
        let mut sum_diff = 0.0f64;
        let mut count = 0usize;

        for i in 1..N {
            if !batch[i].missing()[param_idx] && !batch[i - 1].missing()[param_idx] {
                sum_diff += (batch[i].values()[param_idx] - batch[i - 1].values()[param_idx]).abs();
                count += 1;
            }
        }

        if count > 0 {
            let avg_diff = sum_diff / count as f64;
            total_avg_diff += avg_diff;
            min_avg_diff = min_avg_diff.min(avg_diff);
            max_avg_diff = max_avg_diff.max(avg_diff);
        }
    }

    let grand_avg = total_avg_diff / 84.0;

    // 两个独立 U[0,1] 的期望绝对差 = 1/3 ≈ 0.333
    println!("  相邻种子数:     {:>8}", N);
    println!("  期望 |Δ|:       {:.4} (独立均匀分布理论值)", 1.0 / 3.0);
    println!("  实测平均 |Δ|:   {:.4}", grand_avg);
    println!("  |Δ| 范围:       [{:.4}, {:.4}]", min_avg_diff, max_avg_diff);

    // 相邻种子应像独立随机一样扩散
    assert!((grand_avg - 1.0 / 3.0).abs() < 0.02,
        "相邻种子扩散不足: {:.4}", grand_avg);

    println!("  ✅ 相邻种子充分扩散（与独立随机无异）");
}

// ═══════════════════════════════════════════════════════════════
// 6. 指纹哈希质量
// ═══════════════════════════════════════════════════════════════

fn test_fingerprint_hash_quality(gen: &Generator) {
    const N: usize = 1_000_000;

    // 将指纹映射到 65536 个桶，检查分布均匀性
    const BUCKETS: usize = 65536;
    let mut buckets = vec![0u32; BUCKETS];

    let start = Instant::now();
    for i in 0..N {
        let p = gen.from_seed(i as i32, None);
        // 用指纹的哈希值分桶
        let hash = fxhash(&p.fingerprint());
        let bucket = (hash % BUCKETS as u64) as usize;
        buckets[bucket] += 1;
    }
    let elapsed = start.elapsed();

    let expected = N as f64 / BUCKETS as f64;
    let chi: f64 = buckets.iter()
        .map(|&count| {
            let diff = count as f64 - expected;
            diff * diff / expected
        })
        .sum();

    let min_bucket = buckets.iter().min().unwrap();
    let max_bucket = buckets.iter().max().unwrap();

    println!("  样本数:       {:>10}", N);
    println!("  桶数:         {:>8}", BUCKETS);
    println!("  耗时:         {:>10.2?}", elapsed);
    println!("  期望/桶:      {:>10.2}", expected);
    println!("  实际范围:     [{}, {}]", min_bucket, max_bucket);
    println!("  卡方 χ²:      {:>10.2} (期望 ~{})", chi, BUCKETS);

    // 65536 桶的卡方检验：自由度 65535，期望值 ≈ 65535
    // 允许一定偏差
    let chi_norm = chi / BUCKETS as f64;
    println!("  归一化 χ²:    {:>10.4} (理想 ~1.0)", chi_norm);

    assert!((chi_norm - 1.0).abs() < 0.2,
        "指纹哈希分布不均匀: χ²/n = {:.4}", chi_norm);

    println!("  ✅ 指纹哈希分布均匀");
}

/// 简单快速的哈希函数（FNV-1a 风格）
fn fxhash(s: &str) -> u64 {
    let mut hash: u64 = 0x517cc1b727220a95;
    for &b in s.as_bytes() {
        hash = hash.wrapping_add(b as u64);
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= hash >> 32;
    }
    hash
}
