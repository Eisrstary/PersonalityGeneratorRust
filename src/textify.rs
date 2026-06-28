//! 文本生成器 —— 将人格数据转换为可读文本。
//!
//! 支持三种输出模式：角色扮演人设、紧凑模式、详细模式。

use crate::generator::Personality;
use crate::params::ALL_PARAMS;

/// 领域名称映射。
const DOMAIN_NAMES: [(char, &str); 8] = [
    ('A', "信息处理"),
    ('B', "情绪模式"),
    ('C', "动机与价值观"),
    ('D', "行为风格"),
    ('E', "自我认知"),
    ('F', "社交特征"),
    ('G', "发展特征"),
    ('H', "身体-环境反应"),
];

// ═══════════════════════════════════════════════════════════════
// 公共 API
// ═══════════════════════════════════════════════════════════════

/// 将人格转换为角色扮演人设文本（最完整）。
pub fn to_roleplay(p: &Personality) -> String {
    let mut out = String::new();

    out.push_str("【人格档案】\n");
    out.push_str(&format!(
        "指纹: {} | 缺失: {}/84\n\n",
        p.fingerprint,
        p.missing_count()
    ));

    out.push_str("【核心性格】\n");
    out.push_str(&core_traits(p));
    out.push_str("\n\n");

    for &(dom, name) in &DOMAIN_NAMES {
        out.push_str(&format!("【{}】\n", name));
        out.push_str(&domain_desc(p, dom));
        out.push_str("\n\n");
    }

    out.push_str("【角色扮演提示】\n");
    out.push_str(&role_hints(p));

    out
}

/// 紧凑模式：只输出偏离中值较大的参数。
pub fn to_compact(p: &Personality) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "指纹: {} | 缺失: {}/84\n",
        p.fingerprint,
        p.missing_count()
    ));

    for i in 0..84 {
        if p.missing[i] {
            continue;
        }
        let dev = (p.values[i] - 0.5).abs();
        if dev > 0.3 {
            let arrow = if p.values[i] > 0.5 { "↑" } else { "↓" };
            out.push_str(&format!(
                "  {} {}: {} ({:.2})\n",
                ALL_PARAMS[i].id, ALL_PARAMS[i].name, arrow, p.values[i]
            ));
        }
    }

    out
}

/// 详细模式：逐参数输出完整描述。
pub fn to_detailed(p: &Personality) -> String {
    let mut out = String::new();

    for i in 0..84 {
        let def = &ALL_PARAMS[i];
        let desc = if p.missing[i] {
            "[缺失]".to_string()
        } else {
            let raw = def.min + p.values[i] * (def.max - def.min);
            def.describe(raw)
        };
        out.push_str(&format!("{} {}: {}\n", def.id, def.name, desc));
    }

    out
}

// ═══════════════════════════════════════════════════════════════
// 内部辅助
// ═══════════════════════════════════════════════════════════════

fn core_traits(p: &Personality) -> String {
    let mut out = String::new();

    let c025 = p.get("C025");
    let c033 = p.get("C033");
    if c025 > 0.6 && c033 > 0.5 {
        out.push_str("  • 外向亲和——主动接近他人，渴望温暖关系\n");
    } else if c025 < 0.4 && c033 < 0.5 {
        out.push_str("  • 内向疏离——倾向独处，人际非核心需求\n");
    } else if c025 > 0.6 && c033 < 0.5 {
        out.push_str("  • 外向工具型——主动社交但视人际为手段\n");
    }

    let b015 = p.get("B015");
    let b021 = p.get("B021");
    if b015 > 0.6 && b021 > 0.5 {
        out.push_str("  • 高共情——易被感染，伤害他人后强烈内疚\n");
    } else if b015 < 0.4 && b021 < 0.5 {
        out.push_str("  • 情感淡漠——不易感染，伤害他人较少内疚\n");
    }

    let c032 = p.get("C032");
    let e051 = p.get("E051");
    if c032 > 0.6 && e051 > 0.5 {
        out.push_str("  • 使命型权力——权力是完成使命的工具\n");
    } else if c032 > 0.6 {
        out.push_str("  • 权力导向——渴望影响和控制他人\n");
    } else if e051 > 0.6 {
        out.push_str("  • 使命驱动——有清晰的人生目标\n");
    }

    let d040 = p.get("D040");
    let c030 = p.get("C030");
    if d040 > 0.6 && c030 < 0.4 {
        out.push_str("  • 冲动攻击——高攻击倾向且缺乏控制\n");
    } else if d040 > 0.6 && c030 > 0.6 {
        out.push_str("  • 克制攻击——有攻击倾向但能有效控制\n");
    } else if d040 < 0.4 {
        out.push_str("  • 和平倾向——攻击基线较低\n");
    }

    out.trim_end().to_string()
}

fn domain_desc(p: &Personality, dom: char) -> String {
    let mut out = String::new();
    let mut count = 0;

    for i in 0..84 {
        if ALL_PARAMS[i].domain != dom || p.missing[i] {
            continue;
        }
        let dev = (p.values[i] - 0.5).abs();
        if dev > 0.2 {
            let def = &ALL_PARAMS[i];
            let raw = def.min + p.values[i] * (def.max - def.min);
            out.push_str(&format!("  {}\n", def.describe(raw)));
            count += 1;
        }
    }

    if count == 0 {
        out.push_str("  [表现均衡，无显著极端特征]\n");
    }

    out.trim_end().to_string()
}

fn role_hints(p: &Personality) -> String {
    let mut out = String::new();

    let c028 = p.get("C028");
    if c028 > 0.7 {
        out.push_str("  • 说话风格：独立自主，抵触被指挥\n");
    } else if c028 < 0.3 {
        out.push_str("  • 说话风格：顺从配合，接受安排\n");
    }

    let b017 = p.get("B017");
    if b017 > 0.7 {
        out.push_str("  • 情绪表达：容易羞耻，社交失误后长时间纠结\n");
    } else if b017 < 0.3 {
        out.push_str("  • 情绪表达：不在意他人眼光，恢复快\n");
    }

    let c030 = p.get("C030");
    let d039 = p.get("D039");
    if c030 < 0.3 {
        out.push_str("  • 决策风格：冲动型，快速决定立即行动\n");
    } else if d039 > 0.7 {
        out.push_str("  • 决策风格：拖延型，决定后需较长时间启动\n");
    } else {
        out.push_str("  • 决策风格：平衡型，思考与行动节奏合理\n");
    }

    let f061 = p.get("F061");
    let f062 = p.get("F062");
    if f061 < 0.3 && f062 > 0.7 {
        out.push_str("  • 人际关系：高度警觉，默认不信任，时刻准备发现背叛\n");
    } else if f061 > 0.7 {
        out.push_str("  • 人际关系：容易信任，对陌生人持开放态度\n");
    } else {
        out.push_str("  • 人际关系：谨慎开放，需时间建立信任\n");
    }

    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::Generator;

    #[test]
    fn test_textify_outputs() {
        let gen = Generator::new();
        let p = gen.generate_from_seed(42, None);

        let rp = to_roleplay(&p);
        assert!(rp.contains("【人格档案】"));
        assert!(rp.contains("【核心性格】"));
        assert!(rp.contains("【角色扮演提示】"));

        let compact = to_compact(&p);
        assert!(compact.contains("指纹:"));

        let detailed = to_detailed(&p);
        assert!(detailed.contains("A001"));
        assert!(detailed.contains("H084"));
    }
}
