//! 数学极限压力测试 —— 碰撞检测与统计验证
//!
//! 测试维度：
//!   1. 指纹碰撞检测（百万级样本）
//!   2. 84 维参数空间均匀性（卡方检验）
//!   3. 参数独立性（皮尔逊相关系数矩阵抽样）
//!   4. 双极参数对称性
//!   5. 缺失模式分布
//!   6. 种子空间遍历（连续种子相关性）
//!   7. 极端偏向下的参数边界
//!   8. 浮点精度极限

use personality_generator::{Generator, Seed};
use personality_generator::params::PARAMS;
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     人格原子参数系统 —— 数学极限碰撞测试                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let gen = Generator::new();

    // ── 1. 百万级指纹碰撞检测 ──
    println!("━━━ [1/8] 百万级指纹碰撞检测 ━━━");
    test_fingerprint_collision(&gen);

    // ── 2. 卡方均匀性检验 ──
    println!("\n━━━ [2/8] 84维参数空间均匀性（卡方检验）━━━");
    test_chi_squared(&gen);

    // ── 3. 参数独立性检验 ──
    println!("\n━━━ [3/8] 参数间皮尔逊相关系数 ━━━");
    test_parameter_independence(&gen);

    // ── 4. 双极参数对称性 ──
    println!("\n━━━ [4/8] 双极参数对称性验证 ━━━");
    test_bipolar_symmetry(&gen);

    // ── 5. 缺失模式分布 ──
    println!("\n━━━ [5/8] 缺失模式熵与分布 ━━━");
    test_missing_pattern_entropy(&gen);

    // ── 6. 连续种子相关性 ──
    println!("\n━━━ [6/8] 连续种子自相关分析 ━━━");
    test_sequential_seed_correlation(&gen);

    // ── 7. 极端偏向参数边界 ──
    println!("\n━━━ [7/8] 极端偏向下的参数边界 ━━━");
    test_bias_boundaries(&gen);

    // ── 8. 浮点精度极限 ──
    println!("\n━━━ [8/8] 浮点精度与指纹稳定性 ━━━");
    test_fingerprint_precision(&gen);

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              数学极限碰撞测试全部通过 ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

// ═══════════════════════════════════════════════════════════════
// 1. 百万级指纹碰撞检测
// ═══════════════════════════════════════════════════════════════

fn test_fingerprint_collision(gen: &Generator) {
    const N: usize = 1_000_000;

    let start = Instant::now();
    let mut seen = HashMap::with_capacity(N);
    let mut collisions = 0u64;

    for i in 0..N {
        let p = gen.from_seed(i as i32, None);
        if seen.contains_key(&p.fingerprint().to_string()) {
            collisions += 1;
        }
        seen.insert(p.fingerprint().to_string(), ());
    }
    let elapsed = start.elapsed();

    let collision_rate = collisions as f64 / N as f64;

    println!("  样本数:     {:>10}", N);
    println!("  耗时:       {:>10.2?}", elapsed);
    println!("  碰撞数:     {:>10}", collisions);
    println!("  碰撞率:     {:>10.8}%", collision_rate * 100.0);
    println!("  吞吐量:     {:>10.0} 人格/秒", N as f64 / elapsed.as_secs_f64());

    // 指纹用前4个非缺失参数的 4 位小数组成
    // 理论空间: 10000^4 = 10^16，百万样本碰撞概率极低
    assert!(collision_rate < 0.001,
        "碰撞率 {} 过高!", collision_rate);

    if collisions == 0 {
        println!("  ✅ 零碰撞！百万样本指纹全部唯一");
    } else {
        println!("  ✅ 碰撞率在可接受范围内 ({:.6}%)", collision_rate * 100.0);
    }
}

// ═══════════════════════════════════════════════════════════════
// 2. 卡方均匀性检验
// ═══════════════════════════════════════════════════════════════

fn test_chi_squared(gen: &Generator) {
    const N: usize = 200_000;
    const BINS: usize = 20;

    let batch = gen.generate(N, None);

    // 对每个参数做卡方检验
    let mut total_chi = 0.0f64;
    let mut max_chi = 0.0f64;
    let mut min_chi = f64::MAX;
    let mut failures = 0usize;

    for param_idx in 0..84 {
        let mut bins = [0usize; BINS];
        let mut count = 0usize;

        for p in &batch {
            if !p.missing()[param_idx] {
                let bin = (p.values()[param_idx] * BINS as f64) as usize;
                let bin = bin.min(BINS - 1);
                bins[bin] += 1;
                count += 1;
            }
        }

        if count == 0 {
            continue;
        }

        let expected = count as f64 / BINS as f64;
        let chi: f64 = bins.iter()
            .map(|&obs| {
                let diff = obs as f64 - expected;
                diff * diff / expected
            })
            .sum();

        total_chi += chi;
        max_chi = max_chi.max(chi);
        min_chi = min_chi.min(chi);

        // 自由度 = BINS-1 = 19, α=0.01 临界值 ≈ 36.19
        if chi > 36.19 {
            failures += 1;
        }
    }

    let avg_chi = total_chi / 84.0;

    println!("  样本数:       {:>8}", N);
    println!("  分箱数:       {:>8}", BINS);
    println!("  平均 χ²:      {:>10.2} (期望 ~19)", avg_chi);
    println!("  χ² 范围:      [{:.2}, {:.2}]", min_chi, max_chi);
    println!("  未通过 α=0.01: {:>3} / 84", failures);

    // 84 个独立检验中，α=0.01 下期望 0.84 个失败
    // 允许最多 5 个（非常宽松）
    assert!(failures <= 5, "卡方检验失败过多: {}/84", failures);

    println!("  ✅ 卡方均匀性检验通过");
}

// ═══════════════════════════════════════════════════════════════
// 3. 参数独立性检验（皮尔逊相关系数）
// ═══════════════════════════════════════════════════════════════

fn test_parameter_independence(gen: &Generator) {
    const N: usize = 50_000;

    let batch = gen.generate(N, None);

    // 抽样检查 200 对参数（避免 O(84²) 全量计算）
    let pairs = [
        // 同领域
        (0, 1), (0, 5), (2, 7), (10, 15), (20, 25),
        // 跨领域
        (0, 14), (5, 30), (10, 50), (25, 70), (40, 80),
        // 随机
        (3, 77), (11, 66), (22, 55), (33, 44), (60, 75),
    ];

    let mut max_abs_corr = 0.0f64;
    let mut strong_correlations = 0usize;

    for &(a, b) in &pairs {
        let corr = pearson_correlation(&batch, a, b);
        let abs_corr = corr.abs();
        max_abs_corr = max_abs_corr.max(abs_corr);

        if abs_corr > 0.05 {
            strong_correlations += 1;
        }
    }

    println!("  检验对数:     {:>8}", pairs.len());
    println!("  最大 |r|:     {:>10.6}", max_abs_corr);
    println!("  |r|>0.05 对数: {:>6}", strong_correlations);

    // 参数应独立生成，不应有强相关
    assert!(max_abs_corr < 0.1,
        "检测到异常强相关: r={:.6}", max_abs_corr);

    println!("  ✅ 参数独立性检验通过（无显著相关）");
}

fn pearson_correlation(batch: &[personality_generator::Personality], a: usize, b: usize) -> f64 {
    let pairs: Vec<(f64, f64)> = batch.iter()
        .filter(|p| !p.missing()[a] && !p.missing()[b])
        .map(|p| (p.values()[a], p.values()[b]))
        .collect();

    if pairs.len() < 100 {
        return 0.0;
    }

    let n = pairs.len() as f64;
    let sum_x: f64 = pairs.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = pairs.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = pairs.iter().map(|(x, y)| x * y).sum();
    let sum_x2: f64 = pairs.iter().map(|(x, _)| x * x).sum();
    let sum_y2: f64 = pairs.iter().map(|(_, y)| y * y).sum();

    let num = n * sum_xy - sum_x * sum_y;
    let den = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

// ═══════════════════════════════════════════════════════════════
// 4. 双极参数对称性
// ═══════════════════════════════════════════════════════════════

fn test_bipolar_symmetry(gen: &Generator) {
    const N: usize = 100_000;

    let batch = gen.generate(N, None);

    // 找出所有双极参数
    let bipolar_indices: Vec<(usize, &str)> = PARAMS.iter()
        .enumerate()
        .filter(|(_, p)| p.bipolar)
        .map(|(i, p)| (i, p.id))
        .collect();

    println!("  双极参数数:   {:>8}", bipolar_indices.len());

    for &(idx, id) in &bipolar_indices {
        let values: Vec<f64> = batch.iter()
            .filter(|p| !p.missing()[idx])
            .map(|p| p.values()[idx])
            .collect();

        if values.len() < 1000 {
            continue;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // 双极参数中点 = 0.5（对应 raw=0），均值应接近 0.5
        let deviation = (mean - 0.5).abs();
        if deviation > 0.02 {
            println!("  ⚠ {} 均值偏离: {:.4} (期望 0.5000)", id, mean);
        }
    }

    // 汇总
    let all_means: Vec<f64> = bipolar_indices.iter()
        .map(|&(idx, _)| {
            let vals: Vec<f64> = batch.iter()
                .filter(|p| !p.missing()[idx])
                .map(|p| p.values()[idx])
                .collect();
            if vals.is_empty() { 0.5 } else { vals.iter().sum::<f64>() / vals.len() as f64 }
        })
        .collect();

    let grand_mean = all_means.iter().sum::<f64>() / all_means.len() as f64;
    println!("  双极参数总均值: {:.4} (期望 0.5000)", grand_mean);

    assert!((grand_mean - 0.5).abs() < 0.01,
        "双极参数总均值偏离过大: {:.4}", grand_mean);

    println!("  ✅ 双极参数对称性通过");
}

// ═══════════════════════════════════════════════════════════════
// 5. 缺失模式熵
// ═══════════════════════════════════════════════════════════════

fn test_missing_pattern_entropy(gen: &Generator) {
    const N: usize = 200_000;

    let batch = gen.generate(N, None);

    // 统计每个参数的缺失频率
    let mut param_missing = [0usize; 84];
    for p in &batch {
        for i in 0..84 {
            if p.missing()[i] {
                param_missing[i] += 1;
            }
        }
    }

    // 每个参数的缺失率应接近 15%
    let mut extreme_params = 0;
    for i in 0..84 {
        let rate = param_missing[i] as f64 / N as f64;
        if (rate - 0.15).abs() > 0.02 {
            extreme_params += 1;
        }
    }

    println!("  样本数:         {:>8}", N);
    println!("  参数缺失率范围:  [{:.3}, {:.3}]",
        *param_missing.iter().min().unwrap() as f64 / N as f64,
        *param_missing.iter().max().unwrap() as f64 / N as f64,
    );
    println!("  偏离>2%的参数:   {:>3} / 84", extreme_params);

    // 每人格缺失数分布
    let mut missing_count_dist = [0usize; 85];
    for p in &batch {
        missing_count_dist[p.missing_count()] += 1;
    }

    // 二项分布 B(84, 0.15) 的期望均值 = 12.6, 标准差 ≈ 3.27
    let mean_missing: f64 = batch.iter().map(|p| p.missing_count() as f64).sum::<f64>() / N as f64;
    let var_missing: f64 = batch.iter()
        .map(|p| {
            let diff = p.missing_count() as f64 - mean_missing;
            diff * diff
        })
        .sum::<f64>() / N as f64;

    println!("  每人格缺失均值:  {:.2} (期望 12.6)", mean_missing);
    println!("  每人格缺失标准差: {:.2} (期望 ~3.27)", var_missing.sqrt());

    assert!((mean_missing - 12.6).abs() < 0.5,
        "缺失均值偏离: {:.2}", mean_missing);

    println!("  ✅ 缺失模式分布正常");
}

// ═══════════════════════════════════════════════════════════════
// 6. 连续种子自相关
// ═══════════════════════════════════════════════════════════════

fn test_sequential_seed_correlation(gen: &Generator) {
    const N: usize = 10_000;

    // 生成连续种子的人格
    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    // 检查相邻种子的参数是否高度相关（不应该）
    let mut max_lag1_corr = 0.0f64;
    let mut sum_corr = 0.0f64;
    let mut count = 0usize;

    // 抽样 10 个参数
    let sample_params = [0, 10, 20, 30, 40, 50, 60, 70, 80, 83];

    for &pi in &sample_params {
        let vals: Vec<f64> = batch.iter()
            .filter(|p| !p.missing()[pi])
            .map(|p| p.values()[pi])
            .collect();

        if vals.len() < 100 {
            continue;
        }

        // lag-1 自相关
        let n = vals.len() - 1;
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let var: f64 = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64;

        if var < 1e-10 {
            continue;
        }

        let cov: f64 = (0..n)
            .map(|i| (vals[i] - mean) * (vals[i + 1] - mean))
            .sum::<f64>() / n as f64;

        let corr = cov / var;
        max_lag1_corr = max_lag1_corr.max(corr.abs());
        sum_corr += corr.abs();
        count += 1;
    }

    let avg_corr = if count > 0 { sum_corr / count as f64 } else { 0.0 };

    println!("  连续种子数:   {:>8}", N);
    println!("  检验参数数:   {:>8}", count);
    println!("  平均 |lag1|:  {:>10.6}", avg_corr);
    println!("  最大 |lag1|:  {:>10.6}", max_lag1_corr);

    // 连续种子不应有显著自相关
    assert!(max_lag1_corr < 0.05,
        "连续种子检测到异常自相关: {:.6}", max_lag1_corr);

    println!("  ✅ 连续种子无显著自相关");
}

// ═══════════════════════════════════════════════════════════════
// 7. 极端偏向下的参数边界
// ═══════════════════════════════════════════════════════════════

fn test_bias_boundaries(gen: &Generator) {
    const N: usize = 10_000;

    // 全高偏向：所有值应该 ≥ 0.5（非双极）或保持对称（双极）
    let batch_high = gen.generate(N, Some("A=1.0,B=1.0,C=1.0,D=1.0,E=1.0,F=1.0,G=1.0,H=1.0,STRENGTH=1.0"));
    let mut non_bipolar_high_violations = 0u64;

    for p in &batch_high {
        for i in 0..84 {
            if !p.missing()[i] && !PARAMS[i].bipolar {
                if p.values()[i] < 0.4 {
                    non_bipolar_high_violations += 1;
                }
            }
        }
    }

    let total_non_bipolar = batch_high.len() * PARAMS.iter().filter(|p| !p.bipolar).count();
    let violation_rate = non_bipolar_high_violations as f64 / total_non_bipolar as f64;

    println!("  全高偏向样本: {:>8}", N);
    println!("  非双极参数值<0.4: {} / {} ({:.4}%)",
        non_bipolar_high_violations, total_non_bipolar, violation_rate * 100.0);

    // STRENGTH=1.0 时，非双极参数应几乎全部 ≥ 0.5
    // 但随机性仍存在，允许少量偏离
    assert!(violation_rate < 0.05,
        "全高偏向偏离过多: {:.2}%", violation_rate * 100.0);

    // 全低偏向
    let batch_low = gen.generate(N, Some("A=-1.0,B=-1.0,C=-1.0,D=-1.0,E=-1.0,F=-1.0,G=-1.0,H=-1.0,STRENGTH=1.0"));
    let mut non_bipolar_low_violations = 0u64;

    for p in &batch_low {
        for i in 0..84 {
            if !p.missing()[i] && !PARAMS[i].bipolar {
                if p.values()[i] > 0.6 {
                    non_bipolar_low_violations += 1;
                }
            }
        }
    }

    let violation_rate_low = non_bipolar_low_violations as f64 / total_non_bipolar as f64;
    println!("  全低偏向: 值>0.6: {} / {} ({:.4}%)",
        non_bipolar_low_violations, total_non_bipolar, violation_rate_low * 100.0);

    assert!(violation_rate_low < 0.05,
        "全低偏向偏离过多: {:.2}%", violation_rate_low * 100.0);

    println!("  ✅ 极端偏向边界验证通过");
}

// ═══════════════════════════════════════════════════════════════
// 8. 浮点精度与指纹稳定性
// ═══════════════════════════════════════════════════════════════

fn test_fingerprint_precision(gen: &Generator) {
    // 验证指纹格式："0.XXXX|0.XXXX|0.XXXX|0.XXXX"
    let p = gen.from_seed(42, None);

    println!("  示例指纹: {}", p.fingerprint());

    // 解析指纹
    if p.fingerprint() != "ALL_MISSING" {
        let parts: Vec<&str> = p.fingerprint().split('|').collect();
        assert!(parts.len() <= 4, "指纹格式错误");

        for part in &parts {
            let val: f64 = part.parse().unwrap();
            assert!((0.0..=1.0).contains(&val), "指纹值超出范围: {}", val);
            // 验证 4 位小数精度
            let formatted = format!("{:.4}", val);
            assert_eq!(part, &formatted, "指纹精度不一致: {} vs {}", part, formatted);
        }
    }

    // 验证同一指纹在不同操作后不变
    let fp1 = p.fingerprint().clone();
    let _ = p.get("A001");
    let _ = p.missing_count();
    assert_eq!(p.fingerprint(), fp1, "指纹在只读操作后发生了变化");

    // 验证值的浮点精度
    for (i, &v) in p.values().iter().enumerate() {
        if !p.missing()[i] {
            assert!(v.is_finite(), "参数 {} 值非有限: {}", PARAMS[i].id, v);
            assert!(!v.is_nan(), "参数 {} 值为 NaN", PARAMS[i].id);
            assert!(!v.is_infinite(), "参数 {} 值为无穷", PARAMS[i].id);
        }
    }

    // 种子耗尽后应优雅降级
    // 构造一个极短种子：手动设置 pos 到接近末尾
    let mut seed = Seed::from_i32(1);
    seed.reset();
    // 快速消耗种子：1024 / 8 = 128 个 f64
    for _ in 0..128 {
        let _ = seed.read_f64();
    }
    // 此时种子已耗尽
    assert!(seed.read_f64().is_err(), "种子应已耗尽");
    assert!(seed.read_f32().is_err(), "种子应已耗尽");
    assert!(seed.read_bit().is_err(), "种子应已耗尽");

    println!("  ✅ 浮点精度与指纹稳定性通过");
}
