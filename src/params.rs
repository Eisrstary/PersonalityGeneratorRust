//! 84 个原子参数定义。
//!
//! 每个参数描述一个独立的心理功能维度，分为 A-H 八个领域。
//! 参数 ≠ 特质：参数描述特定时刻的取值，承认情境崩塌和漂移。

/// 单个参数的定义。
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// 参数 ID，如 "A001"
    pub id: &'static str,
    /// 所属领域：'A'~'H'
    pub domain: char,
    /// 参数名称
    pub name: &'static str,
    /// 低端描述
    pub low_desc: &'static str,
    /// 高端描述
    pub high_desc: &'static str,
    /// 原始值下限
    pub min: f64,
    /// 原始值上限
    pub max: f64,
    /// 是否为双极参数（取值范围含负数）
    pub bipolar: bool,
}

impl ParamDef {
    /// 根据原始值生成中文描述。
    pub fn describe(&self, raw: f64) -> String {
        let n = (raw - self.min) / (self.max - self.min);
        if n < 0.2 {
            format!("[极低] {}", self.low_desc)
        } else if n < 0.4 {
            format!("[偏低] {}", self.low_desc)
        } else if n < 0.6 {
            format!("[中等] {}与{}之间", self.low_desc, self.high_desc)
        } else if n < 0.8 {
            format!("[偏高] {}", self.high_desc)
        } else {
            format!("[极高] {}", self.high_desc)
        }
    }
}

/// 全部 84 个参数。使用静态数组保证编译期大小确定。
pub static ALL_PARAMS: [ParamDef; 84] = [
    // ═══ A: 信息摄入 (A001-A010) ═══
    ParamDef { id: "A001", domain: 'A', name: "视觉采样率", low_desc: "凝视锁定(1Hz)", high_desc: "高速扫描(10Hz)", min: 1.0, max: 10.0, bipolar: false },
    ParamDef { id: "A002", domain: 'A', name: "听觉歧义容忍", low_desc: "立即消歧(0s)", high_desc: "无限悬置(10s)", min: 0.0, max: 10.0, bipolar: false },
    ParamDef { id: "A003", domain: 'A', name: "内感受分辨率", low_desc: "完全不觉察", high_desc: "每个心跳都清晰感知", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "A004", domain: 'A', name: "社会性线索优先级", low_desc: "面孔=物体", high_desc: "面孔自动捕获注意", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "A005", domain: 'A', name: "新异刺激打断阈值", low_desc: "雷打不动(80dB)", high_desc: "落叶惊心(20dB)", min: 20.0, max: 80.0, bipolar: false },
    ParamDef { id: "A006", domain: 'A', name: "背景-前景分离效率", low_desc: "淹没在噪音中(500ms)", high_desc: "鸡尾酒效应大师(50ms)", min: 50.0, max: 500.0, bipolar: false },
    ParamDef { id: "A007", domain: 'A', name: "预期违背消耗", low_desc: "意外=无所谓(0%)", high_desc: "意外=认知地震(100%)", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "A008", domain: 'A', name: "威胁线索放大系数", low_desc: "威胁=客观威胁", high_desc: "中性表情=敌意信号", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "A009", domain: 'A', name: "痛苦线索敏感度", low_desc: "他人痛苦=背景噪音", high_desc: "他人皱眉=自己心痛", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "A010", domain: 'A', name: "猎物/捕食者注意偏向", low_desc: "注意流向弱者", high_desc: "注意流向强者", min: -1.0, max: 1.0, bipolar: true },

    // ═══ B: 情绪 (B011-B024) ═══
    ParamDef { id: "B011", domain: 'B', name: "基础情绪唤醒阈值", low_desc: "极易唤醒", high_desc: "极难唤醒", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B012", domain: 'B', name: "情绪颗粒度", low_desc: "感觉=一团糟", high_desc: "恼火/愤懑/愠怒分明", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "B013", domain: 'B', name: "自动思维情绪附着力", low_desc: "想法=纯认知", high_desc: "一个想法=情绪炸弹", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B014", domain: 'B', name: "情绪调节策略库", low_desc: "只有本能反应", high_desc: "20+种主动调节方法", min: 0.0, max: 20.0, bipolar: false },
    ParamDef { id: "B015", domain: 'B', name: "内疚感基线", low_desc: "伤害他人=完全无感", high_desc: "伤害他人=自我折磨", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B016", domain: 'B', name: "他人痛苦-自身愉悦转化", low_desc: "他人痛苦=不适", high_desc: "他人痛苦=愉悦", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "B017", domain: 'B', name: "羞耻感基线", low_desc: "出丑=无所谓", high_desc: "出丑=想消失", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B018", domain: 'B', name: "积极情绪维持能力", low_desc: "快乐=瞬间", high_desc: "快乐=持续一整天", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B019", domain: 'B', name: "愤怒-攻击转化率", low_desc: "愤怒=内心体验", high_desc: "愤怒=立即行动", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B020", domain: 'B', name: "情绪标签命名速度", low_desc: "说不出感受(10s)", high_desc: "瞬间精准命名(0.5s)", min: 0.5, max: 10.0, bipolar: false },
    ParamDef { id: "B021", domain: 'B', name: "情绪传染易感性", low_desc: "他人哭泣=干眼", high_desc: "他人哭泣=立刻泪崩", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B022", domain: 'B', name: "怨恨衰减半衰期", low_desc: "冒犯=秒忘(0天)", high_desc: "冒犯=终身铭记(3650天)", min: 0.0, max: 3650.0, bipolar: false },
    ParamDef { id: "B023", domain: 'B', name: "嫉妒触发敏感度", low_desc: "他人优势=完全无感", high_desc: "微小差距=嫉妒燃烧", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "B024", domain: 'B', name: "幸灾乐祸阈限", low_desc: "他人不幸=不适", high_desc: "微小不幸=暗喜", min: 0.0, max: 1.0, bipolar: false },

    // ═══ C: 动机与价值 (C025-C038) ═══
    ParamDef { id: "C025", domain: 'C', name: "趋近-回避基线", low_desc: "默认后撤", high_desc: "默认前倾", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "C026", domain: 'C', name: "意义寻求强度", low_desc: "活着就好", high_desc: "每件事都追问意义", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "C027", domain: 'C', name: "延迟折扣率", low_desc: "只要现在(1.0)", high_desc: "全押未来(0.0)", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C028", domain: 'C', name: "自主性需求", low_desc: "被指令=舒适", high_desc: "被指令=本能反抗", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "C029", domain: 'C', name: "胜任感锚点", low_desc: "永远不够好", high_desc: "做了一点就够了", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "C030", domain: 'C', name: "冲动控制缓冲", low_desc: "冲动=行动(0s)", high_desc: "冲动…缓冲…行动(300s)", min: 0.0, max: 300.0, bipolar: false },
    ParamDef { id: "C031", domain: 'C', name: "支配-顺从倾向", low_desc: "自愿服从", high_desc: "必须主导", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "C032", domain: 'C', name: "权力动机", low_desc: "影响他人=无感", high_desc: "控制他人=核心驱力", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C033", domain: 'C', name: "亲和动机", low_desc: "人际=工具", high_desc: "人际=目的", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C034", domain: 'C', name: "地位渴求", low_desc: "地位=无所谓", high_desc: "地位=生命意义", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C035", domain: 'C', name: "利他惩罚倾向", low_desc: "不公=无视", high_desc: "不公=自掏成本也要罚", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C036", domain: 'C', name: "欺骗接受度", low_desc: "谎言=不可接受", high_desc: "谎言=合理工具", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C037", domain: 'C', name: "价值-行为一致性", low_desc: "说的≠做的", high_desc: "言行完全一致", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "C038", domain: 'C', name: "刺激寻求", low_desc: "平静=理想", high_desc: "刺激=必需", min: 0.0, max: 1.0, bipolar: false },

    // ═══ D: 行为执行 (D039-D042) ═══
    ParamDef { id: "D039", domain: 'D', name: "行为蓄能时间", low_desc: "决定=行动(0s)", high_desc: "决定…(∞)…行动(3600s)", min: 0.0, max: 3600.0, bipolar: false },
    ParamDef { id: "D040", domain: 'D', name: "攻击行为基线", low_desc: "从不攻击", high_desc: "主动攻击", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "D041", domain: 'D', name: "规则遵循度", low_desc: "规则=建议", high_desc: "规则=铁律", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "D042", domain: 'D', name: "行为灵活性", low_desc: "受阻=卡死", high_desc: "受阻=秒换方案", min: 0.0, max: 1.0, bipolar: false },

    // ═══ E: 元认知与自我 (E043-E055) ═══
    ParamDef { id: "E043", domain: 'E', name: "思维标签化频率", low_desc: "思维=透明", high_desc: "频繁观察自己的思维", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E044", domain: 'E', name: "反刍思维强度", low_desc: "负面经历=翻篇", high_desc: "负面经历=无限循环", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E045", domain: 'E', name: "内隐自尊", low_desc: "潜意识自我=负面", high_desc: "潜意识自我=正面", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "E046", domain: 'E', name: "外显自尊", low_desc: "声称的自我价值=低", high_desc: "声称的自我价值=高", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E047", domain: 'E', name: "自我感知校准度", low_desc: "自我评价=严重偏差", high_desc: "自我评价=客观精准", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "E048", domain: 'E', name: "道德推脱能力", low_desc: "错=错", high_desc: "错=可合理化", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E049", domain: 'E', name: "责任归因偏向", low_desc: "问题=我", high_desc: "问题=世界", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "E050", domain: 'E', name: "自我批评强度", low_desc: "错误=无视", high_desc: "错误=自我鞭笞", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E051", domain: 'E', name: "使命感清晰度", low_desc: "为何而活=？", high_desc: "为何而活=！", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E052", domain: 'E', name: "道德-审美耦合度", low_desc: "善≠美", high_desc: "善=美", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E053", domain: 'E', name: "矛盾共存耐受", low_desc: "冲突=必须解决", high_desc: "冲突=可以共存", min: 0.0, max: 1440.0, bipolar: false },
    ParamDef { id: "E054", domain: 'E', name: "框架重构力", low_desc: "失败=失败", high_desc: "失败=数据", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "E055", domain: 'E', name: "自我欺骗强度", low_desc: "对自己诚实", high_desc: "完全相信自己编织的谎言", min: 0.0, max: 1.0, bipolar: false },

    // ═══ F: 社交信号 (F056-F062) ═══
    ParamDef { id: "F056", domain: 'F', name: "面部镜像延迟", low_desc: "对方笑=瞬间同步(0ms)", high_desc: "对方笑=无反应(2000ms)", min: 0.0, max: 2000.0, bipolar: false },
    ParamDef { id: "F057", domain: 'F', name: "自我暴露深度梯度", low_desc: "初次见面=全盘托出", high_desc: "十年好友=仍设防", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "F058", domain: 'F', name: "社交代价敏感度", low_desc: "说'不'=轻松", high_desc: "说'不'前模拟N种反应", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "F059", domain: 'F', name: "欺骗生理舒适度", low_desc: "说谎=心跳加速", high_desc: "说谎=心率完全平稳", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "F060", domain: 'F', name: "印象管理精细度", low_desc: "不在乎形象", high_desc: "精心设计每一面", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "F061", domain: 'F', name: "信任默认值", low_desc: "陌生人=敌人", high_desc: "陌生人=朋友", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "F062", domain: 'F', name: "背叛检测灵敏度", low_desc: "利用=看不见", high_desc: "蛛丝马迹=警觉", min: 0.0, max: 1.0, bipolar: false },

    // ═══ G: 时间性与发展 (G063-G066) ═══
    ParamDef { id: "G063", domain: 'G', name: "参数漂移速率", low_desc: "人格=固定", high_desc: "人格=流动", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "G064", domain: 'G', name: "重大事件相变阈值", low_desc: "什么事都改不了我", high_desc: "小事也能改变我", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "G065", domain: 'G', name: "情境人格切换幅度", low_desc: "在家=在职场", high_desc: "在家≠在职场/判若两人", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "G066", domain: 'G', name: "身份叙事更新速率", low_desc: "自我定义=固定", high_desc: "自我定义=持续重写", min: 0.0, max: 1.0, bipolar: false },

    // ═══ H: 身体-环境耦合 (H067-H084) ═══
    ParamDef { id: "H067", domain: 'H', name: "坐姿-思维关联", low_desc: "驼背=无关", high_desc: "驼背=消极想法↑", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "H068", domain: 'H', name: "呼吸-情绪耦联", low_desc: "呼吸=自主", high_desc: "呼吸=情绪晴雨表", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "H069", domain: 'H', name: "手势-语速锁定", low_desc: "手势=随机", high_desc: "手势=语音同步", min: 0.0, max: 500.0, bipolar: false },
    ParamDef { id: "H070", domain: 'H', name: "温度-社交距离", low_desc: "冷=无关", high_desc: "冷=想靠近", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "H071", domain: 'H', name: "饱腹-慷慨系数", low_desc: "饿=自私", high_desc: "饱=慷慨", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "H072", domain: 'H', name: "昼夜节律-创造力", low_desc: "创造力=恒定", high_desc: "创造力=时段函数", min: -100.0, max: 100.0, bipolar: false },
    ParamDef { id: "H073", domain: 'H', name: "微表情抑制力", low_desc: "表情=自动", high_desc: "表情=可控", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "H074", domain: 'H', name: "疼痛-攻击链接", low_desc: "痛=痛", high_desc: "痛=想攻击", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "H075", domain: 'H', name: "光照-决策速度", low_desc: "暗=决策不变", high_desc: "暗=决策延迟", min: 0.0, max: 60.0, bipolar: false },
    ParamDef { id: "H076", domain: 'H', name: "运动-情绪提升", low_desc: "运动=纯身体", high_desc: "运动=情绪药", min: 0.0, max: 200.0, bipolar: false },
    ParamDef { id: "H077", domain: 'H', name: "睡眠债务-认知衰减", low_desc: "少睡=无影响", high_desc: "少睡=认知崩溃", min: 0.0, max: 50.0, bipolar: false },
    ParamDef { id: "H078", domain: 'H', name: "噪音-压力耦联", low_desc: "噪音=背景", high_desc: "噪音=皮质醇↑", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "H079", domain: 'H', name: "气味-记忆唤起率", low_desc: "气味=气味", high_desc: "气味=时光机", min: 0.0, max: 1.0, bipolar: false },
    ParamDef { id: "H080", domain: 'H', name: "触觉-信任关联", low_desc: "被触=无关", high_desc: "被触=信任变化", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "H081", domain: 'H', name: "饥饿-风险偏好", low_desc: "饿=保守", high_desc: "饿=冒险", min: -1.0, max: 1.0, bipolar: true },
    ParamDef { id: "H082", domain: 'H', name: "姿势-权力感映射", low_desc: "姿势=姿势", high_desc: "扩张姿势=睾酮↑", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "H083", domain: 'H', name: "温度-攻击性", low_desc: "热=无影响", high_desc: "热=攻击↑", min: 0.0, max: 100.0, bipolar: false },
    ParamDef { id: "H084", domain: 'H', name: "海拔-思维抽象度", low_desc: "高度=无关", high_desc: "高度=抽象思维↑", min: -1.0, max: 1.0, bipolar: true },
];

/// 根据 ID 查找参数定义。
pub fn find_param(id: &str) -> Option<&'static ParamDef> {
    ALL_PARAMS.iter().find(|p| p.id == id)
}

/// 获取参数 ID 对应的索引（0~83）。
pub fn param_index(id: &str) -> Option<usize> {
    ALL_PARAMS.iter().position(|p| p.id == id)
}

/// 构建 ID → 索引 的查找表，用于快速访问。
pub fn build_index_map() -> std::collections::HashMap<&'static str, usize> {
    ALL_PARAMS.iter().enumerate().map(|(i, p)| (p.id, i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_params_count() {
        assert_eq!(ALL_PARAMS.len(), 84);
    }

    #[test]
    fn test_find_param() {
        let p = find_param("A001").unwrap();
        assert_eq!(p.name, "视觉采样率");
        assert_eq!(p.domain, 'A');
    }

    #[test]
    fn test_param_index() {
        assert_eq!(param_index("A001"), Some(0));
        assert_eq!(param_index("H084"), Some(83));
        assert_eq!(param_index("ZZZZ"), None);
    }

    #[test]
    fn test_describe() {
        let p = find_param("A001").unwrap();
        let desc = p.describe(1.0);
        assert!(desc.contains("极低"));
        let desc = p.describe(10.0);
        assert!(desc.contains("极高"));
    }
}
