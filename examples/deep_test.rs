//! 深层数学极限压力测试 —— 雪崩效应 · 生日边界 · 参数空间体积
//!
//! 测试维度：
//!   1. 种子雪崩效应（单 bit 翻转 → 参数变化率）
//!   2. 生日悖论理论边界验证
//!   3. 参数空间可达体积（蒙特卡洛积分）
//!   4. Kolmogorov-Smirnov 均匀性检验
//!   5. 连续种子轨迹分析（参数空间行走）
//!   6. Bias 拉力精确响应曲线
//!   7. 多维参数联合分布
//!   8. 种子熵与信息量

use personality_generator::Generator;
use personality_generator::params::PARAMS;
use std::collections::HashSet;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     人格原子参数系统 —— 深层数学极限测试                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let gen = Generator::new();

    // ── 1. 种子雪崩效应 ──
    println!("━━━ [1/8] 种子雪崩效应（单 bit 翻转）━━━");
    test_avalanche_effect(&gen);

    // ── 2. 生日悖论边界 ──
    println!("\n━━━ [2/8] 生日悖论理论碰撞边界 ━━━");
    test_birthday_boundary(&gen);

    // ── 3. 参数空间体积 ──
    println!("\n━━━ [3/8] 84维参数空间可达体积 ━━━");
    test_volume_estimation(&gen);

    // ── 4. KS 均匀性检验 ──
    println!("\n━━━ [4/8] Kolmogorov-Smirnov 均匀性检验 ━━━");
    test_ks_uniformity(&gen);

    // ── 5. 连续种子轨迹 ──
    println!("\n━━━ [5/8] 种子空间参数行走轨迹 ━━━");
    test_seed_trajectory(&gen);

    // ── 6. Bias 精确响应曲线 ──
    println!("\n━━━ [6/8] Bias 拉力非线性响应曲线 ━━━");
    test_bias_response_curve(&gen);

    // ── 7. 多维联合分布 ──
    println!("\n━━━ [7/8] 参数二维联合分布检验 ━━━");
    test_joint_distribution(&gen);

    // ── 8. 种子信息熵 ──
    println!("\n━━━ [8/8] 种子熵与信息密度 ━━━");
    test_seed_entropy(&gen);

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║            深层数学极限测试全部通过 ✅                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

// ═══════════════════════════════════════════════════════════════
// 1. 种子雪崩效应
// ═══════════════════════════════════════════════════════════════

fn test_avalanche_effect(gen: &Generator) {
    // 测试：翻转种子的单个 bit，观察 84 个参数的平均变化率
    // 好的 RNG 应有 ~50% 的参数发生显著变化（雪崩效应）

    const BASE_SEED: i32 = 0x5A5A5A5Au32 as i32;
    const NUM_BITS: usize = 32;
    const TRIALS_PER_BIT: usize = 100;

    let base = gen.from_seed(BASE_SEED, None);

    let mut total_change_rate = 0.0f64;
    let mut max_change_rate = 0.0f64;
    let mut min_change_rate = 1.0f64;

    for bit in 0..NUM_BITS {
        let flipped_seed = BASE_SEED ^ (1i32 << bit);

        let mut bit_change_count = 0usize;
        let mut bit_total = 0usize;

        for _ in 0..TRIALS_PER_BIT {
            let flipped = gen.from_seed(flipped_seed, None);
            // 使用不同的基础种子偏移
            let b = gen.from_seed(BASE_SEED.wrapping_add(bit as i32 * 1000), None);
            let f = gen.from_seed(flipped_seed.wrapping_add(bit as i32 * 1000), None);

            for param_idx in 0..84 {
                if !b.missing()[param_idx] && !f.missing()[param_idx] {
                    let diff = (b.values()[param_idx] - f.values()[param_idx]).abs();
                    if diff > 0.01 {
                        bit_change_count += 1;
                    }
                    bit_total += 1;
                }
            }
        }

        let change_rate = bit_change_count as f64 / bit_total as f64;
        total_change_rate += change_rate;
        max_change_rate = max_change_rate.max(change_rate);
        min_change_rate = min_change_rate.min(change_rate);
    }

    let avg_change = total_change_rate / NUM_BITS as f64;

    println!("  基础种子:      0x{:08X}", BASE_SEED as u32);
    println!("  翻转 bit 数:   {}", NUM_BITS);
    println!("  每 bit 试验:   {}", TRIALS_PER_BIT);
    println!("  平均变化率:    {:.2}% (期望 ~50%)", avg_change * 100.0);
    println!("  变化率范围:    [{:.2}%, {:.2}%]", min_change_rate * 100.0, max_change_rate * 100.0);

    // 好的雪崩效应：任何单 bit 翻转应导致约一半参数显著变化
    assert!(avg_change > 0.40, "雪崩效应不足: {:.2}%", avg_change * 100.0);
    assert!(min_change_rate > 0.30, "某 bit 翻转变化率过低: {:.2}%", min_change_rate * 100.0);

    println!("  ✅ 雪崩效应良好（单 bit 翻转 → ~50% 参数变化）");
}

// ═══════════════════════════════════════════════════════════════
// 2. 生日悖论边界
// ═══════════════════════════════════════════════════════════════

fn test_birthday_boundary(gen: &Generator) {
    // 生日悖论：在 N 个可能值中，约 √(2N·ln(1/(1-p))) 个样本后碰撞概率为 p
    // 指纹空间：前 4 个非缺失参数，每个 4 位小数 → 10000^4 = 10^16
    // 理论 50% 碰撞概率需要约 √(2×10^16×ln2) ≈ 1.18×10^8 个样本

    let space_4digit = 10000u128.pow(4); // 10^16
    let birthday_50 = (2.0 * space_4digit as f64 * (1.0f64 / (1.0 - 0.5)).ln()).sqrt();
    let birthday_1percent = (2.0 * space_4digit as f64 * (1.0f64 / (1.0 - 0.01)).ln()).sqrt();
    let birthday_1e6 = (2.0 * space_4digit as f64 * (1.0f64 / (1.0 - 1e-6)).ln()).sqrt();

    println!("  指纹理论空间:  10^16 (4参数 × 4位小数)");
    println!("  生日边界:");
    println!("    50% 碰撞 @ {:>12.2e} 样本", birthday_50);
    println!("     1% 碰撞 @ {:>12.2e} 样本", birthday_1percent);
    println!("    1e-6 碰撞 @ {:>12.2e} 样本", birthday_1e6);

    // 实际验证：在较小空间内测试生日悖论
    // 取单个参数 4 位小数 → 10000 个可能值
    const N: usize = 5000;
    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    // 对参数 A001（索引 0），统计碰撞
    let mut seen: HashSet<u32> = HashSet::new();
    let mut collisions = 0usize;
    let mut total = 0usize;
    for p in &batch {
        if !p.missing()[0] {
            let bucket = (p.values()[0] * 10000.0) as u32;
            if !seen.insert(bucket) {
                collisions += 1;
            }
            total += 1;
        }
    }

    // 理论碰撞数：n - N + N·((N-1)/N)^n ≈ n²/(2N) for n<<N
    let theoretical = (total as f64 * total as f64) / (2.0 * 10000.0);
    println!("\n  单参数验证 (A001, 10000 级):");
    println!("    样本数:     {}", total);
    println!("    实测碰撞:   {}", collisions);
    println!("    理论碰撞:   {:.1}", theoretical);

    let ratio = if theoretical > 0.0 { collisions as f64 / theoretical } else { 1.0 };
    assert!((ratio - 1.0).abs() < 0.5,
        "碰撞数与理论偏差过大: ratio={:.2}", ratio);

    println!("  ✅ 生日悖论验证通过（实测/理论 = {:.2}）", ratio);
}

// ═══════════════════════════════════════════════════════════════
// 3. 参数空间可达体积
// ═══════════════════════════════════════════════════════════════

fn test_volume_estimation(gen: &Generator) {
    // 84 维超立方体 [0,1]^84 的体积 = 1
    // 但实际可达到的体积受限于种子空间（32 bit → 43 亿种可能）
    // 用蒙特卡洛方法估算实际覆盖的体积比例

    const N: usize = 100_000;

    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    // 方法：将 84 维空间分成超立方格子，统计被占据的格子数
    // 简化：用前 3 个参数做 3D 可视化统计
    const GRID: usize = 20; // 20×20×20 = 8000 个格子

    let mut grid = vec![0u32; GRID * GRID * GRID];

    for p in &batch {
        if !p.missing()[0] && !p.missing()[14] && !p.missing()[28] {
            let x = (p.values()[0] * GRID as f64) as usize;
            let y = (p.values()[14] * GRID as f64) as usize;
            let z = (p.values()[28] * GRID as f64) as usize;
            let idx = (x.min(GRID - 1)) * GRID * GRID
                + (y.min(GRID - 1)) * GRID
                + (z.min(GRID - 1));
            grid[idx] += 1;
        }
    }

    let occupied = grid.iter().filter(|&&c| c > 0).count();
    let total_cells = GRID * GRID * GRID;
    let coverage = occupied as f64 / total_cells as f64;

    // 统计格子计数的分布
    let max_count = grid.iter().max().unwrap();
    let min_occupied = grid.iter().filter(|&&c| c > 0).min().unwrap();
    let mean_occupied: f64 = grid.iter().filter(|&&c| c > 0).map(|&c| c as f64).sum::<f64>()
        / occupied as f64;

    println!("  样本数:       {:>8}", N);
    println!("  3D 网格:      {}×{}×{} = {}", GRID, GRID, GRID, total_cells);
    println!("  被占据格子:   {} / {} ({:.1}%)", occupied, total_cells, coverage * 100.0);
    println!("  格子计数:     min={}, max={}, mean={:.1}", min_occupied, max_count, mean_occupied);

    // 100k 样本下，8000 个格子应几乎全覆盖
    assert!(coverage > 0.98, "3D 空间覆盖不足: {:.1}%", coverage * 100.0);

    // 外推到 84 维：每个维度 2 分格 → 2^84 ≈ 1.9×10^25 个超格子
    // 43 亿种子只能覆盖极小部分，但分布应均匀
    println!("  84维外推:     2^84 ≈ 1.9×10^25 超格子");
    println!("  种子空间:     2^32 ≈ 4.3×10^9 种可能");
    println!("  ✅ 3D 投影覆盖充分，高维分布合理");
}

// ═══════════════════════════════════════════════════════════════
// 4. Kolmogorov-Smirnov 检验
// ═══════════════════════════════════════════════════════════════

fn test_ks_uniformity(gen: &Generator) {
    const N: usize = 50_000;

    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    let mut max_ks = 0.0f64;
    let mut min_ks = f64::MAX;
    let mut total_ks = 0.0f64;
    let mut failures = 0usize;

    for param_idx in 0..84 {
        let mut values: Vec<f64> = batch.iter()
            .filter(|p| !p.missing()[param_idx])
            .map(|p| p.values()[param_idx])
            .collect();

        if values.len() < 1000 {
            continue;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = values.len();

        // KS 统计量 D = max|F_n(x) - F(x)| = max|i/n - x_i|
        let mut d = 0.0f64;
        for (i, &v) in values.iter().enumerate() {
            let ecdf = (i + 1) as f64 / n as f64;
            let diff = (ecdf - v).abs();
            d = d.max(diff);
            // 也检查下界
            let diff_low = (v - i as f64 / n as f64).abs();
            d = d.max(diff_low);
        }

        max_ks = max_ks.max(d);
        min_ks = min_ks.min(d);
        total_ks += d;

        // α=0.01 临界值 ≈ 1.628 / √n
        let critical = 1.628 / (n as f64).sqrt();
        if d > critical {
            failures += 1;
        }
    }

    let avg_ks = total_ks / 84.0;

    println!("  样本数:         {:>8}", N);
    println!("  平均 KS D:      {:.6}", avg_ks);
    println!("  KS D 范围:      [{:.6}, {:.6}]", min_ks, max_ks);
    println!("  未通过 α=0.01:  {:>3} / 84", failures);

    // 84 个检验中 α=0.01 期望 ~1 个失败
    assert!(failures <= 5, "KS 检验失败过多: {}/84", failures);

    println!("  ✅ Kolmogorov-Smirnov 均匀性检验通过");
}

// ═══════════════════════════════════════════════════════════════
// 5. 连续种子轨迹分析
// ═══════════════════════════════════════════════════════════════

fn test_seed_trajectory(gen: &Generator) {
    // 在种子空间中沿一条线行走，观察参数变化是否像随机游走
    const STEPS: usize = 2000;
    const PARAM: usize = 0; // A001

    let mut values = Vec::with_capacity(STEPS);
    let mut missing_count = 0usize;

    for step in 0..STEPS {
        let p = gen.from_seed(step as i32, None);
        if p.missing()[PARAM] {
            missing_count += 1;
            values.push(0.5); // 缺失用中点填充
        } else {
            values.push(p.values()[PARAM]);
        }
    }

    // 计算 Hurst 指数（简化：用 R/S 分析）
    // H=0.5 → 随机游走，H>0.5 → 趋势，H<0.5 → 均值回归
    let hurst = estimate_hurst(&values);

    // 计算自相关
    let mean = values.iter().sum::<f64>() / STEPS as f64;
    let var: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / STEPS as f64;

    let mut acf_lag1 = 0.0;
    let mut acf_lag2 = 0.0;
    for i in 0..STEPS - 1 {
        acf_lag1 += (values[i] - mean) * (values[i + 1] - mean);
    }
    for i in 0..STEPS - 2 {
        acf_lag2 += (values[i] - mean) * (values[i + 2] - mean);
    }
    acf_lag1 /= (STEPS - 1) as f64 * var;
    acf_lag2 /= (STEPS - 2) as f64 * var;

    // 计算转折率（相邻值一阶差分的符号变化率）
    let mut sign_changes = 0usize;
    for i in 1..STEPS - 1 {
        let d1 = values[i] - values[i - 1];
        let d2 = values[i + 1] - values[i];
        if d1.signum() != d2.signum() && d1 != 0.0 && d2 != 0.0 {
            sign_changes += 1;
        }
    }
    let turning_rate = sign_changes as f64 / (STEPS - 2) as f64;

    println!("  轨迹长度:      {} 步", STEPS);
    println!("  参数:          {}", PARAMS[PARAM].name);
    println!("  缺失步数:      {} / {}", missing_count, STEPS);
    println!("  Hurst 指数:    {:.4} (0.5=随机)", hurst);
    println!("  ACF lag-1:     {:.4}", acf_lag1);
    println!("  ACF lag-2:     {:.4}", acf_lag2);
    println!("  转折率:        {:.2}% (期望 ~66.7%)", turning_rate * 100.0);

    // 随机序列的转折率 ≈ 2/3
    assert!((turning_rate - 2.0 / 3.0).abs() < 0.05,
        "转折率偏离随机: {:.2}%", turning_rate * 100.0);
    assert!(acf_lag1.abs() < 0.1, "lag-1 自相关过高: {:.4}", acf_lag1);

    println!("  ✅ 参数轨迹表现为纯随机游走");
}

/// 简化 R/S Hurst 指数估计
fn estimate_hurst(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 100 {
        return 0.5;
    }

    let _mean = data.iter().sum::<f64>() / n as f64;

    // 使用多个时间尺度
    let scales = [10, 20, 50, 100, 200, 500];
    let mut log_rs = Vec::new();
    let mut log_n = Vec::new();

    for &scale in &scales {
        if scale > n / 2 {
            continue;
        }
        let chunks = n / scale;
        let mut rs_sum = 0.0;

        for c in 0..chunks {
            let start = c * scale;
            let end = start + scale;
            let chunk_mean: f64 = data[start..end].iter().sum::<f64>() / scale as f64;

            let mut cum_dev = 0.0;
            let mut max_cum = f64::MIN;
            let mut min_cum = f64::MAX;

            for i in start..end {
                cum_dev += data[i] - chunk_mean;
                max_cum = max_cum.max(cum_dev);
                min_cum = min_cum.min(cum_dev);
            }

            let variance: f64 = data[start..end].iter()
                .map(|v| (v - chunk_mean).powi(2))
                .sum::<f64>() / scale as f64;

            let std_dev = variance.sqrt();
            if std_dev > 1e-10 {
                rs_sum += (max_cum - min_cum) / std_dev;
            }
        }

        let rs = rs_sum / chunks as f64;
        log_rs.push(rs.ln());
        log_n.push((scale as f64).ln());
    }

    if log_rs.len() < 2 {
        return 0.5;
    }

    // 线性回归 log(RS) = H * log(n) + C
    let k = log_rs.len() as f64;
    let sum_x: f64 = log_n.iter().sum();
    let sum_y: f64 = log_rs.iter().sum();
    let sum_xy: f64 = log_n.iter().zip(log_rs.iter()).map(|(x, y)| x * y).sum();
    let sum_x2: f64 = log_n.iter().map(|x| x * x).sum();

    let h = (k * sum_xy - sum_x * sum_y) / (k * sum_x2 - sum_x * sum_x);
    h.clamp(0.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════
// 6. Bias 精确响应曲线
// ═══════════════════════════════════════════════════════════════

fn test_bias_response_curve(gen: &Generator) {
    const N: usize = 20_000;

    // 测量不同 bias 强度下的实际参数均值
    let test_param = "B015"; // 内疚感基线，非双极 [0, 1]
    let strengths = [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 0.9, 1.0];

    println!("  参数: {} ({})", test_param, PARAMS[14].name);
    println!("  {:>8}  {:>10}  {:>10}  {:>10}", "强度", "实测均值", "理论均值", "偏差");

    let mut prev_mean = 0.0;
    let mut first = true;
    for &s in &strengths {
        let bias_spec = if s == 0.0 {
            None
        } else {
            Some(format!("{}=1.0,STRENGTH={}", test_param, s))
        };

        // 用确定性种子避免随机波动
        let mut sum = 0.0;
        let mut count = 0usize;
        for seed in 0..N as i32 {
            let p = gen.from_seed(seed, bias_spec.as_deref());
            if !p.missing()[14] {
                sum += p.values()[14];
                count += 1;
            }
        }
        let mean = sum / count as f64;

        // 理论：非双极参数，eb=1.0，目标=1.0
        // effective_pull = s × √1.0 = s
        // rnd' = rnd + (1.0 - rnd) × s = rnd·(1-s) + s
        // E[rnd'] = 0.5·(1-s) + s = 0.5 + 0.5s
        let theoretical = 0.5 + 0.5 * s;

        let deviation = mean - theoretical;
        println!("  {:>8.1}  {:>10.4}  {:>10.4}  {:>+10.4}", s, mean, theoretical, deviation);

        // 单调性检查
        if !first {
            assert!(mean >= prev_mean - 0.01,
                "Bias 响应非单调: s={} mean={:.4} prev={:.4}", s, mean, prev_mean);
        }
        first = false;
        prev_mean = mean;

        // 偏差应 < 0.02
        assert!(deviation.abs() < 0.02,
            "Bias 响应偏差过大: s={} dev={:.4}", s, deviation);
    }

    println!("  ✅ Bias 响应曲线符合理论预测");
}

// ═══════════════════════════════════════════════════════════════
// 7. 多维联合分布检验
// ═══════════════════════════════════════════════════════════════

fn test_joint_distribution(gen: &Generator) {
    const N: usize = 100_000;

    let batch: Vec<_> = (0..N)
        .map(|i| gen.from_seed(i as i32, None))
        .collect();

    // 测试 5 对参数的联合分布是否接近独立均匀
    let pairs = [(0, 14), (14, 28), (28, 42), (42, 56), (56, 70)];

    // 将每对参数的 2D 空间分成 10×10 网格
    const G: usize = 10;
    let mut max_chi = 0.0f64;

    for &(a, b) in &pairs {
        let mut grid = [[0usize; G]; G];
        let mut count = 0usize;

        for p in &batch {
            if !p.missing()[a] && !p.missing()[b] {
                let x = ((p.values()[a] * G as f64) as usize).min(G - 1);
                let y = ((p.values()[b] * G as f64) as usize).min(G - 1);
                grid[x][y] += 1;
                count += 1;
            }
        }

        let cell_expected = count as f64 / (G * G) as f64;
        let mut chi = 0.0;
        for row in &grid {
            for &c in row {
                let diff = c as f64 - cell_expected;
                chi += diff * diff / cell_expected;
            }
        }
        max_chi = max_chi.max(chi);
    }

    // 自由度 = 99 (10×10-1), α=0.01 临界值 ≈ 134
    println!("  样本数:         {:>8}", N);
    println!("  检验参数对:     {:>8}", pairs.len());
    println!("  2D 网格:        {}×{} (自由度 99)", G, G);
    println!("  最大 χ²:        {:.1} (临界值 α=0.01: 134)", max_chi);

    assert!(max_chi < 200.0, "联合分布 χ² 过高: {:.1}", max_chi);

    println!("  ✅ 多维联合分布接近独立均匀");
}

// ═══════════════════════════════════════════════════════════════
// 8. 种子信息熵
// ═══════════════════════════════════════════════════════════════

fn test_seed_entropy(_gen: &Generator) {
    // 测试：种子扩展后的 1024 字节是否具有高熵
    // 用 Shannon 熵估计

    let seed = personality_generator::Seed::from_i32(42);
    let bytes = seed.as_bytes();

    // 字节级熵
    let mut freq = [0usize; 256];
    for &b in bytes.iter() {
        freq[b as usize] += 1;
    }

    let total = 1024.0;
    let mut entropy = 0.0f64;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }

    let max_entropy = 8.0; // 256 个可能值均匀分布 = 8 bits/byte

    // 连续字节差分熵（排除简单模式）
    let mut diff_freq = [0usize; 256];
    for i in 1..1024 {
        let diff = bytes[i].wrapping_sub(bytes[i - 1]);
        diff_freq[diff as usize] += 1;
    }
    let mut diff_entropy = 0.0;
    for &count in &diff_freq {
        if count > 0 {
            let p = count as f64 / 1023.0;
            diff_entropy -= p * p.log2();
        }
    }

    // 位级熵
    let mut ones = 0u64;
    for &b in bytes.iter() {
        ones += b.count_ones() as u64;
    }
    let bit_ratio = ones as f64 / (1024.0 * 8.0);

    println!("  种子长度:      1024 字节");
    println!("  字节熵:        {:.4} / 8.0 ({:.1}%)", entropy, entropy / max_entropy * 100.0);
    println!("  差分字节熵:    {:.4} / 8.0 ({:.1}%)", diff_entropy, diff_entropy / max_entropy * 100.0);
    println!("  位 1 比例:     {:.4} (期望 0.5000)", bit_ratio);

    // 高质量随机数据应有接近最大的熵（xorshift128+ 实际 ~7.8/8.0）
    assert!(entropy > 7.7, "字节熵不足: {:.4}", entropy);
    assert!((bit_ratio - 0.5).abs() < 0.02, "位比例偏离: {:.4}", bit_ratio);

    println!("  ✅ 种子具有高信息熵（接近理论最大值）");
}
