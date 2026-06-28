//! 84 个原子参数定义。
//!
//! 每个参数是一个独立的心理功能维度，用编译期常量表达。
//! 参数 ≠ 特质：参数描述特定时刻快照，承认情境崩塌和时间漂移。

// ═══════════════════════════════════════════════════════════════
// Domain —— 8 个领域
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// A: 信息摄入
    Intake,
    /// B: 情绪生成与调节
    Emotion,
    /// C: 动机与价值
    Motivation,
    /// D: 行为执行
    Action,
    /// E: 元认知与自我
    MetaCognition,
    /// F: 社交信号
    Social,
    /// G: 时间性与发展
    Temporal,
    /// H: 身体-环境耦合
    Somatic,
}

impl Domain {
    pub const ALL: [Domain; 8] = [
        Self::Intake, Self::Emotion, Self::Motivation, Self::Action,
        Self::MetaCognition, Self::Social, Self::Temporal, Self::Somatic,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Intake => "信息处理",
            Self::Emotion => "情绪模式",
            Self::Motivation => "动机与价值观",
            Self::Action => "行为风格",
            Self::MetaCognition => "自我认知",
            Self::Social => "社交特征",
            Self::Temporal => "发展特征",
            Self::Somatic => "身体-环境反应",
        }
    }

    pub const fn code(self) -> char {
        match self {
            Self::Intake => 'A', Self::Emotion => 'B',
            Self::Motivation => 'C', Self::Action => 'D',
            Self::MetaCognition => 'E', Self::Social => 'F',
            Self::Temporal => 'G', Self::Somatic => 'H',
        }
    }

    pub fn from_code(c: char) -> Option<Self> {
        match c {
            'A' | 'a' => Some(Self::Intake), 'B' | 'b' => Some(Self::Emotion),
            'C' | 'c' => Some(Self::Motivation), 'D' | 'd' => Some(Self::Action),
            'E' | 'e' => Some(Self::MetaCognition), 'F' | 'f' => Some(Self::Social),
            'G' | 'g' => Some(Self::Temporal), 'H' | 'h' => Some(Self::Somatic),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// ParamDef —— 编译期常量
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy)]
pub struct ParamDef {
    pub index: u8,
    pub id: &'static str,
    pub domain: Domain,
    pub name: &'static str,
    pub low_desc: &'static str,
    pub high_desc: &'static str,
    /// 原始值范围
    pub range: Range,
    /// 是否为双极（跨越零点的 [-a, +b] 范围）
    pub bipolar: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    pub const fn new(min: f64, max: f64) -> Self { Self { min, max } }
    pub const fn symmetric(half: f64) -> Self { Self { min: -half, max: half } }
    pub const fn unit() -> Self { Self { min: 0.0, max: 1.0 } }
    pub fn span(&self) -> f64 { self.max - self.min }
    pub fn mid(&self) -> f64 { (self.min + self.max) / 2.0 }
    pub fn clamp(&self, v: f64) -> f64 { v.clamp(self.min, self.max) }
    /// 归一化 raw → [0, 1]
    pub fn normalize(&self, raw: f64) -> f64 {
        let s = self.span();
        if s > 0.0 { (raw - self.min) / s } else { 0.5 }
    }
    /// 反归一化 [0, 1] → raw
    pub fn denormalize(&self, normalized: f64) -> f64 {
        self.min + normalized * self.span()
    }
}

impl ParamDef {
    pub fn describe(&self, raw: f64) -> &'static str {
        let n = self.range.normalize(raw);
        if n < 0.2 { return "[极低]" }
        if n < 0.4 { return "[偏低]" }
        if n < 0.6 { return "[中等]" }
        if n < 0.8 { return "[偏高]" }
        "[极高]"
    }

    pub fn describe_full(&self, raw: f64) -> String {
        let n = self.range.normalize(raw);
        let label = if n < 0.2 { "[极低]" } else if n < 0.4 { "[偏低]" }
            else if n < 0.6 { "[中等]" } else if n < 0.8 { "[偏高]" }
            else { "[极高]" };
        if n >= 0.4 && n < 0.6 {
            format!("{label} {low}与{high}之间", low = self.low_desc, high = self.high_desc)
        } else if n < 0.5 {
            format!("{label} {low}", low = self.low_desc)
        } else {
            format!("{label} {high}", high = self.high_desc)
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 84 参数静态表 —— 编译期完全确定
// ═══════════════════════════════════════════════════════════════

macro_rules! param {
    ($idx:expr, $id:literal, $domain:ident, $name:literal, $low:literal, $high:literal, $min:expr, $max:expr, $bipolar:expr) => {
        ParamDef {
            index: $idx,
            id: $id,
            domain: Domain::$domain,
            name: $name,
            low_desc: $low,
            high_desc: $high,
            range: Range::new($min, $max),
            bipolar: $bipolar,
        }
    };
}

/// 84 个参数，编译期常量数组。索引 0..83。
pub static PARAMS: [ParamDef; 84] = [
    // ═══ A: 信息摄入 (0..9) ═══
    param!( 0, "A001", Intake, "视觉采样率", "凝视锁定(1Hz)", "高速扫描(10Hz)", 1.0, 10.0, false),
    param!( 1, "A002", Intake, "听觉歧义容忍", "立即消歧(0s)", "无限悬置(10s)", 0.0, 10.0, false),
    param!( 2, "A003", Intake, "内感受分辨率", "完全不觉察", "每个心跳都清晰感知", 0.0, 1.0, false),
    param!( 3, "A004", Intake, "社会性线索优先级", "面孔=物体", "面孔自动捕获注意", 0.0, 1.0, false),
    param!( 4, "A005", Intake, "新异刺激打断阈值", "雷打不动(80dB)", "落叶惊心(20dB)", 20.0, 80.0, false),
    param!( 5, "A006", Intake, "背景-前景分离效率", "淹没在噪音中(500ms)", "鸡尾酒效应大师(50ms)", 50.0, 500.0, false),
    param!( 6, "A007", Intake, "预期违背消耗", "意外=无所谓(0%)", "意外=认知地震(100%)", 0.0, 100.0, false),
    param!( 7, "A008", Intake, "威胁线索放大系数", "威胁=客观威胁", "中性表情=敌意信号", 0.0, 1.0, false),
    param!( 8, "A009", Intake, "痛苦线索敏感度", "他人痛苦=背景噪音", "他人皱眉=自己心痛", 0.0, 1.0, false),
    param!( 9, "A010", Intake, "猎物/捕食者注意偏向", "注意流向弱者", "注意流向强者", -1.0, 1.0, true),

    // ═══ B: 情绪 (10..23) ═══
    param!(10, "B011", Emotion, "基础情绪唤醒阈值", "极易唤醒", "极难唤醒", 0.0, 1.0, false),
    param!(11, "B012", Emotion, "情绪颗粒度", "感觉=一团糟", "恼火/愤懑/愠怒分明", 0.0, 100.0, false),
    param!(12, "B013", Emotion, "自动思维情绪附着力", "想法=纯认知", "一个想法=情绪炸弹", 0.0, 1.0, false),
    param!(13, "B014", Emotion, "情绪调节策略库", "只有本能反应", "20+种主动调节方法", 0.0, 20.0, false),
    param!(14, "B015", Emotion, "内疚感基线", "伤害他人=完全无感", "伤害他人=自我折磨", 0.0, 1.0, false),
    param!(15, "B016", Emotion, "他人痛苦-自身愉悦转化", "他人痛苦=不适", "他人痛苦=愉悦", -1.0, 1.0, true),
    param!(16, "B017", Emotion, "羞耻感基线", "出丑=无所谓", "出丑=想消失", 0.0, 1.0, false),
    param!(17, "B018", Emotion, "积极情绪维持能力", "快乐=瞬间", "快乐=持续一整天", 0.0, 1.0, false),
    param!(18, "B019", Emotion, "愤怒-攻击转化率", "愤怒=内心体验", "愤怒=立即行动", 0.0, 1.0, false),
    param!(19, "B020", Emotion, "情绪标签命名速度", "说不出感受(10s)", "瞬间精准命名(0.5s)", 0.5, 10.0, false),
    param!(20, "B021", Emotion, "情绪传染易感性", "他人哭泣=干眼", "他人哭泣=立刻泪崩", 0.0, 1.0, false),
    param!(21, "B022", Emotion, "怨恨衰减半衰期", "冒犯=秒忘(0天)", "冒犯=终身铭记(3650天)", 0.0, 3650.0, false),
    param!(22, "B023", Emotion, "嫉妒触发敏感度", "他人优势=完全无感", "微小差距=嫉妒燃烧", 0.0, 1.0, false),
    param!(23, "B024", Emotion, "幸灾乐祸阈限", "他人不幸=不适", "微小不幸=暗喜", 0.0, 1.0, false),

    // ═══ C: 动机与价值 (24..37) ═══
    param!(24, "C025", Motivation, "趋近-回避基线", "默认后撤", "默认前倾", -1.0, 1.0, true),
    param!(25, "C026", Motivation, "意义寻求强度", "活着就好", "每件事都追问意义", 0.0, 100.0, false),
    param!(26, "C027", Motivation, "延迟折扣率", "只要现在(1.0)", "全押未来(0.0)", 0.0, 1.0, false),
    param!(27, "C028", Motivation, "自主性需求", "被指令=舒适", "被指令=本能反抗", 0.0, 100.0, false),
    param!(28, "C029", Motivation, "胜任感锚点", "永远不够好", "做了一点就够了", 0.0, 100.0, false),
    param!(29, "C030", Motivation, "冲动控制缓冲", "冲动=行动(0s)", "冲动…缓冲…行动(300s)", 0.0, 300.0, false),
    param!(30, "C031", Motivation, "支配-顺从倾向", "自愿服从", "必须主导", -1.0, 1.0, true),
    param!(31, "C032", Motivation, "权力动机", "影响他人=无感", "控制他人=核心驱力", 0.0, 1.0, false),
    param!(32, "C033", Motivation, "亲和动机", "人际=工具", "人际=目的", 0.0, 1.0, false),
    param!(33, "C034", Motivation, "地位渴求", "地位=无所谓", "地位=生命意义", 0.0, 1.0, false),
    param!(34, "C035", Motivation, "利他惩罚倾向", "不公=无视", "不公=自掏成本也要罚", 0.0, 1.0, false),
    param!(35, "C036", Motivation, "欺骗接受度", "谎言=不可接受", "谎言=合理工具", 0.0, 1.0, false),
    param!(36, "C037", Motivation, "价值-行为一致性", "说的≠做的", "言行完全一致", 0.0, 1.0, false),
    param!(37, "C038", Motivation, "刺激寻求", "平静=理想", "刺激=必需", 0.0, 1.0, false),

    // ═══ D: 行为执行 (38..41) ═══
    param!(38, "D039", Action, "行为蓄能时间", "决定=行动(0s)", "决定…(∞)…行动(3600s)", 0.0, 3600.0, false),
    param!(39, "D040", Action, "攻击行为基线", "从不攻击", "主动攻击", 0.0, 1.0, false),
    param!(40, "D041", Action, "规则遵循度", "规则=建议", "规则=铁律", 0.0, 1.0, false),
    param!(41, "D042", Action, "行为灵活性", "受阻=卡死", "受阻=秒换方案", 0.0, 1.0, false),

    // ═══ E: 元认知与自我 (42..54) ═══
    param!(42, "E043", MetaCognition, "思维标签化频率", "思维=透明", "频繁观察自己的思维", 0.0, 1.0, false),
    param!(43, "E044", MetaCognition, "反刍思维强度", "负面经历=翻篇", "负面经历=无限循环", 0.0, 1.0, false),
    param!(44, "E045", MetaCognition, "内隐自尊", "潜意识自我=负面", "潜意识自我=正面", -1.0, 1.0, true),
    param!(45, "E046", MetaCognition, "外显自尊", "声称的自我价值=低", "声称的自我价值=高", 0.0, 1.0, false),
    param!(46, "E047", MetaCognition, "自我感知校准度", "自我评价=严重偏差", "自我评价=客观精准", 0.0, 100.0, false),
    param!(47, "E048", MetaCognition, "道德推脱能力", "错=错", "错=可合理化", 0.0, 1.0, false),
    param!(48, "E049", MetaCognition, "责任归因偏向", "问题=我", "问题=世界", -1.0, 1.0, true),
    param!(49, "E050", MetaCognition, "自我批评强度", "错误=无视", "错误=自我鞭笞", 0.0, 1.0, false),
    param!(50, "E051", MetaCognition, "使命感清晰度", "为何而活=？", "为何而活=！", 0.0, 1.0, false),
    param!(51, "E052", MetaCognition, "道德-审美耦合度", "善≠美", "善=美", 0.0, 1.0, false),
    param!(52, "E053", MetaCognition, "矛盾共存耐受", "冲突=必须解决", "冲突=可以共存", 0.0, 1440.0, false),
    param!(53, "E054", MetaCognition, "框架重构力", "失败=失败", "失败=数据", 0.0, 1.0, false),
    param!(54, "E055", MetaCognition, "自我欺骗强度", "对自己诚实", "完全相信自己编织的谎言", 0.0, 1.0, false),

    // ═══ F: 社交信号 (55..61) ═══
    param!(55, "F056", Social, "面部镜像延迟", "对方笑=瞬间同步(0ms)", "对方笑=无反应(2000ms)", 0.0, 2000.0, false),
    param!(56, "F057", Social, "自我暴露深度梯度", "初次见面=全盘托出", "十年好友=仍设防", 0.0, 100.0, false),
    param!(57, "F058", Social, "社交代价敏感度", "说'不'=轻松", "说'不'前模拟N种反应", 0.0, 1.0, false),
    param!(58, "F059", Social, "欺骗生理舒适度", "说谎=心跳加速", "说谎=心率完全平稳", 0.0, 1.0, false),
    param!(59, "F060", Social, "印象管理精细度", "不在乎形象", "精心设计每一面", 0.0, 1.0, false),
    param!(60, "F061", Social, "信任默认值", "陌生人=敌人", "陌生人=朋友", 0.0, 1.0, false),
    param!(61, "F062", Social, "背叛检测灵敏度", "利用=看不见", "蛛丝马迹=警觉", 0.0, 1.0, false),

    // ═══ G: 时间性与发展 (62..65) ═══
    param!(62, "G063", Temporal, "参数漂移速率", "人格=固定", "人格=流动", 0.0, 1.0, false),
    param!(63, "G064", Temporal, "重大事件相变阈值", "什么事都改不了我", "小事也能改变我", 0.0, 100.0, false),
    param!(64, "G065", Temporal, "情境人格切换幅度", "在家=在职场", "在家≠在职场/判若两人", 0.0, 100.0, false),
    param!(65, "G066", Temporal, "身份叙事更新速率", "自我定义=固定", "自我定义=持续重写", 0.0, 1.0, false),

    // ═══ H: 身体-环境耦合 (66..83) ═══
    param!(66, "H067", Somatic, "坐姿-思维关联", "驼背=无关", "驼背=消极想法↑", 0.0, 100.0, false),
    param!(67, "H068", Somatic, "呼吸-情绪耦联", "呼吸=自主", "呼吸=情绪晴雨表", -1.0, 1.0, true),
    param!(68, "H069", Somatic, "手势-语速锁定", "手势=随机", "手势=语音同步", 0.0, 500.0, false),
    param!(69, "H070", Somatic, "温度-社交距离", "冷=无关", "冷=想靠近", 0.0, 100.0, false),
    param!(70, "H071", Somatic, "饱腹-慷慨系数", "饿=自私", "饱=慷慨", -1.0, 1.0, true),
    param!(71, "H072", Somatic, "昼夜节律-创造力", "创造力=恒定", "创造力=时段函数", -100.0, 100.0, false),
    param!(72, "H073", Somatic, "微表情抑制力", "表情=自动", "表情=可控", 0.0, 1.0, false),
    param!(73, "H074", Somatic, "疼痛-攻击链接", "痛=痛", "痛=想攻击", 0.0, 1.0, false),
    param!(74, "H075", Somatic, "光照-决策速度", "暗=决策不变", "暗=决策延迟", 0.0, 60.0, false),
    param!(75, "H076", Somatic, "运动-情绪提升", "运动=纯身体", "运动=情绪药", 0.0, 200.0, false),
    param!(76, "H077", Somatic, "睡眠债务-认知衰减", "少睡=无影响", "少睡=认知崩溃", 0.0, 50.0, false),
    param!(77, "H078", Somatic, "噪音-压力耦联", "噪音=背景", "噪音=皮质醇↑", 0.0, 100.0, false),
    param!(78, "H079", Somatic, "气味-记忆唤起率", "气味=气味", "气味=时光机", 0.0, 1.0, false),
    param!(79, "H080", Somatic, "触觉-信任关联", "被触=无关", "被触=信任变化", -1.0, 1.0, true),
    param!(80, "H081", Somatic, "饥饿-风险偏好", "饿=保守", "饿=冒险", -1.0, 1.0, true),
    param!(81, "H082", Somatic, "姿势-权力感映射", "姿势=姿势", "扩张姿势=睾酮↑", 0.0, 100.0, false),
    param!(82, "H083", Somatic, "温度-攻击性", "热=无影响", "热=攻击↑", 0.0, 100.0, false),
    param!(83, "H084", Somatic, "海拔-思维抽象度", "高度=无关", "高度=抽象思维↑", -1.0, 1.0, true),
];

/// 编译期常量：参数总数。
pub const PARAM_COUNT: usize = PARAMS.len();

/// 编译期常量：双极参数列表。
pub const BIPOLAR_PARAMS: [usize; 11] = [9, 15, 24, 30, 44, 48, 67, 70, 79, 80, 83];

/// 按 ID 查找参数。O(n) 但 n=84，编译器可能完全展开。
pub fn by_id(id: &str) -> Option<&'static ParamDef> {
    PARAMS.iter().find(|p| p.id.eq_ignore_ascii_case(id))
}

/// 按索引获取参数。编译期可内联。
pub const fn by_index(idx: usize) -> Option<&'static ParamDef> {
    if idx < PARAM_COUNT { Some(&PARAMS[idx]) } else { None }
}

/// 按领域过滤参数。
pub fn by_domain(domain: Domain) -> impl Iterator<Item = &'static ParamDef> {
    PARAMS.iter().filter(move |p| p.domain == domain)
}

/// 构建 ID→索引 的快速查找表（一次性）。
pub fn index_map() -> std::collections::HashMap<&'static str, usize> {
    PARAMS.iter().map(|p| (p.id, p.index as usize)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count() { assert_eq!(PARAMS.len(), 84); }

    #[test]
    fn lookup() {
        assert_eq!(by_id("A001").unwrap().name, "视觉采样率");
        assert_eq!(by_id("h084").unwrap().name, "海拔-思维抽象度");
        assert!(by_id("ZZZZ").is_none());
    }

    #[test]
    fn index_consistency() {
        for (i, p) in PARAMS.iter().enumerate() {
            assert_eq!(p.index as usize, i, "{} index mismatch", p.id);
        }
    }

    #[test]
    fn domain_ranges() {
        assert_eq!(by_domain(Domain::Intake).count(), 10);
        assert_eq!(by_domain(Domain::Emotion).count(), 14);
        assert_eq!(by_domain(Domain::Motivation).count(), 14);
        assert_eq!(by_domain(Domain::Action).count(), 4);
        assert_eq!(by_domain(Domain::MetaCognition).count(), 13);
        assert_eq!(by_domain(Domain::Social).count(), 7);
        assert_eq!(by_domain(Domain::Temporal).count(), 4);
        assert_eq!(by_domain(Domain::Somatic).count(), 18);
    }

    #[test]
    fn bipolar_count() {
        let n = PARAMS.iter().filter(|p| p.bipolar).count();
        assert_eq!(n, 11);
    }

    #[test]
    fn range_valid() {
        for p in &PARAMS {
            assert!(p.range.min < p.range.max, "{} range invalid", p.id);
            if p.bipolar {
                assert!(p.range.min <= 0.0 && p.range.max >= 0.0, "{} bipolar but zero not in range", p.id);
            }
        }
    }
}
