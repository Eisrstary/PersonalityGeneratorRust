use crate::generator::Personality;
use crate::params::{self, Domain, PARAMS};

pub fn to_roleplay(p: &Personality) -> String {
    let mut o = String::new();
    o.push_str("【人格档案】\n");
    o.push_str(&format!("指纹: {} | 缺失: {}/84\n\n", p.fingerprint(), p.missing_count()));
    o.push_str("【核心性格】\n"); o.push_str(&core_traits(p)); o.push_str("\n\n");
    for dom in &Domain::ALL { o.push_str(&format!("【{}】\n{}\n\n", dom.name(), domain_desc(p, *dom))); }
    o.push_str("【角色扮演提示】\n"); o.push_str(&role_hints(p)); o
}

pub fn to_compact(p: &Personality) -> String {
    let mut o = format!("指纹: {} | 缺失: {}/84\n", p.fingerprint(), p.missing_count());
    for i in 0..84 {
        if p.missing()[i] { continue; }
        let dev = (p.values()[i] - 0.5).abs();
        if dev > 0.3 { o.push_str(&format!("  {} {}: {} ({:.2})\n", PARAMS[i].id, PARAMS[i].name, if p.values()[i] > 0.5 { "↑" } else { "↓" }, p.values()[i])); }
    }
    o
}

pub fn to_detailed(p: &Personality) -> String {
    let mut o = String::new();
    for i in 0..84 {
        let def = &PARAMS[i];
        let desc = if p.missing()[i] { "[缺失]".to_string() } else { def.describe_full(def.range.denormalize(p.values()[i])) };
        o.push_str(&format!("{} {}: {}\n", def.id, def.name, desc));
    }
    o
}

fn core_traits(p: &Personality) -> String {
    let mut o = String::new();
    let (c025, c033) = (p.get("C025"), p.get("C033"));
    if c025 > 0.6 && c033 > 0.5 { o.push_str("  • 外向亲和——主动接近他人，渴望温暖关系\n"); }
    else if c025 < 0.4 && c033 < 0.5 { o.push_str("  • 内向疏离——倾向独处，人际非核心需求\n"); }
    else if c025 > 0.6 && c033 < 0.5 { o.push_str("  • 外向工具型——主动社交但视人际为手段\n"); }
    let (b015, b021) = (p.get("B015"), p.get("B021"));
    if b015 > 0.6 && b021 > 0.5 { o.push_str("  • 高共情——易被感染，伤害他人后强烈内疚\n"); }
    else if b015 < 0.4 && b021 < 0.5 { o.push_str("  • 情感淡漠——不易感染，伤害他人较少内疚\n"); }
    let (c032, e051) = (p.get("C032"), p.get("E051"));
    if c032 > 0.6 && e051 > 0.5 { o.push_str("  • 使命型权力——权力是完成使命的工具\n"); }
    else if c032 > 0.6 { o.push_str("  • 权力导向——渴望影响和控制他人\n"); }
    else if e051 > 0.6 { o.push_str("  • 使命驱动——有清晰的人生目标\n"); }
    let (d040, c030) = (p.get("D040"), p.get("C030"));
    if d040 > 0.6 && c030 < 0.4 { o.push_str("  • 冲动攻击——高攻击倾向且缺乏控制\n"); }
    else if d040 > 0.6 && c030 > 0.6 { o.push_str("  • 克制攻击——有攻击倾向但能有效控制\n"); }
    else if d040 < 0.4 { o.push_str("  • 和平倾向——攻击基线较低\n"); }
    o.trim_end().to_string()
}

fn domain_desc(p: &Personality, dom: Domain) -> String {
    let mut o = String::new(); let mut n = 0;
    for def in params::by_domain(dom) {
        let i = def.index as usize;
        if p.missing()[i] { continue; }
        if (p.values()[i] - 0.5).abs() > 0.2 { o.push_str(&format!("  {}\n", def.describe_full(def.range.denormalize(p.values()[i])))); n += 1; }
    }
    if n == 0 { o.push_str("  [表现均衡，无显著极端特征]\n"); }
    o.trim_end().to_string()
}

fn role_hints(p: &Personality) -> String {
    let mut o = String::new();
    match p.get("C028") { v if v > 0.7 => o.push_str("  • 说话风格：独立自主，抵触被指挥\n"), v if v < 0.3 => o.push_str("  • 说话风格：顺从配合，接受安排\n"), _ => {} }
    match p.get("B017") { v if v > 0.7 => o.push_str("  • 情绪表达：容易羞耻，社交失误后长时间纠结\n"), v if v < 0.3 => o.push_str("  • 情绪表达：不在意他人眼光，恢复快\n"), _ => {} }
    let (c030, d039) = (p.get("C030"), p.get("D039"));
    if c030 < 0.3 { o.push_str("  • 决策风格：冲动型，快速决定立即行动\n"); }
    else if d039 > 0.7 { o.push_str("  • 决策风格：拖延型，决定后需较长时间启动\n"); }
    else { o.push_str("  • 决策风格：平衡型，思考与行动节奏合理\n"); }
    let (f061, f062) = (p.get("F061"), p.get("F062"));
    if f061 < 0.3 && f062 > 0.7 { o.push_str("  • 人际关系：高度警觉，默认不信任，时刻准备发现背叛\n"); }
    else if f061 > 0.7 { o.push_str("  • 人际关系：容易信任，对陌生人持开放态度\n"); }
    else { o.push_str("  • 人际关系：谨慎开放，需时间建立信任\n"); }
    o.trim_end().to_string()
}

#[cfg(test)] #[test] fn all_modes() { let p = crate::Generator::from_seed(42, None); assert!(to_roleplay(&p).contains("【人格档案】")); assert!(to_compact(&p).contains("指纹:")); assert!(to_detailed(&p).contains("A001")); }
