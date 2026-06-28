//! PAPS CLI —— 人格原子参数系统命令行工具
//!
//! 提供命令行接口来操作和查询PAPS系统。

use clap::{Parser, Subcommand};
use personality_generator::api::PersonalitySystem;
use personality_generator::core::ParameterDomain;

/// 人格原子参数系统 (PAPS) CLI
#[derive(Parser)]
#[command(name = "paps")]
#[command(version = personality_generator::VERSION)]
#[command(about = "Personality Atomic Parameter System - 无原型·纯参数·全光谱", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示系统信息
    Info,
    /// 列出所有参数
    List {
        /// 按领域筛选 (A-H)
        #[arg(short, long)]
        domain: Option<char>,
    },
    /// 获取参数值
    Get {
        /// 参数ID (如 A001, B015)
        param_id: String,
    },
    /// 设置参数值
    Set {
        /// 参数ID
        param_id: String,
        /// 参数值
        value: f64,
    },
    /// 分析当前参数值的耦合效应
    Analyze,
    /// 推进时间（应用漂移）
    Time {
        /// 天数
        days: f64,
    },
    /// 触发相变事件
    Event {
        /// 事件类型: betrayal, loss, humiliation, power_gain, power_loss, forgiveness, mission_failure
        event_type: String,
    },
    /// 管理关系
    Relation {
        #[command(subcommand)]
        action: RelationAction,
    },
    /// 显示ε状态
    Epsilon,
    /// 导出系统状态
    Export {
        /// 输出文件路径
        #[arg(short, long, default_value = "paps_state.json")]
        output: String,
    },
    /// 导入系统状态
    Import {
        /// 输入文件路径
        input: String,
    },
}

#[derive(Subcommand)]
enum RelationAction {
    /// 添加关系
    Add {
        /// 关系ID
        id: String,
        /// 关系类型: intimate, acquaintance, stranger, hostile, superior, subordinate
        #[arg(short, long, default_value = "stranger")]
        rel_type: String,
    },
    /// 计算参数在关系中的坍缩值
    Collapse {
        /// 参数ID
        param_id: String,
        /// 关系ID
        rel_id: String,
    },
    /// 跨关系分析
    Analyze {
        /// 显示前N个差异最大的参数
        #[arg(short, long, default_value = "10")]
        top: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut system = PersonalitySystem::new();

    match cli.command {
        Commands::Info => {
            let info = system.system_info();
            println!("╔══════════════════════════════════════════════════════╗");
            println!("║  人格原子参数系统 (PAPS) v{}                    ║", info.version);
            println!("║  Personality Atomic Parameter System                ║");
            println!("╠══════════════════════════════════════════════════════╣");
            println!("║  参数总数:     {:>3}                                  ║", info.total_parameters);
            println!("║  领域数:       {:>3}                                  ║", info.total_domains);
            println!("║  耦合关系数:   {:>3}                                  ║", info.coupling_count);
            println!("║  关系数:       {:>3}                                  ║", info.relationship_count);
            println!("║  ε 值:         {:.4}                                ║", info.epsilon_value);
            println!("╚══════════════════════════════════════════════════════╝");
        }

        Commands::List { domain } => {
            if let Some(d) = domain {
                let domain_enum = match d {
                    'A' => ParameterDomain::InformationIntake,
                    'B' => ParameterDomain::EmotionGeneration,
                    'C' => ParameterDomain::MotivationValue,
                    'D' => ParameterDomain::BehaviorExecution,
                    'E' => ParameterDomain::MetacognitionSelf,
                    'F' => ParameterDomain::SocialSignal,
                    'G' => ParameterDomain::TemporalityDevelopment,
                    'H' => ParameterDomain::BodyEnvironmentCoupling,
                    _ => {
                        eprintln!("无效的领域: {} (使用 A-H)", d);
                        return;
                    }
                };
                println!("领域 {}: {}", d, domain_enum);
                println!("{:-<60}", "");
                let params = system.get_domain_parameters(domain_enum);
                for pid in &params {
                    if let Some(param) = system.get_parameter(pid) {
                        println!("  {} | {} | {}", pid, param.name, param.definition);
                    }
                }
            } else {
                println!("所有84个参数:");
                println!("{:-<60}", "");
                for pid in system.all_parameter_ids() {
                    if let Some(param) = system.get_parameter(&pid) {
                        println!("  {} | {} | {}", pid, param.name, param.definition);
                    }
                }
            }
        }

        Commands::Get { param_id } => {
            match system.get_value(&param_id) {
                Some(val) => println!("{} = {:.4}", param_id, val),
                None => eprintln!("参数 {} 不存在", param_id),
            }
        }

        Commands::Set { param_id, value } => {
            match system.set_value(&param_id, value) {
                Ok(()) => println!("{} 已设置为 {:.4}", param_id, value),
                Err(e) => eprintln!("错误: {}", e),
            }
        }

        Commands::Analyze => {
            let results = system.analyze_couplings();
            if results.is_empty() {
                println!("当前参数值未触发任何耦合效应。");
            } else {
                println!("激活的耦合效应 ({} 条):", results.len());
                println!("{:-<80}", "");
                for r in &results {
                    println!("  {}↑ + {}↑ → {}", r.param_a, r.param_b, r.phenomenon);
                }
            }
        }

        Commands::Time { days } => {
            let changes = system.advance_time(days);
            println!("时间推进 {} 天，{} 个参数发生变化:", days, changes.len());
            for (pid, drift) in &changes {
                println!("  {} 漂移: {:+.4}", pid, drift);
            }
        }

        Commands::Event { event_type } => {
            let changes = system.trigger_phase_change(&event_type);
            println!("相变事件 '{}' 触发，{} 个参数跳变:", event_type, changes.len());
            for (pid, val) in &changes {
                println!("  {} → {:.4}", pid, val);
            }
        }

        Commands::Relation { action } => match action {
            RelationAction::Add { id, rel_type } => {
                system.add_relationship(&id, &rel_type);
                println!("关系 '{}' ({}) 已添加", id, rel_type);
            }
            RelationAction::Collapse { param_id, rel_id } => {
                match system.collapse_in_relationship(&param_id, &rel_id) {
                    Some(val) => println!("{} 在关系 '{}' 中的坍缩值: {:.4}", param_id, rel_id, val),
                    None => eprintln!("无法计算坍缩值（参数或关系不存在）"),
                }
            }
            RelationAction::Analyze { top } => {
                let results = system.cross_relational_analysis(top);
                if results.is_empty() {
                    println!("无足够关系进行分析。请先添加关系。");
                } else {
                    println!("跨关系参数差异分析 (前 {} 个):", top);
                    println!("{:-<80}", "");
                    for r in &results {
                        println!("  {} (方差: {:.4})", r.param_id, r.variance);
                        for (rid, val) in &r.values {
                            println!("    {}: {:.4}", rid, val);
                        }
                    }
                }
            }
        },

        Commands::Epsilon => {
            println!("ε = {:.4}", system.epsilon_value());
            println!();
            println!("{}", system.epsilon_acknowledgment());
        }

        Commands::Export { output } => {
            match system.export_state() {
                Ok(json) => {
                    std::fs::write(&output, &json).unwrap_or_else(|e| {
                        eprintln!("写入文件失败: {}", e);
                    });
                    println!("系统状态已导出到 {}", output);
                }
                Err(e) => eprintln!("导出失败: {}", e),
            }
        }

        Commands::Import { input } => {
            match std::fs::read_to_string(&input) {
                Ok(json) => match system.import_state(&json) {
                    Ok(()) => println!("系统状态已从 {} 导入", input),
                    Err(e) => eprintln!("导入失败: {}", e),
                },
                Err(e) => eprintln!("读取文件失败: {}", e),
            }
        }
    }
}
