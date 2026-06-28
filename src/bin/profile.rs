//! 人格档案生成器 —— 统一输出三种格式
//!
//! 用法:
//!   paps-profile                      → 人类可读文本（默认，全随机）
//!   paps-profile --format raw         → 完整原始 JSON
//!   paps-profile --format ai          → AI 角色扮演优化 JSON
//!   paps-profile --tendencies FILE    → 从 JSON 文件读取倾向配置
//!
//! 倾向 JSON 格式:
//!   { "A001": "high", "B015": "very_high", "C031": "negative", ... }
//!   未指定的参数随机生成。

use personality_generator::api::PersonalitySystem;
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let format = args
        .iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("text");

    let tendencies_file = args
        .iter()
        .position(|a| a == "--tendencies")
        .and_then(|i| args.get(i + 1));

    let mut system = PersonalitySystem::new();

    // 加载倾向配置，或全部随机
    if let Some(path) = tendencies_file {
        let json = fs::read_to_string(path).expect("无法读取倾向配置文件");
        let tendencies: HashMap<String, String> =
            serde_json::from_str(&json).expect("倾向配置 JSON 格式错误");
        system.set_tendencies(&tendencies).expect("倾向设置失败");
    } else {
        // 全部 random（使用 medium 倾向）
        let all_ids = system.all_parameter_ids();
        let mut tendencies = HashMap::new();
        for id in &all_ids {
            tendencies.insert(id.clone(), "medium".to_string());
        }
        system.set_tendencies(&tendencies).unwrap();
    }

    system.add_relationship("家人", "intimate");
    system.add_relationship("同事", "acquaintance");
    system.add_relationship("陌生人", "stranger");
    system.add_relationship("竞争对手", "hostile");

    let collapse_params = &["B015", "A009", "C033", "F061", "D040", "B021", "C031", "E046"];

    match format {
        "raw" | "json" => println!("{}", system.export_profile_json(collapse_params)),
        "ai" => println!("{}", system.export_profile_ai_json(collapse_params)),
        _ => println!("{}", system.export_profile_text(collapse_params)),
    }
}
