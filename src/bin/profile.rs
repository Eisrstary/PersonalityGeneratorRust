//! 人格档案生成器
//!
//! 用法:
//!   paps-profile                           → 人类 Markdown（默认，全随机）
//!   paps-profile --format ai               → AI Markdown
//!   paps-profile --format raw              → 原始 JSON
//!   paps-profile --tendencies FILE         → 从倾向配置生成
//!   paps-profile --batch N                 → 批量生成 N 份，输出到 output/batch/
//!   paps-profile --batch N --format ai     → 批量生成 N 份 AI 格式

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

    let batch_count: Option<usize> = args
        .iter()
        .position(|a| a == "--batch")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok());

    if let Some(n) = batch_count {
        let subdir = match format { "raw"|"json" => "json", "ai" => "ai", _ => "human" };
        let ext = match format { "raw"|"json" => "json", _ => "md" };
        let dir = format!("output/batch/{}", subdir);
        fs::create_dir_all(&dir).expect("无法创建目录");
        for i in 1..=n {
            let mut system = PersonalitySystem::new();
            generate(&mut system, tendencies_file);
            let output = match format {
                "raw"|"json" => system.export_profile_json(&default_collapse_params()),
                "ai" => system.export_profile_ai_md(&default_collapse_params()),
                _ => system.export_profile_text(&default_collapse_params()),
            };
            let path = format!("{}/profile_{:04}.{}", dir, i, ext);
            fs::write(&path, output).expect("写入失败");
        }
        eprintln!("Done: {} files → {}", n, dir);
        return;
    }

    let mut system = PersonalitySystem::new();
    generate(&mut system, tendencies_file);

    let collapse_params = default_collapse_params();
    match format {
        "raw" | "json" => println!("{}", system.export_profile_json(&collapse_params)),
        "ai" => println!("{}", system.export_profile_ai_md(&collapse_params)),
        _ => println!("{}", system.export_profile_text(&collapse_params)),
    }
}

fn default_collapse_params() -> Vec<&'static str> {
    vec!["B015", "A009", "C033", "F061", "D040", "B021", "C031", "E046"]
}

fn generate(system: &mut PersonalitySystem, tendencies_file: Option<&String>) {
    if let Some(path) = tendencies_file {
        let json = fs::read_to_string(path).expect("无法读取倾向配置文件");
        let config: serde_json::Value = serde_json::from_str(&json).expect("倾向配置 JSON 格式错误");
        if let Some(tendencies_obj) = config.get("tendencies") {
            let tendencies: HashMap<String, String> =
                serde_json::from_value(tendencies_obj.clone()).expect("tendencies 格式错误");
            system.set_tendencies(&tendencies).expect("倾向设置失败");
        } else if config.is_object() && config.get("inactive").is_none() {
            let tendencies: HashMap<String, String> =
                serde_json::from_value(config.clone()).expect("倾向配置 JSON 格式错误");
            system.set_tendencies(&tendencies).expect("倾向设置失败");
        }
        if let Some(inactive_arr) = config.get("inactive").and_then(|v| v.as_array()) {
            for item in inactive_arr {
                if let Some(id) = item.as_str() {
                    system.deactivate_parameter(id).ok();
                }
            }
        }
    } else {
        let all_ids = system.all_parameter_ids();
        let mut tendencies = HashMap::new();
        for id in &all_ids {
            tendencies.insert(id.clone(), "any".to_string());
        }
        system.set_tendencies(&tendencies).unwrap();
    }

    system.add_relationship("家人", "intimate");
    system.add_relationship("同事", "acquaintance");
    system.add_relationship("陌生人", "stranger");
    system.add_relationship("竞争对手", "hostile");
}
