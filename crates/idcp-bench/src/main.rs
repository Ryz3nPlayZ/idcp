use idcp_system::{ExecutionMode, ScenarioProfile, evaluate_measured};

fn main() {
    println!("IDCP cross-engine benchmark");
    println!(
        "{:<15} {:>9} {:>9} {:>9} {:>9} {:>12}",
        "profile", "mem%", "flow%", "copy%", "live_ns", "score_x"
    );
    for profile in ScenarioProfile::all() {
        let naive = evaluate_measured(profile, ExecutionMode::Naive);
        let idcp = evaluate_measured(profile, ExecutionMode::Idcp);
        let improvement = idcp.improvement_over(&naive);
        println!(
            "{:<15} {:>8.1}% {:>8.1}% {:>8.1}% {:>9} {:>11.2}",
            profile.slug(),
            improvement.memory_percent,
            improvement.flow_percent,
            improvement.copy_percent,
            idcp.flow_latency_ns,
            improvement.score_multiplier
        );
    }
}
