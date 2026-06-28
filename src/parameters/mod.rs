//! 原子参数库 —— 84个参数的总装模块
//!
//! 将所有8个领域的参数组装成统一的参数注册表

mod domain_a;
mod domain_b;
mod domain_c;
mod domain_d;
mod domain_e;
mod domain_f;
mod domain_g;
mod domain_h;

use crate::core::*;
use std::collections::HashMap;

// Re-export the ParameterDefinition type
pub use crate::core::ParameterGranularity;

/// 参数注册表 —— 所有84个参数的集中存储和查询
#[derive(Debug, Clone)]
pub struct ParameterRegistry {
    /// 按ID索引的参数定义
    definitions: HashMap<ParameterId, ParameterDefinition>,
    /// 按领域分组的参数ID列表
    by_domain: HashMap<ParameterDomain, Vec<ParameterId>>,
    /// 参数ID的完整列表（按编号排序）
    ordered_ids: Vec<ParameterId>,
}

impl ParameterRegistry {
    /// 构建完整的参数注册表（包含所有84个参数）
    pub fn new() -> Self {
        let mut definitions = HashMap::with_capacity(84);
        let mut by_domain: HashMap<ParameterDomain, Vec<ParameterId>> = HashMap::new();

        // 收集所有领域的参数
        let all_params: Vec<ParameterDefinition> = vec![
            domain_a::domain_a_parameters(),
            domain_b::domain_b_parameters(),
            domain_c::domain_c_parameters(),
            domain_d::domain_d_parameters(),
            domain_e::domain_e_parameters(),
            domain_f::domain_f_parameters(),
            domain_g::domain_g_parameters(),
            domain_h::domain_h_parameters(),
        ]
        .into_iter()
        .flatten()
        .collect();

        let mut ordered_ids: Vec<ParameterId> = all_params.iter().map(|p| p.id.clone()).collect();
        ordered_ids.sort_by(|a, b| {
            a.domain.cmp(&b.domain).then(a.number.cmp(&b.number))
        });

        for param in all_params {
            let domain = param.domain;
            let id = param.id.clone();
            by_domain.entry(domain).or_default().push(id.clone());
            definitions.insert(id, param);
        }

        ParameterRegistry {
            definitions,
            by_domain,
            ordered_ids,
        }
    }

    /// 获取参数总数
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// 根据ID获取参数定义
    pub fn get(&self, id: &ParameterId) -> Option<&ParameterDefinition> {
        self.definitions.get(id)
    }

    /// 根据字符串ID获取参数定义
    pub fn get_by_str(&self, s: &str) -> Option<&ParameterDefinition> {
        let id = ParameterId::parse(s)?;
        self.get(&id)
    }

    /// 获取某个领域的所有参数ID
    pub fn get_domain_ids(&self, domain: &ParameterDomain) -> &[ParameterId] {
        self.by_domain.get(domain).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 获取某个领域的所有参数定义
    pub fn get_domain_params(&self, domain: &ParameterDomain) -> Vec<&ParameterDefinition> {
        self.get_domain_ids(domain)
            .iter()
            .filter_map(|id| self.get(id))
            .collect()
    }

    /// 获取所有参数ID（按编号排序）
    pub fn all_ids(&self) -> &[ParameterId] {
        &self.ordered_ids
    }

    /// 遍历所有参数
    pub fn iter(&self) -> impl Iterator<Item = &ParameterDefinition> {
        self.definitions.values()
    }

    /// 获取参数的耦合关系
    pub fn get_couplings(&self, id: &ParameterId) -> Vec<&CouplingDescription> {
        self.get(id).map(|p| p.couplings.iter().collect()).unwrap_or_default()
    }

    /// 获取参数的崩塌条件
    pub fn get_collapses(&self, id: &ParameterId) -> Vec<&CollapseCondition> {
        self.get(id).map(|p| p.collapses.iter().collect()).unwrap_or_default()
    }

    /// 获取参数的漂移模式
    pub fn get_drifts(&self, id: &ParameterId) -> Vec<&DriftPattern> {
        self.get(id).map(|p| p.drifts.iter().collect()).unwrap_or_default()
    }

    /// 获取参数的反转条件
    pub fn get_reversals(&self, id: &ParameterId) -> Vec<&ReversalCondition> {
        self.get(id).map(|p| p.reversals.iter().collect()).unwrap_or_default()
    }

    /// 检查参数是否存在
    pub fn contains(&self, id: &ParameterId) -> bool {
        self.definitions.contains_key(id)
    }

    /// 获取所有领域的统计信息
    pub fn domain_stats(&self) -> Vec<(ParameterDomain, usize)> {
        let mut stats: Vec<_> = self
            .by_domain
            .iter()
            .map(|(d, ids)| (*d, ids.len()))
            .collect();
        stats.sort_by_key(|(d, _)| *d as u8);
        stats
    }
}

impl Default for ParameterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_total_count() {
        let registry = ParameterRegistry::new();
        assert_eq!(registry.len(), 84, "Should have exactly 84 parameters");
    }

    #[test]
    fn test_registry_all_domains_present() {
        let registry = ParameterRegistry::new();
        let domains = [
            ParameterDomain::InformationIntake,
            ParameterDomain::EmotionGeneration,
            ParameterDomain::MotivationValue,
            ParameterDomain::BehaviorExecution,
            ParameterDomain::MetacognitionSelf,
            ParameterDomain::SocialSignal,
            ParameterDomain::TemporalityDevelopment,
            ParameterDomain::BodyEnvironmentCoupling,
        ];
        for domain in &domains {
            let ids = registry.get_domain_ids(domain);
            assert!(!ids.is_empty(), "Domain {:?} should have parameters", domain);
        }
    }

    #[test]
    fn test_parameter_lookup() {
        let registry = ParameterRegistry::new();
        assert!(registry.get_by_str("A001").is_some());
        assert!(registry.get_by_str("B015f").is_some());
        assert!(registry.get_by_str("H084").is_some());
        assert!(registry.get_by_str("Z999").is_none());
    }

    #[test]
    fn test_domain_a_count() {
        let registry = ParameterRegistry::new();
        let ids = registry.get_domain_ids(&ParameterDomain::InformationIntake);
        assert_eq!(ids.len(), 10, "Domain A should have 10 parameters");
    }

    #[test]
    fn test_domain_b_count() {
        let registry = ParameterRegistry::new();
        let ids = registry.get_domain_ids(&ParameterDomain::EmotionGeneration);
        assert_eq!(ids.len(), 14, "Domain B should have 14 parameters");
    }
}
