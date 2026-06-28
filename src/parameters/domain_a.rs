//! 领域A: 信息摄入 (A001-A010)
//! 世界如何进入这个系统

use crate::core::*;
use crate::parameters::*;

// ============================================================================
// 领域A: 信息摄入 (A001-A010)
// ============================================================================

pub fn domain_a_parameters() -> Vec<ParameterDefinition> {
    vec![
        // A001 视觉采样率
        ParameterDefinition {
            id: ParameterId::parse("A001").unwrap(),
            name: "视觉采样率".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "单位时间内视觉注意点的切换频率".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("凝视锁定(1Hz)".into(), "高速扫描(10Hz)".into()),
            granularity: ParameterGranularity::Decomposable(vec!["A001a".into(), "A001b".into()]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("A009").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "看到一切但看不到人".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "威胁情境".into(),
                direction: CollapseDirection::Directional(0.0),
                description: "隧道视觉(锁定)或过度警觉(暴涨)——方向取决于A008".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "随年龄缓慢下降".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::VerySlow,
                },
                DriftPattern {
                    description: "创伤后可能出现永久偏移".into(),
                    direction: DriftDirection::Variable,
                    rate_category: DriftRate::Fast,
                },
            ],
            reversals: vec![ReversalCondition {
                trigger: "极度疲劳".into(),
                from_meaning: "高采样率".into(),
                to_meaning: "认知崩溃(零采样)".into(),
            }],
            default_value: ParameterValue::unbounded(4.0),
        },
        // A002 听觉歧义容忍窗口
        ParameterDefinition {
            id: ParameterId::parse("A002").unwrap(),
            name: "听觉歧义容忍窗口".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "对模糊语音/语调保持多解而不急于消歧的持续时间".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("立即消歧(0s)".into(), "无限悬置(10s+)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "A002a".into(), "A002b".into(), "A002c".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("A035").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "在模糊中寻找隐藏敌意".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A035").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "听不懂讽刺但也不觉得被冒犯".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "被信任者背叛".into(),
                direction: CollapseDirection::HighToLow,
                description: "A002b/c可能从高永久跳变到零".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "随年龄通常上升(经验积累)".into(),
                    direction: DriftDirection::Increasing,
                    rate_category: DriftRate::Slow,
                },
                DriftPattern {
                    description: "反复背叛后永久下降".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::Fast,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::unbounded(3.0),
        },
        // A003 内感受分辨率
        ParameterDefinition {
            id: ParameterId::parse("A003").unwrap(),
            name: "内感受分辨率".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "对自身躯体信号(心跳、呼吸、胃紧、肌肉张力)的觉察精度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("完全不觉察(0)".into(), "每个心跳都清晰感知(1)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B020").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "身体知道但无法命名(躯体化风险)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B020").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "高情绪颗粒度".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B015").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "躯体化(情绪通过身体表达但意识不到)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "创伤".into(),
                direction: CollapseDirection::HighToLow,
                description: "躯体解离".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "可通过正念训练提升".into(),
                    direction: DriftDirection::Increasing,
                    rate_category: DriftRate::Moderate,
                },
                DriftPattern {
                    description: "慢性压力下缓慢下降".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        // A004 社会性线索优先级
        ParameterDefinition {
            id: ParameterId::parse("A004").unwrap(),
            name: "社会性线索优先级".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "面孔/注视方向/身体朝向相对于非社会性物体的注意优先级".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("面孔=物体(0)".into(), "面孔自动捕获注意(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "A004a".into(), "A004b".into(), "A004c".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B021").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "高度关注人但不被感染(观察者型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A008").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "不关注面孔但高度警觉身体姿势(威胁检测替代通路)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "被群体驱逐".into(),
                direction: CollapseDirection::HighToLow,
                description: "A004a可能崩塌(内群体面孔变为威胁信号)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "孤独长期化后缓慢下降".into(),
                direction: DriftDirection::Decreasing,
                rate_category: DriftRate::Slow,
            }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.7),
        },
        // A005 新异刺激打断阈值
        ParameterDefinition {
            id: ParameterId::parse("A005").unwrap(),
            name: "新异刺激打断阈值".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "意外刺激使当前注意焦点发生偏移的最小强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("雷打不动(高阈值)".into(), "落叶惊心(低阈值)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "A005a".into(), "A005b".into(), "A005c".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("C030").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "极易分心且无法抑制冲动(ADHD型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A008").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "高度集中但过度警觉(狙击手型)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "安全→威胁切换".into(),
                direction: CollapseDirection::HighToLow,
                description: "阈值可能从高跳变到极低".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "随年龄通常上升(更不容易被打断)".into(),
                    direction: DriftDirection::Increasing,
                    rate_category: DriftRate::VerySlow,
                },
                DriftPattern {
                    description: "睡眠剥夺后急剧下降".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::VeryFast,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        // A006 背景-前景分离效率
        ParameterDefinition {
            id: ParameterId::parse("A006").unwrap(),
            name: "背景-前景分离效率".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "在多声源/多刺激环境中提取目标信息的速度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("淹没在噪音中(慢)".into(), "鸡尾酒效应大师(快)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("A004").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "在人群中精准锁定一个人的声音".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("A002").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "在嘈杂环境中完全无法交流(社交退缩的风险因子)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "疲劳".into(),
                direction: CollapseDirection::HighToLow,
                description: "效率急剧下降".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "随年龄缓慢下降".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::VerySlow,
                },
                DriftPattern {
                    description: "音乐训练可提升".into(),
                    direction: DriftDirection::Increasing,
                    rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.6),
        },
        // A007 预期违背消耗
        ParameterDefinition {
            id: ParameterId::parse("A007").unwrap(),
            name: "预期违背消耗".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "处理不符合预期的信息时消耗的认知资源比例".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("意外=无所谓(0%)".into(), "意外=认知地震(100%)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "A007a".into(), "A007b".into(), "A007c".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("E040").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "自我概念受到挑战时认知资源急剧消耗(防御反应)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C026").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "意外触发强烈的意义寻求".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "威胁情境".into(),
                direction: CollapseDirection::HighToLow,
                description: "A007b可能从高跳变到零(不再预期社交常规，进入战斗模式)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "反复经历同类违背后缓慢下降(习惯化)".into(),
                direction: DriftDirection::Decreasing,
                rate_category: DriftRate::Moderate,
            }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.4),
        },
        // A008 威胁线索放大系数
        ParameterDefinition {
            id: ParameterId::parse("A008").unwrap(),
            name: "威胁线索放大系数".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "将模糊/中性刺激解读为威胁信号的倾向强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("威胁=客观威胁(0)".into(), "中性表情=敌意信号(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "A008a".into(), "A008b".into(), "A008c".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("A035").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "偏执型信息处理(一切都是针对我的)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C030").unwrap(),
                    condition: ValueCondition::Low,
                    phenomenon: "高度警觉+冲动反应(先发制人型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B019").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "感知到威胁后迅速愤怒(敌意归因→愤怒→攻击链)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "安全→威胁切换".into(),
                direction: CollapseDirection::LowToHigh,
                description: "A008可能从低跳变到极高".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "长期暴露于真实威胁环境中：永久升高(适应性警觉)".into(),
                    direction: DriftDirection::Increasing,
                    rate_category: DriftRate::Moderate,
                },
                DriftPattern {
                    description: "长期安全环境中：缓慢下降".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![ReversalCondition {
                trigger: "极度恐惧".into(),
                from_meaning: "威胁放大".into(),
                to_meaning: "完全麻木(冻结反应)".into(),
            }],
            default_value: ParameterValue::normalized(0.3),
        },
        // A009 痛苦线索敏感度
        ParameterDefinition {
            id: ParameterId::parse("A009").unwrap(),
            name: "痛苦线索敏感度".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "对他人痛苦表情/声音/姿态的注意捕获强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("他人痛苦=背景噪音(0)".into(), "他人皱眉=自己心痛(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec![
                "A009a".into(), "A009b".into(), "A009c".into(),
            ]),
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("B015").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "伤害他人后自我折磨(迫不得已型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B016").unwrap(),
                    condition: ValueCondition::DirectionBipolar(1.0),
                    phenomenon: "感知痛苦+享受痛苦(施虐型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B016").unwrap(),
                    condition: ValueCondition::DirectionBipolar(-1.0),
                    phenomenon: "共情饱和型".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "长期施害".into(),
                direction: CollapseDirection::HighToLow,
                description: "A009c可能从高跳变到零(内疚疲劳→共情麻木)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "反复暴露于他人痛苦而不采取行动：缓慢下降(共情疲劳)".into(),
                direction: DriftDirection::Decreasing,
                rate_category: DriftRate::Slow,
            }],
            reversals: vec![ReversalCondition {
                trigger: "被受害者反抗".into(),
                from_meaning: "高敏感".into(),
                to_meaning: "愤怒替代共情".into(),
            }],
            default_value: ParameterValue::normalized(0.6),
        },
        // A010 猎物/捕食者注意偏向
        ParameterDefinition {
            id: ParameterId::parse("A010").unwrap(),
            name: "猎物/捕食者注意偏向".into(),
            domain: ParameterDomain::InformationIntake,
            definition: "注意资源自动流向弱者(猎物)还是强者(捕食者)的倾向".into(),
            spectrum: SpectrumType::Bipolar,
            spectrum_labels: ("注意自动流向弱者(-1)".into(), "注意自动流向强者(+1)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription {
                    parameter: ParameterId::parse("C031").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "寻找可保护对象(保护者型)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("B019").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "在强者面前自卑，在弱者面前发泄(踢猫效应链)".into(),
                },
                CouplingDescription {
                    parameter: ParameterId::parse("C034").unwrap(),
                    condition: ValueCondition::High,
                    phenomenon: "崇拜强者+渴望成为强者(向上认同)".into(),
                },
            ],
            collapses: vec![CollapseCondition {
                trigger: "获得权力".into(),
                direction: CollapseDirection::Directional(-1.0),
                description: "从仰视强者变为俯视弱者".into(),
            }],
            drifts: vec![
                DriftPattern {
                    description: "社会地位上升时缓慢偏向-1(更多注意弱者)".into(),
                    direction: DriftDirection::Decreasing,
                    rate_category: DriftRate::Slow,
                },
                DriftPattern {
                    description: "社会地位下降时缓慢偏向+1(更多注意强者)".into(),
                    direction: DriftDirection::Increasing,
                    rate_category: DriftRate::Slow,
                },
            ],
            reversals: vec![],
            default_value: ParameterValue::bipolar(0.0),
        },
    ]
}
