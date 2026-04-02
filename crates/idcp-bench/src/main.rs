use idcp_system::{ExecutionMode, ScenarioProfile, evaluate};

fn main() {
    println!("IDCP cross-engine benchmark");
    println!(
        "{:<15} {:>9} {:>9} {:>9} {:>12}",
        "profile", "mem%", "flow%", "copy%", "score_x"
    );
    for profile in ScenarioProfile::all() {
        let naive = evaluate(profile, ExecutionMode::Naive);
        let idcp = evaluate(profile, ExecutionMode::Idcp);
        let improvement = idcp.improvement_over(&naive);
        println!(
            "{:<15} {:>8.1}% {:>8.1}% {:>8.1}% {:>11.2}",
            profile.slug(),
            improvement.memory_percent,
            improvement.flow_percent,
            improvement.copy_percent,
            improvement.score_multiplier
        );
    }
}
