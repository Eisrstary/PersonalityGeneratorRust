//! 人格档案生成器 —— 生成一份完整的人格参数档案
//!
//! 这不是"人格类型"报告，而是参数在当前时刻的快照。
//! 包含：参数值、耦合分析、关系坍缩、ε声明。

use personality_generator::api::PersonalitySystem;
use rand::Rng;

fn main() {
    let mut system = PersonalitySystem::new();
    let mut rng = rand::thread_rng();

    // ============================================================
    // 模拟一个"人"的参数配置 —— 不是类型，只是此刻的取值
    // ============================================================

    // 领域A: 信息摄入
    system.set_value("A001", rng.gen_range(3.0..7.0)).unwrap();   // 视觉采样率 3-7Hz
    system.set_value("A002", rng.gen_range(1.0..8.0)).unwrap();   // 听觉歧义容忍 1-8s
    system.set_value("A003", rng.gen_range(0.3..0.8)).unwrap();   // 内感受分辨率
    system.set_value("A004", rng.gen_range(0.4..0.9)).unwrap();   // 社会性线索优先级
    system.set_value("A005", rng.gen_range(0.3..0.7)).unwrap();   // 新异刺激打断阈值
    system.set_value("A006", rng.gen_range(0.4..0.8)).unwrap();   // 背景-前景分离效率
    system.set_value("A007", rng.gen_range(0.2..0.6)).unwrap();   // 预期违背消耗
    system.set_value("A008", rng.gen_range(0.1..0.5)).unwrap();   // 威胁线索放大系数
    system.set_value("A009", rng.gen_range(0.4..0.8)).unwrap();   // 痛苦线索敏感度
    system.set_value("A010", rng.gen_range(-0.3..0.3)).unwrap();  // 猎物/捕食者注意偏向

    // 领域B: 情绪生成与调节
    system.set_value("B011", rng.gen_range(0.3..0.7)).unwrap();   // 基础情绪唤醒阈值
    system.set_value("B012", rng.gen_range(0.3..0.7)).unwrap();   // 情绪颗粒度
    system.set_value("B013", rng.gen_range(0.3..0.6)).unwrap();   // 自动思维情绪附着力
    system.set_value("B014", rng.gen_range(3.0..12.0)).unwrap();  // 情绪调节策略库
    system.set_value("B015", rng.gen_range(0.3..0.8)).unwrap();   // 内疚感基线
    system.set_value("B016", rng.gen_range(-0.8..0.2)).unwrap();  // 他人痛苦-自身愉悦转化
    system.set_value("B017", rng.gen_range(0.3..0.7)).unwrap();   // 羞耻感基线
    system.set_value("B018", rng.gen_range(0.3..0.7)).unwrap();   // 积极情绪维持能力
    system.set_value("B019", rng.gen_range(0.1..0.5)).unwrap();   // 愤怒-攻击转化率
    system.set_value("B020", rng.gen_range(1.0..6.0)).unwrap();   // 情绪标签命名速度
    system.set_value("B021", rng.gen_range(0.3..0.7)).unwrap();   // 情绪传染易感性
    system.set_value("B022", rng.gen_range(7.0..90.0)).unwrap();  // 怨恨衰减半衰期(天)
    system.set_value("B023", rng.gen_range(0.2..0.6)).unwrap();   // 嫉妒触发敏感度
    system.set_value("B024", rng.gen_range(0.1..0.5)).unwrap();   // 幸灾乐祸阈限

    // 领域C: 动机与价值
    system.set_value("C025", rng.gen_range(-0.2..0.5)).unwrap();  // 趋近-回避基线
    system.set_value("C026", rng.gen_range(3.0..30.0)).unwrap();  // 意义寻求强度
    system.set_value("C027", rng.gen_range(0.2..0.6)).unwrap();   // 延迟折扣率
    system.set_value("C028", rng.gen_range(0.3..0.7)).unwrap();   // 自主性需求
    system.set_value("C029", rng.gen_range(0.3..0.7)).unwrap();   // 胜任感锚点
    system.set_value("C030", rng.gen_range(2.0..15.0)).unwrap();  // 冲动控制缓冲(s)
    system.set_value("C031", rng.gen_range(-0.3..0.4)).unwrap();  // 支配-顺从倾向
    system.set_value("C032", rng.gen_range(0.2..0.6)).unwrap();   // 权力动机
    system.set_value("C033", rng.gen_range(0.3..0.8)).unwrap();   // 亲和动机
    system.set_value("C034", rng.gen_range(0.2..0.6)).unwrap();   // 地位渴求
    system.set_value("C035", rng.gen_range(0.3..0.7)).unwrap();   // 利他惩罚倾向
    system.set_value("C036", rng.gen_range(0.1..0.5)).unwrap();   // 欺骗接受度
    system.set_value("C037", rng.gen_range(0.4..0.8)).unwrap();   // 价值-行为一致性
    system.set_value("C038", rng.gen_range(0.2..0.6)).unwrap();   // 刺激寻求

    // 领域D: 行为执行
    system.set_value("D039", rng.gen_range(60.0..600.0)).unwrap(); // 行为蓄能时间(s)
    system.set_value("D040", rng.gen_range(0.0..0.3)).unwrap();   // 攻击行为基线
    system.set_value("D041", rng.gen_range(0.4..0.8)).unwrap();   // 规则遵循度
    system.set_value("D042", rng.gen_range(0.3..0.7)).unwrap();   // 行为灵活性

    // 领域E: 元认知与自我
    system.set_value("E043", rng.gen_range(0.2..0.6)).unwrap();   // 思维标签化频率
    system.set_value("E044", rng.gen_range(0.2..0.6)).unwrap();   // 反刍思维强度
    system.set_value("E045", rng.gen_range(-0.3..0.5)).unwrap();  // 内隐自尊
    system.set_value("E046", rng.gen_range(0.3..0.7)).unwrap();   // 外显自尊
    system.set_value("E047", rng.gen_range(0.3..0.7)).unwrap();   // 自我感知校准度
    system.set_value("E048", rng.gen_range(0.1..0.5)).unwrap();   // 道德推脱能力
    system.set_value("E049", rng.gen_range(-0.3..0.3)).unwrap();  // 责任归因偏向
    system.set_value("E050", rng.gen_range(0.3..0.7)).unwrap();   // 自我批评强度
    system.set_value("E051", rng.gen_range(0.1..0.6)).unwrap();   // 使命感清晰度
    system.set_value("E052", rng.gen_range(0.2..0.6)).unwrap();   // 道德-审美耦合度
    system.set_value("E053", rng.gen_range(3.0..30.0)).unwrap();  // 矛盾共存耐受(分钟)
    system.set_value("E054", rng.gen_range(0.3..0.7)).unwrap();   // 框架重构力
    system.set_value("E055", rng.gen_range(0.1..0.5)).unwrap();   // 自我欺骗强度

    // 领域F: 社交信号
    system.set_value("F056", rng.gen_range(100.0..500.0)).unwrap(); // 面部镜像延迟(ms)
    system.set_value("F057", rng.gen_range(0.4..0.9)).unwrap();   // 自我暴露深度梯度
    system.set_value("F058", rng.gen_range(0.3..0.7)).unwrap();   // 社交代价敏感度
    system.set_value("F059", rng.gen_range(0.2..0.6)).unwrap();   // 欺骗生理舒适度
    system.set_value("F060", rng.gen_range(0.3..0.7)).unwrap();   // 印象管理精细度
    system.set_value("F061", rng.gen_range(0.3..0.7)).unwrap();   // 信任默认值
    system.set_value("F062", rng.gen_range(0.2..0.6)).unwrap();   // 背叛检测灵敏度

    // 领域G: 时间性与发展
    system.set_value("G063", rng.gen_range(0.1..0.5)).unwrap();   // 参数漂移速率
    system.set_value("G064", rng.gen_range(0.3..0.7)).unwrap();   // 重大事件相变阈值
    system.set_value("G065", rng.gen_range(0.1..0.5)).unwrap();   // 情境人格切换幅度
    system.set_value("G066", rng.gen_range(30.0..1000.0)).unwrap(); // 身份叙事更新速率(天)

    // 领域H: 身体-环境耦合
    system.set_value("H067", rng.gen_range(0.2..0.6)).unwrap();   // 坐姿-思维关联
    system.set_value("H068", rng.gen_range(0.3..0.7)).unwrap();   // 呼吸-情绪耦联
    system.set_value("H069", rng.gen_range(0.3..0.8)).unwrap();   // 手势-语速锁定
    system.set_value("H070", rng.gen_range(0.2..0.6)).unwrap();   // 温度-社交距离
    system.set_value("H071", rng.gen_range(0.1..0.5)).unwrap();   // 饱腹-慷慨
    system.set_value("H072", rng.gen_range(0.3..0.7)).unwrap();   // 昼夜节律-创造力
    system.set_value("H073", rng.gen_range(0.3..0.7)).unwrap();   // 微表情抑制力
    system.set_value("H074", rng.gen_range(0.1..0.5)).unwrap();   // 疼痛-攻击链接
    system.set_value("H075", rng.gen_range(0.1..0.5)).unwrap();   // 光照-决策速度
    system.set_value("H076", rng.gen_range(0.3..0.8)).unwrap();   // 运动-情绪提升
    system.set_value("H077", rng.gen_range(0.3..0.7)).unwrap();   // 睡眠债务-认知衰减
    system.set_value("H078", rng.gen_range(0.2..0.6)).unwrap();   // 噪音-压力耦联
    system.set_value("H079", rng.gen_range(0.3..0.7)).unwrap();   // 气味-记忆唤起率
    system.set_value("H080", rng.gen_range(0.3..0.7)).unwrap();   // 触觉-信任关联
    system.set_value("H081", rng.gen_range(-0.3..0.4)).unwrap();  // 饥饿-风险偏好
    system.set_value("H082", rng.gen_range(0.2..0.6)).unwrap();   // 姿势-权力感映射
    system.set_value("H083", rng.gen_range(0.1..0.5)).unwrap();   // 温度-攻击性
    system.set_value("H084", rng.gen_range(0.1..0.4)).unwrap();   // 海拔-思维抽象度

    // ============================================================
    // 添加几个关系，展示关系坍缩
    // ============================================================
    system.add_relationship("家人", "intimate");
    system.add_relationship("同事", "acquaintance");
    system.add_relationship("陌生人", "stranger");
    system.add_relationship("竞争对手", "hostile");

    // ============================================================
    // 输出人格档案
    // ============================================================
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                   人格原子参数档案 (PAPS Profile)                            ║");
    println!("║                                                                            ║");
    println!("║  这不是\"人格类型\"。这不是\"你是怎样的人\"。                                      ║");
    println!("║  这是 84 个参数在此时此刻的取值快照。                                          ║");
    println!("║  这些值会在关系中坍缩、在时间里漂移、在情境中撕裂。                               ║");
    println!("║  不存在\"真正的你\"——只存在此参数空间中的一次不可复制的采样。                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // --- 领域A ---
    print_domain(&system, "领域A: 信息摄入 —— 世界如何进入这个系统", &[
        ("A001", "视觉采样率", "Hz"),
        ("A002", "听觉歧义容忍窗口", "s"),
        ("A003", "内感受分辨率", ""),
        ("A004", "社会性线索优先级", ""),
        ("A005", "新异刺激打断阈值", ""),
        ("A006", "背景-前景分离效率", ""),
        ("A007", "预期违背消耗", ""),
        ("A008", "威胁线索放大系数", ""),
        ("A009", "痛苦线索敏感度", ""),
        ("A010", "猎物/捕食者注意偏向", "±"),
    ]);

    // --- 领域B ---
    print_domain(&system, "领域B: 情绪生成与调节 —— 系统如何生成和调控情感状态", &[
        ("B011", "基础情绪唤醒阈值", ""),
        ("B012", "情绪颗粒度", ""),
        ("B013", "自动思维情绪附着力", ""),
        ("B014", "情绪调节策略库", "种"),
        ("B015", "内疚感基线", ""),
        ("B016", "他人痛苦-自身愉悦转化", "±"),
        ("B017", "羞耻感基线", ""),
        ("B018", "积极情绪维持能力", ""),
        ("B019", "愤怒-攻击转化率", ""),
        ("B020", "情绪标签命名速度", "s"),
        ("B021", "情绪传染易感性", ""),
        ("B022", "怨恨衰减半衰期", "天"),
        ("B023", "嫉妒触发敏感度", ""),
        ("B024", "幸灾乐祸阈限", ""),
    ]);

    // --- 领域C ---
    print_domain(&system, "领域C: 动机与价值 —— 什么驱动系统采取行动", &[
        ("C025", "趋近-回避基线", "±"),
        ("C026", "意义寻求强度", "次/天"),
        ("C027", "延迟折扣率", ""),
        ("C028", "自主性需求", ""),
        ("C029", "胜任感锚点", ""),
        ("C030", "冲动控制缓冲", "s"),
        ("C031", "支配-顺从倾向", "±"),
        ("C032", "权力动机", ""),
        ("C033", "亲和动机", ""),
        ("C034", "地位渴求", ""),
        ("C035", "利他惩罚倾向", ""),
        ("C036", "欺骗接受度", ""),
        ("C037", "价值-行为一致性", ""),
        ("C038", "刺激寻求", ""),
    ]);

    // --- 领域D ---
    print_domain(&system, "领域D: 行为执行 —— 系统如何将意图转化为行动", &[
        ("D039", "行为蓄能时间", "s"),
        ("D040", "攻击行为基线", ""),
        ("D041", "规则遵循度", ""),
        ("D042", "行为灵活性", ""),
    ]);

    // --- 领域E ---
    print_domain(&system, "领域E: 元认知与自我 —— 系统如何观察和定义自己", &[
        ("E043", "思维标签化频率", ""),
        ("E044", "反刍思维强度", ""),
        ("E045", "内隐自尊", "±"),
        ("E046", "外显自尊", ""),
        ("E047", "自我感知校准度", ""),
        ("E048", "道德推脱能力", ""),
        ("E049", "责任归因偏向", "±"),
        ("E050", "自我批评强度", ""),
        ("E051", "使命感清晰度", ""),
        ("E052", "道德-审美耦合度", ""),
        ("E053", "矛盾共存耐受", "分钟"),
        ("E054", "框架重构力", ""),
        ("E055", "自我欺骗强度", ""),
    ]);

    // --- 领域F ---
    print_domain(&system, "领域F: 社交信号 —— 系统如何发送和接收人际信息", &[
        ("F056", "面部镜像延迟", "ms"),
        ("F057", "自我暴露深度梯度", ""),
        ("F058", "社交代价敏感度", ""),
        ("F059", "欺骗生理舒适度", ""),
        ("F060", "印象管理精细度", ""),
        ("F061", "信任默认值", ""),
        ("F062", "背叛检测灵敏度", ""),
    ]);

    // --- 领域G ---
    print_domain(&system, "领域G: 时间性与发展 —— 参数如何随时间变化", &[
        ("G063", "参数漂移速率", ""),
        ("G064", "重大事件相变阈值", ""),
        ("G065", "情境人格切换幅度", ""),
        ("G066", "身份叙事更新速率", "天"),
    ]);

    // --- 领域H ---
    print_domain(&system, "领域H: 身体-环境耦合 —— 身体与环境如何交互影响", &[
        ("H067", "坐姿-思维关联", ""),
        ("H068", "呼吸-情绪耦联", ""),
        ("H069", "手势-语速锁定", ""),
        ("H070", "温度-社交距离", ""),
        ("H071", "饱腹-慷慨", ""),
        ("H072", "昼夜节律-创造力", ""),
        ("H073", "微表情抑制力", ""),
        ("H074", "疼痛-攻击链接", ""),
        ("H075", "光照-决策速度", ""),
        ("H076", "运动-情绪提升", ""),
        ("H077", "睡眠债务-认知衰减", ""),
        ("H078", "噪音-压力耦联", ""),
        ("H079", "气味-记忆唤起率", ""),
        ("H080", "触觉-信任关联", ""),
        ("H081", "饥饿-风险偏好", "±"),
        ("H082", "姿势-权力感映射", ""),
        ("H083", "温度-攻击性", ""),
        ("H084", "海拔-思维抽象度", ""),
    ]);

    // ============================================================
    // 耦合分析
    // ============================================================
    println!();
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                          参数耦合分析                                         │");
    println!("│  以下不是\"人格类型\"——只是当前参数值触发的耦合现象。                              │");
    println!("│  如果参数值改变，这些现象也会改变。                                              │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘");
    println!();

    let couplings = system.analyze_couplings();
    if couplings.is_empty() {
        println!("  (当前参数值组合未触发显著的耦合现象)");
    } else {
        for (i, c) in couplings.iter().enumerate() {
            let display_a = (c.value_a / 10.0).tanh().max(0.0).min(1.0);
            let display_b = (c.value_b / 10.0).tanh().max(0.0).min(1.0);
            let bar_a = "█".repeat((display_a * 15.0) as usize);
            let bar_b = "█".repeat((display_b * 15.0) as usize);
            println!("  [{:02}] {} ({} {:>6.2}) + {} ({} {:>6.2})", i + 1, c.param_a, bar_a, c.value_a, c.param_b, bar_b, c.value_b);
            println!("       → {}", c.phenomenon);
            println!();
        }
    }

    // ============================================================
    // 关系坍缩分析
    // ============================================================
    println!();
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                          关系中的参数坍缩                                       │");
    println!("│  同一个参数在不同关系中取不同值。这不是\"虚伪\"——这是参数的关系依赖性。               │");
    println!("│  Parameter(relationship) = Baseline × RelationModifier                       │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘");
    println!();

    let key_params = ["B015", "A009", "C033", "F061", "D040", "B021", "C031", "E046"];
    let relationships = ["家人", "同事", "陌生人", "竞争对手"];

    // 表头
    print!("  {:<8} {:<20}", "参数", "参数名");
    for rel in &relationships {
        print!(" | {:<10}", rel);
    }
    println!();
    println!("  {:-<80}", "");

    for pid in &key_params {
        let name = get_param_name(&system, pid);
        print!("  {:<8} {:<20}", pid, name);
        for rel in &relationships {
            let val = system.collapse_in_relationship(pid, rel).unwrap_or(-1.0);
            let display = (val.abs() / 5.0).tanh().max(0.0).min(1.0);
            let bar = if val >= 0.0 {
                "▓".repeat((display * 10.0) as usize)
            } else {
                "░".repeat((display * 10.0) as usize)
            };
            print!(" | {:>6.2} {}", val, bar);
        }
        println!();
    }

    // ============================================================
    // 跨关系方差最大的参数
    // ============================================================
    println!();
    println!("┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                    跨关系差异最大的参数 (Top 5)                                  │");
    println!("│  这些参数在不同关系中取值差异最大——它们是\"情境人格切换\"的核心参数。                │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘");
    println!();

    let variances = system.cross_relational_analysis(5);
    for (i, v) in variances.iter().enumerate() {
        let name = get_param_name(&system, &v.param_id);
        println!("  [{}. {}] {} (方差: {:.4})", i + 1, v.param_id, name, v.variance);
        for (rid, val) in &v.values {
            let display = (val.abs() / 50.0).tanh().max(0.0).min(1.0);
            let bar = "▓".repeat((display * 20.0) as usize);
            println!("      {:<10} {:>8.2} {}", rid, val, bar);
        }
        println!();
    }

    // ============================================================
    // ε 声明
    // ============================================================
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                          不可通约余数 ε                                       ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  ε = {:.4}                                                                  ║", system.epsilon_value());
    println!("║                                                                            ║");
    println!("║  即使所有84个参数都被精确测量，即使所有耦合关系都被理解，                           ║");
    println!("║  即使所有漂移/相变/反转都被建模——仍然存在 ε（不可通约余数）。                        ║");
    println!("║                                                                            ║");
    println!("║  ε 是参数无法捕捉的\"那个人的独特历史\"。                                         ║");
    println!("║  ε 是所有参数在特定时刻的不可复制的唯一组合。                                      ║");
    println!("║  ε 是自由意志(如果它存在)的数学表达。                                            ║");
    println!("║  ε 是人格不能被还原为参数的根本原因。                                            ║");
    println!("║  ε 是本系统的自我否定——也是本系统最重要的部分。                                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  —— 人格原子参数系统 (PAPS) v{} ——", personality_generator::VERSION);
    println!();
}

fn print_domain(system: &PersonalitySystem, title: &str, params: &[(&str, &str, &str)]) {
    println!("┌─ {} ─────────────────────────┐", title);
    println!();
    for (id, name, unit) in params {
        let val = system.get_value(id).unwrap_or(-1.0);
        // 对无界值做可视化缩放：用 tanh 映射到 [0, 1]
        let display_val = if *unit == "Hz" || *unit == "s" || *unit == "种" || *unit == "次/天" || *unit == "天" || *unit == "ms" || *unit == "分钟" {
            (val / 20.0).tanh().max(0.0).min(1.0)
        } else {
            val.abs().min(1.0)
        };
        let bar_len = if *unit == "±" {
            ((val + 1.0) / 2.0 * 30.0) as usize
        } else {
            (display_val * 30.0) as usize
        };
        let bar = if val >= 0.0 { "▓" } else { "░" };
        let bar_str = bar.repeat(bar_len);
        if unit.is_empty() {
            println!("  {:<6} {:<22} [{:<30}] {:>5.2}", id, name, bar_str, val);
        } else {
            println!("  {:<6} {:<22} [{:<30}] {:>5.2} {}", id, name, bar_str, val, unit);
        }
    }
    println!();
}

fn get_param_name(system: &PersonalitySystem, id: &str) -> String {
    system.get_parameter(id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| id.to_string())
}
