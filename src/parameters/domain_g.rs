//! 领域G: 时间性与发展 (G063-G066)
//! 参数如何随时间变化 —— 元参数

use crate::core::*;
use super::*;

pub fn domain_g_parameters() -> Vec<ParameterDefinition> {
    vec![
        ParameterDefinition {
            id: ParameterId::parse("G063").unwrap(),
            name: "参数漂移速率".into(),
            domain: ParameterDomain::TemporalityDevelopment,
            definition: "各参数随时间定向变化的速度(每个参数独立)".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("参数永恒不变(0)".into(), "参数瞬息万变(1)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![CouplingDescription {
                parameter: ParameterId::parse("G064").unwrap(), condition: ValueCondition::Low,
                phenomenon: "高漂移+低相变阈值=人格高度可塑".into(),
            }],
            collapses: vec![CollapseCondition {
                trigger: "重大事件(创伤/皈依/成功/丧失)".into(),
                direction: CollapseDirection::Variable,
                description: "可导致参数跳变(非连续变化)".into(),
            }],
            drifts: vec![DriftPattern {
                description: "随年龄G063通常下降(人格越来越稳定)".into(),
                direction: DriftDirection::Decreasing, rate_category: DriftRate::VerySlow,
            }],
            reversals: vec![ReversalCondition {
                trigger: "漂移方向反转".into(),
                from_meaning: "正向漂移".into(),
                to_meaning: "反向漂移(如B015内疚感在长期施害后从上升反转为下降)".into(),
            }],
            default_value: ParameterValue::normalized(0.3),
        },
        ParameterDefinition {
            id: ParameterId::parse("G064").unwrap(),
            name: "重大事件相变阈值".into(),
            domain: ParameterDomain::TemporalityDevelopment,
            definition: "引起参数永久偏移所需的最小事件冲击强度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("什么事都改不了我(100)".into(), "小事也能改变我(0)".into()),
            granularity: ParameterGranularity::Decomposable(vec!["G064a".into(), "G064b".into()]),
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("B022").unwrap(), condition: ValueCondition::High, phenomenon: "低相变阈值+高怨恨=一次背叛改变终身".into() },
                CouplingDescription { parameter: ParameterId::parse("E055").unwrap(), condition: ValueCondition::High, phenomenon: "高相变阈值+高自我欺骗=拒绝承认自己已经改变".into() },
            ],
            collapses: vec![CollapseCondition { trigger: "相变本身就是崩塌".into(), direction: CollapseDirection::Variable, description: "参数永久偏移".into() }],
            drifts: vec![DriftPattern { description: "随年龄G064通常上升(人格越来越稳定)".into(), direction: DriftDirection::Increasing, rate_category: DriftRate::VerySlow }],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.5),
        },
        ParameterDefinition {
            id: ParameterId::parse("G065").unwrap(),
            name: "情境人格切换幅度".into(),
            domain: ParameterDomain::TemporalityDevelopment,
            definition: "同一参数在不同情境(家庭/职场/独处/社交)间的取值差异幅度".into(),
            spectrum: SpectrumType::Normalized,
            spectrum_labels: ("在家=在职场(0%)".into(), "在家≠在职场/判若两人(100%)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("E055").unwrap(), condition: ValueCondition::High, phenomenon: "高切换+高自我欺骗=真诚地相信每个情境的自己都是「真正的自己」".into() },
            ],
            collapses: vec![CollapseCondition { trigger: "当两个情境发生碰撞(如家人出现在职场)".into(), direction: CollapseDirection::Variable, description: "切换可能崩溃".into() }],
            drifts: vec![],
            reversals: vec![],
            default_value: ParameterValue::normalized(0.3),
        },
        ParameterDefinition {
            id: ParameterId::parse("G066").unwrap(),
            name: "身份叙事更新速率".into(),
            domain: ParameterDomain::TemporalityDevelopment,
            definition: "系统的自我定义在新经历后更新的速度".into(),
            spectrum: SpectrumType::Unbounded,
            spectrum_labels: ("自我定义=固定(∞天)".into(), "自我定义=持续重写(0天)".into()),
            granularity: ParameterGranularity::Atomic,
            couplings: vec![
                CouplingDescription { parameter: ParameterId::parse("G064").unwrap(), condition: ValueCondition::High, phenomenon: "固定自我+高相变阈值=「我一直都是这样的人」".into() },
                CouplingDescription { parameter: ParameterId::parse("C037").unwrap(), condition: ValueCondition::Low, phenomenon: "快速更新+低一致=「今天的我和昨天完全不同」".into() },
            ],
            collapses: vec![],
            drifts: vec![DriftPattern { description: "随年龄G066通常下降(自我叙事越来越固定)".into(), direction: DriftDirection::Decreasing, rate_category: DriftRate::VerySlow }],
            reversals: vec![],
            default_value: ParameterValue::unbounded(365.0),
        },
    ]
}
