//! 领域D: 行为执行 (D039-D042)
//! 系统如何将意图转化为行动

use crate::core::*;
use super::*;

pub fn domain_d_parameters() -> Vec<ParameterDefinition> {
    vec![
        ParameterDefinition {
            id: ParameterId::parse("D039").unwrap(),
            name: "行为蓄能时间".into(),
            domain: ParameterDomain::BehaviorExecution,
            definition: "从决定行动到实际开始行动之间的时间延迟".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("决定=行动(0s)".into(), "决定…(∞)…行动".into()),
            granularity: ParameterGranularity::Decomposable(vec!["D039a".into(), "D039b".into(), "D039c".into(), "D039d".into()]),
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("B015").unwrap(), condition: ValueCondition::High, phenomenon: "拖延不愉快任务+内疚=拖延-内疚循环".into() },
                CouplingDescription { parameter: ParameterId::parse("E040").unwrap(), condition: ValueCondition::High, phenomenon: "高蓄能+高使命感=空想家型(有理想但从不行动)".into() },
            ],
            collapses: vec![CollapseCondition { trigger: "截止日期临近".into(), direction: CollapseDirection::HighToLow, description: "截止日期效应".into() }],
            drifts: vec![],
            reversals: vec![],
            default_value: ParameterValue::unbounded(300.0),
        },
        ParameterDefinition {
            id: ParameterId::parse("D040").unwrap(),
            name: "攻击行为基线".into(),
            domain: ParameterDomain::BehaviorExecution,
            definition: "系统在无挑衅情况下发起攻击行为的概率".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("从不攻击(0)".into(), "主动攻击(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec!["D040a".into(), "D040b".into(), "D040c".into(), "D040d".into(), "D040e".into()]),
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("B015b").unwrap(), condition: ValueCondition::Low, phenomenon: "对外攻击+零内疚=冷酷型".into() },
                CouplingDescription { parameter: ParameterId::parse("B015b").unwrap(), condition: ValueCondition::High, phenomenon: "对外攻击+高内疚=迫不得已型".into() },
                CouplingDescription { parameter: ParameterId::parse("B016").unwrap(), condition: ValueCondition::DirectionBipolar(1.0), phenomenon: "攻击+享受痛苦=施虐型".into() },
                CouplingDescription { parameter: ParameterId::parse("C031").unwrap(), condition: ValueCondition::High, phenomenon: "零攻击+高支配=通过非暴力手段控制".into() },
            ],
            collapses: vec![CollapseCondition { trigger: "威胁情境".into(), direction: CollapseDirection::LowToHigh, description: "防御性攻击".into() }],
            drifts: vec![DriftPattern { description: "长期处于暴力环境中D040通常上升".into(), direction: DriftDirection::Increasing, rate_category: DriftRate::Slow }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.1),
        },
        ParameterDefinition {
            id: ParameterId::parse("D041").unwrap(),
            name: "规则遵循度".into(),
            domain: ParameterDomain::BehaviorExecution,
            definition: "系统遵守外部规则(法律/规范/命令)的默认程度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("规则=建议(0)".into(), "规则=铁律(1)".into()),
            granularity: ParameterGranularity::Decomposable(vec!["D041a".into(), "D041b".into(), "D041c".into(), "D041d".into()]),
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("C028").unwrap(), condition: ValueCondition::Low, phenomenon: "高服从命令+低自主性=盲从型".into() },
            ],
            collapses: vec![CollapseCondition { trigger: "当规则与E040(使命感)冲突".into(), direction: CollapseDirection::Variable, description: "D041可能跳变".into() }],
            drifts: vec![],
            reversals: vec![ReversalCondition { trigger: "规则制定者背叛系统".into(), from_meaning: "高遵循".into(), to_meaning: "规则信任崩塌".into() }],
            default_value: ParameterValue::normalized(0.6),
        },
        ParameterDefinition {
            id: ParameterId::parse("D042").unwrap(),
            name: "行为灵活性".into(),
            domain: ParameterDomain::BehaviorExecution,
            definition: "原计划受阻时切换到替代方案的速度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("受阻=卡死(0s)".into(), "受阻=秒换方案(∞)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("E040").unwrap(), condition: ValueCondition::High, phenomenon: "低灵活+高使命=撞了南墙也不回头".into() },
                CouplingDescription { parameter: ParameterId::parse("C037").unwrap(), condition: ValueCondition::Low, phenomenon: "高灵活+低一致=随风倒型".into() },
            ],
            collapses: vec![CollapseCondition { trigger: "压力".into(), direction: CollapseDirection::HighToLow, description: "认知僵化".into() }],
            drifts: vec![DriftPattern { description: "随年龄通常下降(习惯固化)".into(), direction: DriftDirection::Decreasing, rate_category: DriftRate::Slow }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.6),
        },
    ]
}
