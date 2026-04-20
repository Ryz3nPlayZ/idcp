use idcp_system::{ExecutionMode, ScenarioProfile, evaluate, evaluate_measured, execute_runtime};

struct BenchmarkScenario {
    profile: ScenarioProfile,
    purpose: &'static str,
}

#[derive(Clone)]
struct Stats {
    mean: f64,
    median: f64,
    min: f64,
    max: f64,
    stddev: f64,
    samples: usize,
}

impl Stats {
    fn from(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let samples = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / samples as f64;
        let median = if samples % 2 == 0 {
            (sorted[samples / 2 - 1] + sorted[samples / 2]) / 2.0
        } else {
            sorted[samples / 2]
        };
        let variance = sorted
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / samples as f64;

        Some(Self {
            mean,
            median,
            min: sorted[0],
            max: sorted[samples - 1],
            stddev: variance.sqrt(),
            samples,
        })
    }
}

struct RuntimeStats {
    latency_ns: Stats,
    throughput: Stats,
}

fn main() {
    let trials = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(10)
        .max(3);

    let scenarios = [
        BenchmarkScenario {
            profile: ScenarioProfile::AgentMesh,
            purpose: "flow-sensitive mesh (transport + placement)",
        },
        BenchmarkScenario {
            profile: ScenarioProfile::PluginHost,
            purpose: "memory-sharing host (memory + pressure)",
        },
        BenchmarkScenario {
            profile: ScenarioProfile::EmbeddingFarm,
            purpose: "batch-heavy pipeline (flow batching + pressure)",
        },
        BenchmarkScenario {
            profile: ScenarioProfile::TerminalGraph,
            purpose: "interactive graph (low-latency + locality)",
        },
    ];

    println!("# IDCP conventional-vs-idcp benchmark");
    println!("trials_per_scenario={trials}");
    println!(
        "| scenario | purpose | mem% | flow% | copy% | runtime_lat% | runtime_tput% | score_x |"
    );
    println!("| :-- | :-- | --: | --: | --: | --: | --: | --: |");

    for scenario in scenarios {
        let naive_model = evaluate(scenario.profile, ExecutionMode::Naive);
        let idcp_model = evaluate(scenario.profile, ExecutionMode::Idcp);

        let mem_improvement = percent_better(
            naive_model.memory_bytes as f64,
            idcp_model.memory_bytes as f64,
        );
        let copy_improvement = percent_better(
            naive_model.copy_penalty_ns as f64,
            idcp_model.copy_penalty_ns as f64,
        );
        let score_multiplier =
            idcp_model.total_score as f64 / naive_model.total_score.max(1) as f64;

        let naive_flow = collect_flow_stats(scenario.profile, ExecutionMode::Naive, trials);
        let idcp_flow = collect_flow_stats(scenario.profile, ExecutionMode::Idcp, trials);
        let flow_improvement = match (&naive_flow, &idcp_flow) {
            (Some(naive), Some(idcp)) => percent_better(naive.mean.max(1.0), idcp.mean.max(1.0)),
            _ => 0.0,
        };

        let naive_runtime = collect_runtime_stats(scenario.profile, ExecutionMode::Naive, trials);
        let idcp_runtime = collect_runtime_stats(scenario.profile, ExecutionMode::Idcp, trials);
        let (runtime_latency_improvement, runtime_throughput_improvement) =
            match (&naive_runtime, &idcp_runtime) {
                (Some(naive), Some(idcp)) => (
                    percent_better(
                        naive.latency_ns.mean.max(1.0),
                        idcp.latency_ns.mean.max(1.0),
                    ),
                    percent_gain(
                        naive.throughput.mean.max(1.0),
                        idcp.throughput.mean.max(1.0),
                    ),
                ),
                _ => (0.0, 0.0),
            };

        println!(
            "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.2} |",
            scenario.profile.slug(),
            scenario.purpose,
            mem_improvement,
            flow_improvement,
            copy_improvement,
            runtime_latency_improvement,
            runtime_throughput_improvement,
            score_multiplier,
        );

        println!(
            "  - default plan: transport=`{}` batching={} zero_copy={} mem_mib={:.2} copy_ns={}",
            naive_model.flow.transport.label(),
            naive_model.flow.batching,
            naive_model.flow.zero_copy_preferred,
            mib(naive_model.memory_bytes),
            naive_model.copy_penalty_ns,
        );
        println!(
            "  - idcp plan: transport=`{}` batching={} zero_copy={} mem_mib={:.2} copy_ns={}",
            idcp_model.flow.transport.label(),
            idcp_model.flow.batching,
            idcp_model.flow.zero_copy_preferred,
            mib(idcp_model.memory_bytes),
            idcp_model.copy_penalty_ns,
        );

        if let (Some(naive_flow), Some(idcp_flow)) = (&naive_flow, &idcp_flow) {
            println!(
                "  - flow_ns mean±sd (median): default={:.0}±{:.0} ({:.0}), idcp={:.0}±{:.0} ({:.0}) (n={})",
                naive_flow.mean,
                naive_flow.stddev,
                naive_flow.median,
                idcp_flow.mean,
                idcp_flow.stddev,
                idcp_flow.median,
                naive_flow.samples.min(idcp_flow.samples),
            );
        }

        if let (Some(naive_rt), Some(idcp_rt)) = (&naive_runtime, &idcp_runtime) {
            println!(
                "  - runtime latency_ns mean±sd (median): default={:.0}±{:.0} ({:.0}) (min={:.0} max={:.0}) idcp={:.0}±{:.0} ({:.0}) (min={:.0} max={:.0})",
                naive_rt.latency_ns.mean,
                naive_rt.latency_ns.stddev,
                naive_rt.latency_ns.median,
                naive_rt.latency_ns.min,
                naive_rt.latency_ns.max,
                idcp_rt.latency_ns.mean,
                idcp_rt.latency_ns.stddev,
                idcp_rt.latency_ns.median,
                idcp_rt.latency_ns.min,
                idcp_rt.latency_ns.max,
            );
            println!(
                "  - runtime throughput mean±sd: default={:.0}±{:.0} idcp={:.0}±{:.0}",
                naive_rt.throughput.mean,
                naive_rt.throughput.stddev,
                idcp_rt.throughput.mean,
                idcp_rt.throughput.stddev,
            );
        }
        println!();
    }
}

fn collect_flow_stats(
    profile: ScenarioProfile,
    mode: ExecutionMode,
    trials: usize,
) -> Option<Stats> {
    let mut samples = Vec::with_capacity(trials);
    for _ in 0..trials {
        let eval = evaluate_measured(profile, mode);
        samples.push(eval.flow_latency_ns as f64);
    }
    Stats::from(&samples)
}

fn collect_runtime_stats(
    profile: ScenarioProfile,
    mode: ExecutionMode,
    trials: usize,
) -> Option<RuntimeStats> {
    let mut latency_samples = Vec::with_capacity(trials);
    let mut throughput_samples = Vec::with_capacity(trials);

    for _ in 0..trials {
        let runtime = execute_runtime(profile, mode)?;
        latency_samples.push(runtime.end_to_end_latency_ns as f64);
        throughput_samples.push(runtime.throughput_msgs_per_sec as f64);
    }

    Some(RuntimeStats {
        latency_ns: Stats::from(&latency_samples)?,
        throughput: Stats::from(&throughput_samples)?,
    })
}

fn percent_better(base: f64, improved: f64) -> f64 {
    100.0 * (base - improved) / base
}

fn percent_gain(base: f64, improved: f64) -> f64 {
    100.0 * (improved - base) / base
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}
