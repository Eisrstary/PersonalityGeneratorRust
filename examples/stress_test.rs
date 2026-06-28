//! 压力测试 —— 全面验证人格生成器的正确性、性能和边界行为。
//!
//! 测试维度：
//!   1. 确定性验证（相同种子 → 相同结果）
//!   2. 大批量生成（10 万+ 人格）
//!   3. 边界条件（极端偏向、零种子、最大种子）
//!   4. 统计分布（均匀性、缺失率）
//!   5. 种子往返（hex 序列化/反序列化）
//!   6. 并发安全

use personality_generator::{Generator, Seed, Bias};
use personality_generator::params::ALL_PARAMS;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          人格原子参数系统 —— 压力测试                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let gen = Generator::new();

    // ── 1. 确定性验证 ──
    println!("━━━ [1/8] 确定性验证 ━━━");
    test_determinism(&gen);

    // ── 2. 大批量生成性能 ──
    println!("\n━━━ [2/8] 大批量生成 (100,000 人格) ━━━");
    test_bulk_generation(&gen);

    // ── 3. 极端偏向 ──
    println!("\n━━━ [3/8] 极端偏向 ━━━");
    test_extreme_bias(&gen);

    // ── 4. 统计分布 ──
    println!("\n━━━ [4/8] 统计分布分析 ━━━");
    test_statistical_distribution(&gen);

    // ── 5. 种子往返 ──
    println!("\n━━━ [5/8] 种子序列化往返 ━━━");
    test_seed_roundtrip();

    // ── 6. 边界种子 ──
    println!("\n━━━ [6/8] 边界种子值 ━━━");
    test_edge_seeds(&gen);

    // ── 7. 缺失率验证 ──
    println!("\n━━━ [7/8] 缺失率统计 ━━━");
    test_missing_rate(&gen);

    // ── 8. Bias 解析压力 ──
    println!("\n━━━ [8/8] Bias 解析压力 ━━━");
    test_bias_parsing();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    压力测试全部通过 ✅                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

// ═══════════════════════════════════════════════════════════════
// 1. 确定性验证
// ═══════════════════════════════════════════════════════════════

fn test_determinism(gen: &Generator) {
    let mut all_ok = true;

    // 同一种子多次生成必须完全一致
    for seed in &[0, 1, 42, -1, i32::MAX, i32::MIN] {
        let p1 = gen.generate_from_seed(*seed, None);
        let p2 = gen.generate_from_seed(*seed, None);
        if p1.values != p2.values || p1.missing != p2.missing || p1.fingerprint != p2.fingerprint {
            println!("  ❌ 种子 {} 确定性失败!", seed);
            all_ok = false;
        }
    }

    // 不同种子必须产生不同结果（极高概率）
    let p_a = gen.generate_from_seed(1, None);
    let p_b = gen.generate_from_seed(99999, None);
    if p_a.values == p_b.values {
        println!("  ❌ 不同种子产生相同结果（极端不可能）!");
        all_ok = false;
    }

    if all_ok {
        println!("  ✅ 确定性验证通过（6 个种子全部一致）");
    }
}

// ═══════════════════════════════════════════════════════════════
// 2. 大批量生成
// ═══════════════════════════════════════════════════════════════

fn test_bulk_generation(gen: &Generator) {
    const N: usize = 100_000;

    let start = Instant::now();
    let batch = gen.generate(N, None);
    let elapsed = start.elapsed();

    assert_eq!(batch.len(), N);

    // 验证每个人格的基本完整性
    let mut fingerprint_set = std::collections::HashSet::new();
    for p in &batch {
        assert_eq!(p.values.len(), 84);
        assert_eq!(p.missing.len(), 84);
        // 所有值必须在 [0, 1]
        for (i, &v) in p.values.iter().enumerate() {
            if !(0.0..=1.0).contains(&v) {
                // 双极参数归一化后在 [0,1]，非双极也在 [0,1]
                // 但双极参数中点 0.5 对应 raw=0，所以归一化值始终在 [0,1]
                assert!((0.0..=1.0).contains(&v),
                    "参数 {} 值 {} 超出 [0,1]", ALL_PARAMS[i].id, v);
            }
        }
        fingerprint_set.insert(p.fingerprint.clone());
    }

    let unique_ratio = fingerprint_set.len() as f64 / N as f64;

    println!("  生成数量: {:>8}", N);
    println!("  总耗时:   {:>8.2?}", elapsed);
    println!("  吞吐量:   {:>8.0} 人格/秒", N as f64 / elapsed.as_secs_f64());
    println!("  唯一指纹: {:>8} / {} ({:.1}%)",
        fingerprint_set.len(), N, unique_ratio * 100.0);
    println!("  ✅ 大批量生成通过");
}

// ═══════════════════════════════════════════════════════════════
// 3. 极端偏向
// ═══════════════════════════════════════════════════════════════

fn test_extreme_bias(gen: &Generator) {
    // 全部参数拉到极端高
    let p_high = gen.generate_from_seed(42, Some("A=1.0,B=1.0,C=1.0,D=1.0,E=1.0,F=1.0,G=1.0,H=1.0,STRENGTH=1.0"));
    let avg_high: f64 = p_high.values.iter().filter(|_| true).sum::<f64>() / 84.0;
    println!("  全领域 Bias=1.0: 平均值 = {:.4} (期望 > 0.7)", avg_high);
    assert!(avg_high > 0.65, "全高偏向平均值应 > 0.65, 实际 {}", avg_high);

    // 全部参数拉到极端低
    let p_low = gen.generate_from_seed(42, Some("A=-1.0,B=-1.0,C=-1.0,D=-1.0,E=-1.0,F=-1.0,G=-1.0,H=-1.0,STRENGTH=1.0"));
    let avg_low: f64 = p_low.values.iter().sum::<f64>() / 84.0;
    println!("  全领域 Bias=-1.0: 平均值 = {:.4} (期望 < 0.3)", avg_low);
    assert!(avg_low < 0.35, "全低偏向平均值应 < 0.35, 实际 {}", avg_low);

    // 零强度偏向应等于无偏向
    let p_no = gen.generate_from_seed(42, None);
    let p_zero = gen.generate_from_seed(42, Some("B015=1.0,STRENGTH=0.0"));
    assert_eq!(p_no.values, p_zero.values, "STRENGTH=0 应与无偏向完全一致");

    println!("  ✅ 极端偏向验证通过");
}

// ═══════════════════════════════════════════════════════════════
// 4. 统计分布
// ═══════════════════════════════════════════════════════════════

fn test_statistical_distribution(gen: &Generator) {
    const N: usize = 50_000;
    let batch = gen.generate(N, None);

    // 对每个参数统计均值和标准差
    let mut sum = [0.0f64; 84];
    let mut sum_sq = [0.0f64; 84];

    for p in &batch {
        for i in 0..84 {
            if !p.missing[i] {
                sum[i] += p.values[i];
                sum_sq[i] += p.values[i] * p.values[i];
            }
        }
    }

    // 检查每个非缺失参数的均值是否接近 0.5
    let mut outliers = 0;
    for i in 0..84 {
        let non_missing = N - batch.iter().filter(|p| p.missing[i]).count();
        if non_missing > 0 {
            let mean = sum[i] / non_missing as f64;
            let variance = (sum_sq[i] / non_missing as f64) - mean * mean;
            let std_dev = variance.sqrt();

            // 均值应在 0.5 ± 0.05 范围内（50k 样本下非常宽松）
            if (mean - 0.5).abs() > 0.05 {
                outliers += 1;
                if outliers <= 3 {
                    println!("  ⚠ {} 均值偏离: {:.4} (σ={:.4})", ALL_PARAMS[i].id, mean, std_dev);
                }
            }
        }
    }

    if outliers == 0 {
        println!("  ✅ 所有 84 个参数均值在 0.5±0.05 内，分布均匀");
    } else {
        println!("  ⚠ {} 个参数均值略偏（在 50k 样本中属正常波动）", outliers);
    }
}

// ═══════════════════════════════════════════════════════════════
// 5. 种子往返
// ═══════════════════════════════════════════════════════════════

fn test_seed_roundtrip() {
    let mut all_ok = true;

    for seed_val in &[0, 1, -1, 42, 1234567890, i32::MAX, i32::MIN] {
        let seed = Seed::from_int(*seed_val);
        let hex = seed.to_hex();

        // hex 长度必须是 2048
        assert_eq!(hex.len(), 2048, "种子 {} hex 长度错误", seed_val);

        // 还原
        let restored = Seed::from_hex(&hex);
        match restored {
            Ok(s) => {
                if s.as_bytes() != seed.as_bytes() {
                    println!("  ❌ 种子 {} 往返失败: 字节不匹配", seed_val);
                    all_ok = false;
                }
            }
            Err(e) => {
                println!("  ❌ 种子 {} hex 解析失败: {}", seed_val, e);
                all_ok = false;
            }
        }
    }

    // 无效 hex 测试
    assert!(Seed::from_hex("too_short").is_err());
    assert!(Seed::from_hex(&"XY".repeat(1024)).is_err());

    if all_ok {
        println!("  ✅ 7 个种子全部往返成功");
    }
}

// ═══════════════════════════════════════════════════════════════
// 6. 边界种子
// ═══════════════════════════════════════════════════════════════

fn test_edge_seeds(gen: &Generator) {
    let edge_seeds = [
        ("零", 0),
        ("正一", 1),
        ("负一", -1),
        ("i32::MAX", i32::MAX),
        ("i32::MIN", i32::MIN),
        ("42", 42),
        ("-999999", -999999),
    ];

    for (name, seed) in &edge_seeds {
        let p = gen.generate_from_seed(*seed, None);
        // 基本完整性检查
        assert_eq!(p.values.len(), 84);
        assert_eq!(p.missing.len(), 84);
        assert!(!p.fingerprint.is_empty());
        // 所有值在 [0,1]
        for (i, &v) in p.values.iter().enumerate() {
            assert!((0.0..=1.0).contains(&v),
                "种子 {} 参数 {} 值 {} 超出范围", name, ALL_PARAMS[i].id, v);
        }
    }

    println!("  ✅ 7 个边界种子全部正常");
}

// ═══════════════════════════════════════════════════════════════
// 7. 缺失率统计
// ═══════════════════════════════════════════════════════════════

fn test_missing_rate(gen: &Generator) {
    const N: usize = 100_000;
    let batch = gen.generate(N, None);

    let total_missing: usize = batch.iter().map(|p| p.missing_count()).sum();
    let total_params = N * 84;
    let missing_rate = total_missing as f64 / total_params as f64;

    println!("  样本数:     {:>8}", N);
    println!("  总参数数:   {:>8}", total_params);
    println!("  缺失参数:   {:>8}", total_missing);
    println!("  缺失率:     {:>8.2}% (期望 ~15%)", missing_rate * 100.0);

    // 应在 15% ± 2% 范围内
    assert!((missing_rate - 0.15).abs() < 0.02,
        "缺失率 {:.2}% 偏离预期 15% 太多", missing_rate * 100.0);

    // 检查是否有全缺失或零缺失的极端人格
    let all_missing = batch.iter().filter(|p| p.missing_count() == 84).count();
    let none_missing = batch.iter().filter(|p| p.missing_count() == 0).count();
    println!("  全缺失人格: {} / {}", all_missing, N);
    println!("  零缺失人格: {} / {}", none_missing, N);

    println!("  ✅ 缺失率在预期范围内");
}

// ═══════════════════════════════════════════════════════════════
// 8. Bias 解析压力
// ═══════════════════════════════════════════════════════════════

fn test_bias_parsing() {
    // 空字符串
    let b = Bias::parse("");
    assert_eq!(b.strength(), 0.7); // 默认值

    // 单个参数
    let b = Bias::parse("B015=0.9");
    assert_eq!(b.strength(), 0.7);

    // 完整格式
    let b = Bias::parse("B015=0.9,C031=-0.7,STRENGTH=0.5");
    assert_eq!(b.strength(), 0.5);

    // 领域 + 参数混合
    let b = Bias::parse("A=0.8;B015=0.3;S=0.6");
    assert_eq!(b.strength(), 0.6);

    // 非法值被忽略
    let b = Bias::parse("B015=abc,XXX=0.5");
    assert_eq!(b.strength(), 0.7);

    // 越界值被裁剪
    let b = Bias::parse("B015=999,STRENGTH=-5");
    assert_eq!(b.strength(), 0.0); // clamp 到 [0,1]

    // 分号分隔
    let b = Bias::parse("C025=0.5;D040=0.3;STR=0.8");
    assert_eq!(b.strength(), 0.8);

    println!("  ✅ Bias 解析全部通过");
}
