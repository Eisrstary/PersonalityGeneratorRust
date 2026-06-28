//! 领域B: 情绪生成与调节 (B011-B024)
//! 系统如何生成和调控情感状态

use crate::core::*;
use super::*;

pub fn domain_b_parameters() -> Vec<ParameterDefinition> {
    vec![
        // B011 基础情绪唤醒阈值
        ParameterDefinition {
            id: ParameterId::parse("B011").unwrap(),
            name: "基础情绪唤醒阈值".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "愤怒/恐惧/悲伤/喜悦四种基本情绪各自被激活的最小刺激强度".into(),
            spectrum: SpectrumType::MultiDimensional(vec![
                SpectrumDimension {
                    name: "怒阈".into(), low_label: "极易愤怒".into(), high_label: "极难愤怒".into(),
                    spectrum_type: SpectrumType::Normalized,
                },
                SpectrumDimension {
                    name: "惧阈".into(), low_label: "极易恐惧".into(), high_label: "极难恐惧".into(),
                    spectrum_type: SpectrumType::Normalized,
                },
                SpectrumDimension {
                    name: "哀阈".into(), low_label: "极易悲伤".into(), high_label: "极难悲伤".into(),
                    spectrum_type: SpectrumType::Normalized,
                },
                SpectrumDimension {
                    name: "喜阈".into(), low_label: "极易快乐".into(), high_label: "极难快乐".into(),
                    spectrum_type: SpectrumType::Normalized,
                },
            ]),
            spectrum_labels: ("四维独立".into(), "[怒阈 惧阈 哀阈 喜阈]".into()),
            granularity: ParameterGranularity::Molecular,
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B019").unwrap(), condition: ValueCondition::High,
                    phenomenon: "极易愤怒且愤怒迅速转化为行动".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A008").unwrap(), condition: ValueCondition::High,
                    phenomenon: "极易恐惧且将模糊信号解读为威胁(焦虑型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B015").unwrap(), condition: ValueCondition::High,
                    phenomenon: "极易悲伤且悲伤伴随强烈内疚(忧郁型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B018").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "难以快乐且快乐难以维持(快感缺失型)".into(),
                },
            ],
            collapses: vec![
                CollapseCondition {
                    trigger: "疲劳".into(), direction: CollapseDirection::HighToLow,
                    description: "四阈可能同时下降(易激惹+易悲伤+易恐惧)".into(),
                },
                CollapseCondition {
                    trigger: "威胁".into(), direction: CollapseDirection::HighToLow,
                    description: "惧阈下降、喜阈上升(恐惧优先，快乐抑制)".into(),
                },
            ],
            drifts: vec![DriftPattern {
                description: "随年龄：喜阈通常上升(更难快乐)，怒阈通常上升(更不容易愤怒)".into(),
                direction: DriftDirection::Variable, rate_category: DriftRate::VerySlow,
            }],
            reversals: vec![ReversalCondition {
                trigger: "极度安全".into(), from_meaning: "低惧阈".into(), to_meaning: "无所畏惧的短暂状态".into(),
            }],
            default_value: ParameterValue::normalized(0.5),
        },
        // B012 情绪颗粒度
        ParameterDefinition {
            id: ParameterId::parse("B012").unwrap(),
            name: "情绪颗粒度".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "能够区分的离散情绪状态的数量和精度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("感觉=一团糟(0)".into(), "恼火/愤懑/愠怒/暴怒各自分明(100)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B012a".into(), "B012b".into(), "B012c".into(), "B012d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("A003").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高内感受+高颗粒度=精准的情绪自我觉察".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("E039").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高他人颗粒度+高心智理论=精准读心".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B021").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "能精准识别他人情绪但不被感染(临床观察者型)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "压力".into(), direction: CollapseDirection::HighToLow,
                description: "颗粒度急剧下降(情绪退行到「好/坏」二元)".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "可通过情绪词汇训练提升".into(),
                    direction: DriftDirection::Increasing, rate_category: DriftRate::Moderate,
                },
                DriftPattern {
                    description: "抑郁发作期间下降".into(),
                    direction: DriftDirection::Decreasing, rate_category: DriftRate::Fast,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.4),
        },
        // B013 自动思维情绪附着力
        ParameterDefinition {
            id: ParameterId::parse("B013").unwrap(),
            name: "自动思维情绪附着力".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "一个自动产生的想法所携带的情绪冲击强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("想法=纯认知(0)".into(), "一个想法=情绪炸弹(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec!["B013a".into(), "B013b".into()]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("E044").unwrap(), condition: ValueCondition::High,
                    phenomenon: "负面想法+反复咀嚼=反刍型抑郁".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B015").unwrap(), condition: ValueCondition::High,
                    phenomenon: "一个自我批评的想法触发内疚海啸".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B018").unwrap(), condition: ValueCondition::High,
                    phenomenon: "一个积极想法触发持续的积极情绪".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "压力".into(), direction: CollapseDirection::LowToHigh,
                description: "B013a急剧上升(每个负面想法都变成情绪炸弹)".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "CBT训练可降低B013a".into(),
                    direction: DriftDirection::Decreasing, rate_category: DriftRate::Moderate,
                },
                DriftPattern {
                    description: "反复创伤可升高B013a".into(),
                    direction: DriftDirection::Increasing, rate_category: DriftRate::Moderate,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        // B014 情绪调节策略库
        ParameterDefinition {
            id: ParameterId::parse("B014").unwrap(),
            name: "情绪调节策略库".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "系统可主动调用的情绪调节方法的数量和有效性".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("只有本能反应(0)".into(), "20+种主动调节方法(20)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B014a".into(), "B014b".into(), "B014c".into(), "B014d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("E043").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高策略+高正念=情绪调节大师".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B013a").unwrap(), condition: ValueCondition::High,
                    phenomenon: "无法调节+高附着力=情绪失控风险".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "压力".into(), direction: CollapseDirection::HighToLow,
                description: "策略库急剧缩小(退行到最原始的调节方式)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "可通过心理治疗/冥想训练扩展".into(),
                direction: DriftDirection::Increasing, rate_category: DriftRate::Moderate,
            }],
            reversals: vec![ReversalCondition {
                trigger: "过度使用转化策略".into(),
                from_meaning: "转化策略".into(), to_meaning: "情绪压抑(表面转化实则压抑)".into(),
            }],
            default_value: ParameterValue::unbounded(5.0),
        },
        // B015 内疚感基线
        ParameterDefinition {
            id: ParameterId::parse("B015").unwrap(),
            name: "内疚感基线".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "对自身行为造成他人痛苦后的内疚反应强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("伤害他人=完全无感(0)".into(), "伤害他人=自我折磨(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B015a".into(), "B015b".into(), "B015c".into(), "B015d".into(),
                "B015e".into(), "B015f".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B016").unwrap(), condition: ValueCondition::DirectionBipolar(1.0),
                    phenomenon: "感知痛苦+享受痛苦+不内疚=施虐型".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B016").unwrap(), condition: ValueCondition::DirectionBipolar(-1.0),
                    phenomenon: "伤害他人后极度内疚且绝不享受(迫不得已型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C030").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高内疚+高攻击=边伤害边自我折磨".into(),
                },
            ],
            collapses: vec![
                CollapseCondition {
                    trigger: "长期施害".into(), direction: CollapseDirection::HighToLow,
                    description: "内疚疲劳".into(),
                },
                CollapseCondition {
                    trigger: "被受害者原谅".into(), direction: CollapseDirection::LowToHigh,
                    description: "延迟内疚涌现".into(),
                },
            ],
            drifts: vec![
                DriftPattern {
                    description: "反复伤害同一对象：通常下降(习惯化/麻木)".into(),
                    direction: DriftDirection::Decreasing, rate_category: DriftRate::Moderate,
                },
                DriftPattern {
                    description: "停止伤害行为后：可能回升".into(),
                    direction: DriftDirection::Increasing, rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![ReversalCondition {
                trigger: "被受害者反抗/指责".into(),
                from_meaning: "高内疚".into(), to_meaning: "愤怒替代内疚".into(),
            }],
            default_value: ParameterValue::normalized(0.6),
        },
        // B016 他人痛苦-自身愉悦转化
        ParameterDefinition {
            id: ParameterId::parse("B016").unwrap(),
            name: "他人痛苦-自身愉悦转化".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "感知到他人痛苦时自身产生愉悦(正)或不适(负)的强度".into(),
            spectrum: SpectrumType::Bipolar,
            spectrum_labels: ("他人痛苦=不适(-1)".into(), "他人痛苦=愉悦(+1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B016a".into(), "B016b".into(), "B016c".into(), "B016d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B015b").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "对外群体施虐且无内疚".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C035").unwrap(), condition: ValueCondition::High,
                    phenomenon: "对仇人痛苦的愉悦+高怨恨=复仇型满足".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("E042").unwrap(), condition: ValueCondition::High,
                    phenomenon: "将施虐愉悦道德化".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B015").unwrap(), condition: ValueCondition::High,
                    phenomenon: "他人痛苦=自己痛苦+强烈内疚=共情饱和".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "被外群体严重伤害".into(), direction: CollapseDirection::Directional(1.0),
                description: "B016b可能从-1跳变到+1(反向转化)".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "长期接触外群体并建立关系：B016b通常从+1向-1漂移".into(),
                    direction: DriftDirection::Decreasing, rate_category: DriftRate::Slow,
                },
                DriftPattern {
                    description: "长期与外群体冲突：B016b通常从-1向+1漂移".into(),
                    direction: DriftDirection::Increasing, rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::bipolar(-0.5),
        },
        // B017 羞耻感基线
        ParameterDefinition {
            id: ParameterId::parse("B017").unwrap(),
            name: "羞耻感基线".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "在公开场合暴露缺陷/违规/失误后产生的羞耻反应强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("出丑=无所谓(0)".into(), "出丑=想消失(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B017a".into(), "B017b".into(), "B017c".into(), "B017d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("E046").unwrap(), condition: ValueCondition::High,
                    phenomenon: "低羞耻+高印象管理=精心设计形象但不在乎出丑".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("E045").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "高羞耻+低外显自尊=社交恐惧型".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "反复公开羞辱".into(), direction: CollapseDirection::HighToLow,
                description: "羞耻疲劳/厚脸皮化".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "社会地位上升时B017a通常上升(更在意公共形象)".into(),
                    direction: DriftDirection::Increasing, rate_category: DriftRate::Slow,
                },
                DriftPattern {
                    description: "社会地位下降时B017a可能下降(保护性麻木)".into(),
                    direction: DriftDirection::Decreasing, rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        // B018 积极情绪维持能力
        ParameterDefinition {
            id: ParameterId::parse("B018").unwrap(),
            name: "积极情绪维持能力".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "积极情绪(喜悦/满足/宁静)产生后持续的时间长度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("快乐=瞬间(0)".into(), "快乐=持续一整天(1)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B013b").unwrap(), condition: ValueCondition::High,
                    phenomenon: "一个积极想法触发并维持长时间的积极情绪".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "抑郁发作".into(), direction: CollapseDirection::HighToLow,
                description: "急剧下降".into(),
            }],
            drifts: vec![DriftPattern {
                description: "可通过感恩练习/积极心理学干预提升".into(),
                direction: DriftDirection::Increasing, rate_category: DriftRate::Moderate,
            }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        // B019 愤怒-攻击转化率
        ParameterDefinition {
            id: ParameterId::parse("B019").unwrap(),
            name: "愤怒-攻击转化率".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "愤怒情绪直接转化为攻击行为(言语或身体)的概率".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("愤怒=内心体验(0)".into(), "愤怒=立即行动(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B019a".into(), "B019b".into(), "B019c".into(), "B019d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("C030").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "愤怒→攻击链几乎没有缓冲(冲动型暴力)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C030").unwrap(), condition: ValueCondition::High,
                    phenomenon: "愤怒后延迟攻击(预谋型暴力)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A010").unwrap(), condition: ValueCondition::DirectionBipolar(-1.0),
                    phenomenon: "对强者愤怒→对弱者发泄(踢猫效应)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B015").unwrap(), condition: ValueCondition::High,
                    phenomenon: "攻击后内疚崩溃(家暴-道歉循环)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "酒精/物质影响".into(), direction: CollapseDirection::LowToHigh,
                description: "B019急剧上升".into(),
            }],
            drifts: vec![DriftPattern {
                description: "可通过愤怒管理训练降低".into(),
                direction: DriftDirection::Decreasing, rate_category: DriftRate::Moderate,
            }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.3),
        },
        // B020 情绪标签命名速度
        ParameterDefinition {
            id: ParameterId::parse("B020").unwrap(),
            name: "情绪标签命名速度".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "从情绪产生到能用语言准确描述该情绪的时间延迟".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("说不出感受(10s+)".into(), "瞬间精准命名(0.5s)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("A003").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高内感受+快速命名=情绪专家".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B013a").unwrap(), condition: ValueCondition::High,
                    phenomenon: "无法命名+高附着力=被无名情绪淹没".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "压力".into(), direction: CollapseDirection::HighToLow,
                description: "命名速度急剧下降(此处\"下降\"指延迟增加)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "可通过情绪词汇训练提升".into(),
                direction: DriftDirection::Increasing, rate_category: DriftRate::Moderate,
            }],
            reversals: vec![],
            default_value: ParameterValue::unbounded(3.0),
        },
        // B021 情绪传染易感性
        ParameterDefinition {
            id: ParameterId::parse("B021").unwrap(),
            name: "情绪传染易感性".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "他人情绪表达触发自身相同情绪的自动化程度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("他人哭泣=干眼(0)".into(), "他人哭泣=立刻泪崩(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B021a".into(), "B021b".into(), "B021c".into(), "B021d".into(), "B021e".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("C030").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "被他人愤怒传染后参与暴力(暴民型)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "威胁情境".into(), direction: CollapseDirection::HighToLow,
                description: "B021b可能从高跳变到零(对外群体关闭共情通道)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "长期接触外群体并建立关系：B021b通常上升".into(),
                direction: DriftDirection::Increasing, rate_category: DriftRate::Slow,
            }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        // B022 怨恨衰减半衰期
        ParameterDefinition {
            id: ParameterId::parse("B022").unwrap(),
            name: "怨恨衰减半衰期".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "被冒犯/伤害后怨恨情绪的强度衰减到原始强度一半所需的时间".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("冒犯=秒忘(0天)".into(), "冒犯=终身铭记(∞天)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B022a".into(), "B022b".into(), "B022c".into(), "B022d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("C035").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高怨恨+高利他惩罚=正义复仇者(自认)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C034").unwrap(), condition: ValueCondition::High,
                    phenomenon: "高怨恨+高嫉妒=「为什么他们可以而我不能」".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B019").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "内心燃烧但表面平静(冷怨恨/冷暴力)".into(),
                },
            ],
            collapses: vec![
                CollapseCondition {
                    trigger: "被冒犯者道歉/补偿".into(), direction: CollapseDirection::HighToLow,
                    description: "B022可能从∞跳变到接近零".into(),
                },
                CollapseCondition {
                    trigger: "被冒犯者再次冒犯".into(), direction: CollapseDirection::LowToHigh,
                    description: "B022可能从零跳变到∞".into(),
                },
            ],
            drifts: vec![DriftPattern {
                description: "随时间自然衰减(但半衰期因人而异)".into(),
                direction: DriftDirection::Decreasing, rate_category: DriftRate::Variable,
            }],
            reversals: vec![],
            default_value: ParameterValue::unbounded(30.0),
        },
        // B023 嫉妒触发敏感度
        ParameterDefinition {
            id: ParameterId::parse("B023").unwrap(),
            name: "嫉妒触发敏感度".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "他人优势(地位/能力/关系/物质)触发嫉妒情绪的最小差距阈值".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("他人优势=完全无感(0)".into(), "微小差距=嫉妒燃烧(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B023a".into(), "B023b".into(), "B023c".into(), "B023d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("C034").unwrap(), condition: ValueCondition::High,
                    phenomenon: "嫉妒高位者+渴望地位=向上攀爬的驱动力".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A009").unwrap(), condition: ValueCondition::High,
                    phenomenon: "嫉妒某人但仍能感知其痛苦(痛苦型嫉妒)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A009").unwrap(), condition: ValueCondition::Low,
                    phenomenon: "嫉妒某人且无视其痛苦(毁灭型嫉妒)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "被嫉妒对象遭遇不幸".into(), direction: CollapseDirection::HighToLow,
                description: "嫉妒被同情替代".into(),
            }],
            drifts: vec![DriftPattern {
                description: "自身地位上升时B023c通常下降".into(),
                direction: DriftDirection::Decreasing, rate_category: DriftRate::Slow,
            }],
            reversals: vec![ReversalCondition {
                trigger: "地位上升".into(), from_meaning: "向上嫉妒".into(),
                to_meaning: "反向嫉妒(嫉妒低位者的自由/纯真/无责任)".into(),
            }],
            default_value: ParameterValue::normalized(0.4),
        },
        // B024 幸灾乐祸阈限
        ParameterDefinition {
            id: ParameterId::parse("B024").unwrap(),
            name: "幸灾乐祸阈限".into(),
            domain: ParameterDomain::EmotionGeneration,
            definition: "他人不幸触发愉悦反应的最小不幸程度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("他人不幸=不适(0)".into(), "微小不幸=暗喜(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "B024a".into(), "B024b".into(), "B024c".into(), "B024d".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("E042").unwrap(), condition: ValueCondition::High,
                    phenomenon: "将暗喜道德化(「他遭报应了」=神圣的满足)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B023").unwrap(), condition: ValueCondition::High,
                    phenomenon: "嫉妒对象遭殃=双重满足".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "不幸超过一定阈值".into(), direction: CollapseDirection::HighToLow,
                description: "暗喜可能跳变到同情(阈值效应)".into(),
            }],
            drifts: vec![],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.3),
        },
    ]
}
