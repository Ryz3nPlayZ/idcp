use idcp_system::{ExecutionMode, ScenarioProfile, evaluate, evaluate_measured};

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "plan".to_string());
    match command.as_str() {
        "plan" => {
            let profile = args
                .next()
                .as_deref()
                .and_then(ScenarioProfile::from_slug)
                .unwrap_or(ScenarioProfile::AgentMesh);
            print_plan(profile, false);
        }
        "measure" => {
            let profile = args
                .next()
                .as_deref()
                .and_then(ScenarioProfile::from_slug)
                .unwrap_or(ScenarioProfile::AgentMesh);
            print_plan(profile, true);
        }
        "simulate" => {
            let profile = args
                .next()
                .as_deref()
                .and_then(ScenarioProfile::from_slug)
                .unwrap_or(ScenarioProfile::AgentMesh);
            print_simulation(profile);
        }
        "bench" => {
            for profile in ScenarioProfile::all() {
                print_summary(profile);
            }
        }
        "profiles" => {
            for profile in ScenarioProfile::all() {
                println!("{}", profile.slug());
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!(
                "usage: idcpd [plan <profile>|measure <profile>|simulate <profile>|bench|profiles]"
            );
            std::process::exit(2);
        }
    }
}

fn print_plan(profile: ScenarioProfile, measured: bool) {
    let eval = if measured {
        evaluate_measured(profile, ExecutionMode::Idcp)
    } else {
        evaluate(profile, ExecutionMode::Idcp)
    };
    println!(
        "idcpd {} profile={}",
        if measured { "measure" } else { "plan" },
        profile.slug()
    );
    println!(
        "flow transport={} batching={} zero_copy={}",
        eval.flow.transport.label(),
        eval.flow.batching,
        eval.flow.zero_copy_preferred
    );
    println!(
        "placement zone={:?} affinity_score={} copy_penalty_ns={}",
        eval.placement.zone,
        eval.placement.affinity_score,
        eval.placement.estimated_copy_penalty_ns
    );
    println!(
        "pressure level={:?} compress_cold={} enable_page_families={} rebalance_work={} estimated_relief_mib={:.2}",
        eval.pressure.level,
        eval.pressure.compress_cold,
        eval.pressure.enable_page_families,
        eval.pressure.rebalance_work,
        eval.pressure.estimated_relief_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        "memory raw_mib={:.2} smart_mib={:.2} savings={:.1}%",
        eval.memory.raw_bytes as f64 / 1024.0 / 1024.0,
        eval.memory.estimated_bytes as f64 / 1024.0 / 1024.0,
        eval.memory.savings_percent()
    );
    println!("flow_latency_ns={}", eval.flow_latency_ns);
}

fn print_summary(profile: ScenarioProfile) {
    let naive = evaluate_measured(profile, ExecutionMode::Naive);
    let idcp = evaluate_measured(profile, ExecutionMode::Idcp);
    let improvement = idcp.improvement_over(&naive);
    println!(
        "{} mem={:.1}% flow={:.1}% copy={:.1}% naive_ns={} idcp_ns={} score={:.2}x",
        profile.slug(),
        improvement.memory_percent,
        improvement.flow_percent,
        improvement.copy_percent,
        naive.flow_latency_ns,
        idcp.flow_latency_ns,
        improvement.score_multiplier
    );
}

fn print_simulation(profile: ScenarioProfile) {
    let naive = evaluate_measured(profile, ExecutionMode::Naive);
    let idcp = evaluate_measured(profile, ExecutionMode::Idcp);
    let improvement = idcp.improvement_over(&naive);
    println!("idcp simulation profile={}", profile.slug());
    println!(
        "{:<10} {:>10} {:>10} {:>12} {:>10}",
        "mode", "mem_mib", "flow_ns", "copy_penalty", "score"
    );
    println!(
        "{:<10} {:>10.2} {:>10} {:>12} {:>10}",
        "naive",
        naive.memory_bytes as f64 / 1024.0 / 1024.0,
        naive.flow_latency_ns,
        naive.copy_penalty_ns,
        naive.total_score
    );
    println!(
        "{:<10} {:>10.2} {:>10} {:>12} {:>10}",
        "idcp",
        idcp.memory_bytes as f64 / 1024.0 / 1024.0,
        idcp.flow_latency_ns,
        idcp.copy_penalty_ns,
        idcp.total_score
    );
    println!(
        "delta mem={:.1}% flow={:.1}% copy={:.1}% score={:.2}x",
        improvement.memory_percent,
        improvement.flow_percent,
        improvement.copy_percent,
        improvement.score_multiplier
    );
}
