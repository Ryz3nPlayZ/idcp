use idcp_system::{
    ControllerConfig, ExecutionMode, ScenarioProfile, evaluate, evaluate_measured, execute_runtime,
    render_report_markdown, run_controller, summarize_controller,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "plan".to_string());
    match command.as_str() {
        "plan" => {
            let profile = parse_profile(args.next().as_deref());
            print_plan(profile, false);
        }
        "measure" => {
            let profile = parse_profile(args.next().as_deref());
            print_plan(profile, true);
        }
        "simulate" => {
            let profile = parse_profile(args.next().as_deref());
            print_simulation(profile);
        }
        "run" => {
            let profile = parse_profile(args.next().as_deref());
            print_runtime(profile);
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
        "daemon" => {
            let profile = parse_profile(args.next().as_deref());
            let ticks = parse_usize(args.next().as_deref(), 5);
            let interval_ms = parse_u64(args.next().as_deref(), 250);
            let state_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_state_dir);
            if let Err(err) = run_daemon(profile, ticks, interval_ms, &state_dir) {
                exit_err(err);
            }
        }
        "status" => {
            let state_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_state_dir);
            if let Err(err) = print_status(&state_dir) {
                exit_err(err);
            }
        }
        "report" => {
            let profile = parse_profile(args.next().as_deref());
            let ticks = parse_usize(args.next().as_deref(), 5);
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(format!("idcp-report-{}.md", profile.slug())));
            if let Err(err) = write_report(profile, ticks, &output_path) {
                exit_err(err);
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!(
                "usage: idcpd [plan|measure|simulate|run <profile>|bench|profiles|daemon <profile> [ticks] [interval_ms] [state_dir]|status [state_dir]|report <profile> [ticks] [output_path]]"
            );
            std::process::exit(2);
        }
    }
}

fn parse_profile(value: Option<&str>) -> ScenarioProfile {
    value
        .and_then(ScenarioProfile::from_slug)
        .unwrap_or(ScenarioProfile::AgentMesh)
}

fn parse_usize(value: Option<&str>, default: usize) -> usize {
    value.and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn parse_u64(value: Option<&str>, default: u64) -> u64 {
    value.and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn default_state_dir() -> PathBuf {
    PathBuf::from("/tmp/idcpd")
}

fn exit_err(err: io::Error) -> ! {
    eprintln!("{err}");
    std::process::exit(1);
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

fn print_runtime(profile: ScenarioProfile) {
    let naive = execute_runtime(profile, ExecutionMode::Naive);
    let idcp = execute_runtime(profile, ExecutionMode::Idcp);
    println!("idcp runtime profile={}", profile.slug());
    println!(
        "{:<10} {:>12} {:>14} {:>14}",
        "mode", "messages", "latency_ns", "throughput"
    );
    if let Some(result) = naive {
        println!(
            "{:<10} {:>12} {:>14} {:>14}",
            "naive",
            result.processed_messages,
            result.end_to_end_latency_ns,
            result.throughput_msgs_per_sec
        );
    }
    if let Some(result) = idcp {
        println!(
            "{:<10} {:>12} {:>14} {:>14}",
            "idcp",
            result.processed_messages,
            result.end_to_end_latency_ns,
            result.throughput_msgs_per_sec
        );
    }
}

fn run_daemon(
    profile: ScenarioProfile,
    ticks: usize,
    interval_ms: u64,
    state_dir: &Path,
) -> io::Result<()> {
    fs::create_dir_all(state_dir)?;
    let config = ControllerConfig {
        profile,
        ticks,
        interval_ms,
    };
    let snapshots = run_controller(config);
    let summary = summarize_controller(&snapshots);
    let report = render_report_markdown(config, &snapshots);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let latest = snapshots
        .last()
        .ok_or_else(|| io::Error::other("controller produced no snapshots"))?;
    let latest_text = format!(
        "timestamp={now}\nprofile={}\ntick={}\naction={:?}\nmem_percent={:.1}\nflow_percent={:.1}\ncopy_percent={:.1}\nscore_multiplier={:.2}\nruntime_latency_ns={}\nruntime_throughput={}\n",
        profile.slug(),
        latest.tick,
        latest.action,
        latest.improvement.memory_percent,
        latest.improvement.flow_percent,
        latest.improvement.copy_percent,
        latest.improvement.score_multiplier,
        latest.runtime_idcp.end_to_end_latency_ns,
        latest.runtime_idcp.throughput_msgs_per_sec,
    );
    fs::write(state_dir.join("latest.txt"), latest_text)?;
    fs::write(state_dir.join("report.md"), report)?;

    let summary_text = format!(
        "profile={} ticks={} avg_mem={:.1}% avg_flow={:.1}% avg_copy={:.1}% avg_score={:.2}x avg_runtime_latency={:.1}% avg_runtime_throughput={:.1}%\n",
        profile.slug(),
        snapshots.len(),
        summary.avg_memory_percent,
        summary.avg_flow_percent,
        summary.avg_copy_percent,
        summary.avg_score_multiplier,
        summary.avg_runtime_latency_percent,
        summary.avg_runtime_throughput_percent,
    );
    let history_path = state_dir.join("history.log");
    let mut history = fs::read_to_string(&history_path).unwrap_or_default();
    history.push_str(&summary_text);
    fs::write(history_path, history)?;

    println!(
        "idcpd daemon complete profile={} ticks={} state_dir={}",
        profile.slug(),
        snapshots.len(),
        state_dir.display()
    );
    println!(
        "summary avg_mem={:.1}% avg_flow={:.1}% avg_copy={:.1}% avg_score={:.2}x avg_runtime_latency={:.1}% avg_runtime_throughput={:.1}%",
        summary.avg_memory_percent,
        summary.avg_flow_percent,
        summary.avg_copy_percent,
        summary.avg_score_multiplier,
        summary.avg_runtime_latency_percent,
        summary.avg_runtime_throughput_percent
    );
    Ok(())
}

fn print_status(state_dir: &Path) -> io::Result<()> {
    let latest = fs::read_to_string(state_dir.join("latest.txt"))?;
    println!("{latest}");
    Ok(())
}

fn write_report(profile: ScenarioProfile, ticks: usize, output_path: &Path) -> io::Result<()> {
    let config = ControllerConfig {
        profile,
        ticks,
        interval_ms: 0,
    };
    let snapshots = run_controller(config);
    let report = render_report_markdown(config, &snapshots);
    fs::write(output_path, report)?;
    println!(
        "wrote report profile={} ticks={} path={}",
        profile.slug(),
        snapshots.len(),
        output_path.display()
    );
    Ok(())
}
