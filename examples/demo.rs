//! 演示：人格生成器的基本用法。

use personality_generator::{Generator, textify};

fn main() {
    let gen = Generator::new();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     人格原子参数系统 (PAPS) —— Rust 实现演示                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ── 示例 1: 纯随机人格 ──
    println!("━━━ 示例 1: 纯随机人格 (种子=42) ━━━\n");
    let p1 = gen.from_seed(42, None);
    println!("{}", textify::to_roleplay(&p1));

    // ── 示例 2: 带偏向的人格 ──
    println!("\n━━━ 示例 2: 高内疚+高支配 (种子=42, B015=0.9,C031=0.8) ━━━\n");
    let p2 = gen.from_seed(42, Some("B015=0.9,C031=0.8"));
    println!("{}", textify::to_roleplay(&p2));

    // ── 示例 3: 紧凑模式 ──
    println!("\n━━━ 示例 3: 紧凑模式 (种子=999) ━━━\n");
    let p3 = gen.from_seed(999, None);
    println!("{}", textify::to_compact(&p3));

    // ── 示例 4: 详细模式（前 10 个参数） ──
    println!("\n━━━ 示例 4: 详细模式（仅显示前10个参数）━━━\n");
    let detailed = textify::to_detailed(&p3);
    for line in detailed.lines().take(10) {
        println!("{}", line);
    }

    // ── 示例 5: 批量生成统计 ──
    println!("\n━━━ 示例 5: 批量生成 5 个人格的指纹 ━━━\n");
    let batch = gen.generate(5, None);
    for (i, p) in batch.iter().enumerate() {
        println!(
            "  [{}] 指纹: {} | 缺失: {}/84",
            i + 1,
            p.fingerprint(),
            p.missing_count()
        );
    }
}
