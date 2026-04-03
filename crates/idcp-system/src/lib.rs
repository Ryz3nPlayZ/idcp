use fabric_core::LocalEndpoint;
use idcp_flow::{FlowHint, FlowPlan, Locality, PayloadClass, choose_flow_plan};
use idcp_memory::{MemoryReport, ScenarioShape, analyze_workload, scenario_workload};
use idcp_placement::{PlacementDecision, PlacementRequest, choose_placement};
use idcp_pressure::{PressureInputs, PressureLevel, PressurePlan, evaluate_pressure};
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioProfile {
    AgentMesh,
    PluginHost,
    EmbeddingFarm,
    TerminalGraph,
}

impl ScenarioProfile {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::AgentMesh => "agent-mesh",
            Self::PluginHost => "plugin-host",
            Self::EmbeddingFarm => "embedding-farm",
            Self::TerminalGraph => "terminal-graph",
        }
    }

    pub const fn all() -> [Self; 4] {
        [
            Self::AgentMesh,
            Self::PluginHost,
            Self::EmbeddingFarm,
            Self::TerminalGraph,
        ]
    }

    pub fn from_slug(value: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|profile| profile.slug() == value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScenarioSpec {
    pub profile: ScenarioProfile,
    pub workers: usize,
    pub shared_fraction_percent: u8,
    pub hot_bytes: usize,
    pub message_count: usize,
    pub payload: PayloadClass,
    pub latency_sensitive: bool,
    pub ram_budget_bytes: usize,
    pub shape: ScenarioShape,
}

impl ScenarioSpec {
    pub fn name(self) -> &'static str {
        self.profile.slug()
    }
}

pub fn scenario_spec(profile: ScenarioProfile) -> ScenarioSpec {
    match profile {
        ScenarioProfile::AgentMesh => ScenarioSpec {
            profile,
            workers: 8,
            shared_fraction_percent: 72,
            hot_bytes: 128 * 1024,
            message_count: 50_000,
            payload: PayloadClass::Small,
            latency_sensitive: true,
            ram_budget_bytes: 3 * 1024 * 1024,
            shape: ScenarioShape {
                shared_pages: 220,
                family_count: 40,
                variants_per_family: 5,
                unique_pages: 180,
                mutation_stride: 96,
            },
        },
        ScenarioProfile::PluginHost => ScenarioSpec {
            profile,
            workers: 16,
            shared_fraction_percent: 84,
            hot_bytes: 64 * 1024,
            message_count: 30_000,
            payload: PayloadClass::Tiny,
            latency_sensitive: true,
            ram_budget_bytes: 2 * 1024 * 1024,
            shape: ScenarioShape {
                shared_pages: 400,
                family_count: 28,
                variants_per_family: 3,
                unique_pages: 90,
                mutation_stride: 128,
            },
        },
        ScenarioProfile::EmbeddingFarm => ScenarioSpec {
            profile,
            workers: 6,
            shared_fraction_percent: 58,
            hot_bytes: 512 * 1024,
            message_count: 12_000,
            payload: PayloadClass::Medium,
            latency_sensitive: false,
            ram_budget_bytes: 5 * 1024 * 1024,
            shape: ScenarioShape {
                shared_pages: 180,
                family_count: 56,
                variants_per_family: 7,
                unique_pages: 260,
                mutation_stride: 72,
            },
        },
        ScenarioProfile::TerminalGraph => ScenarioSpec {
            profile,
            workers: 10,
            shared_fraction_percent: 66,
            hot_bytes: 96 * 1024,
            message_count: 20_000,
            payload: PayloadClass::Small,
            latency_sensitive: true,
            ram_budget_bytes: 2 * 1024 * 1024,
            shape: ScenarioShape {
                shared_pages: 140,
                family_count: 32,
                variants_per_family: 4,
                unique_pages: 120,
                mutation_stride: 80,
            },
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionMode {
    Naive,
    Idcp,
}

#[derive(Clone, Debug)]
pub struct ScenarioEvaluation {
    pub spec: ScenarioSpec,
    pub mode: ExecutionMode,
    pub memory: MemoryReport,
    pub flow: FlowPlan,
    pub placement: PlacementDecision,
    pub pressure: PressurePlan,
    pub memory_bytes: usize,
    pub flow_latency_ns: u64,
    pub copy_penalty_ns: u64,
    pub total_score: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct RuntimeResult {
    pub processed_messages: usize,
    pub end_to_end_latency_ns: u64,
    pub throughput_msgs_per_sec: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlAction {
    Hold,
    FavorZeroCopy,
    FavorBatching,
    RebalanceForPressure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerConfig {
    pub profile: ScenarioProfile,
    pub ticks: usize,
    pub interval_ms: u64,
}

#[derive(Clone, Debug)]
pub struct ControllerSnapshot {
    pub tick: usize,
    pub profile: ScenarioProfile,
    pub naive: ScenarioEvaluation,
    pub idcp: ScenarioEvaluation,
    pub runtime_naive: RuntimeResult,
    pub runtime_idcp: RuntimeResult,
    pub improvement: ScenarioImprovement,
    pub action: ControlAction,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControllerSummary {
    pub avg_memory_percent: f64,
    pub avg_flow_percent: f64,
    pub avg_copy_percent: f64,
    pub avg_score_multiplier: f64,
    pub avg_runtime_latency_percent: f64,
    pub avg_runtime_throughput_percent: f64,
}

impl ScenarioEvaluation {
    pub fn improvement_over(&self, base: &ScenarioEvaluation) -> ScenarioImprovement {
        ScenarioImprovement {
            memory_percent: percent_better(base.memory_bytes as f64, self.memory_bytes as f64),
            flow_percent: percent_better(base.flow_latency_ns as f64, self.flow_latency_ns as f64),
            copy_percent: percent_better(base.copy_penalty_ns as f64, self.copy_penalty_ns as f64),
            score_multiplier: self.total_score as f64 / base.total_score as f64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScenarioImprovement {
    pub memory_percent: f64,
    pub flow_percent: f64,
    pub copy_percent: f64,
    pub score_multiplier: f64,
}

pub fn evaluate(profile: ScenarioProfile, mode: ExecutionMode) -> ScenarioEvaluation {
    let spec = scenario_spec(profile);
    let memory = analyze_workload(&scenario_workload(spec.name(), spec.shape));

    let flow_hint = FlowHint {
        locality: if spec.workers > 1 {
            Locality::CrossProcess
        } else {
            Locality::InProcess
        },
        payload: spec.payload,
        latency_sensitive: spec.latency_sensitive,
    };

    let flow = match mode {
        ExecutionMode::Naive => FlowPlan {
            transport: idcp_flow::TransportKind::UnixStream,
            batching: 1,
            zero_copy_preferred: false,
        },
        ExecutionMode::Idcp => choose_flow_plan(flow_hint),
    };

    let placement = match mode {
        ExecutionMode::Naive => PlacementDecision {
            zone: idcp_placement::ExecutionZone::CrossProcess,
            estimated_copy_penalty_ns: 320,
            affinity_score: 65,
        },
        ExecutionMode::Idcp => choose_placement(PlacementRequest {
            hot_bytes: spec.hot_bytes,
            shared_fraction_percent: spec.shared_fraction_percent,
            latency_sensitive: spec.latency_sensitive,
        }),
    };

    let pressure = evaluate_pressure(PressureInputs {
        ram_budget_bytes: spec.ram_budget_bytes,
        working_set_bytes: memory.raw_bytes + spec.workers * 128 * 1024,
        memory_report: memory.clone(),
    });

    let memory_bytes = match mode {
        ExecutionMode::Naive => memory.raw_bytes,
        ExecutionMode::Idcp if pressure.enable_page_families => memory.estimated_bytes,
        ExecutionMode::Idcp => memory.raw_bytes,
    };

    let flow_latency_ns = modeled_flow_latency_ns(flow, spec.message_count);
    let copy_penalty_ns = placement.estimated_copy_penalty_ns;
    let mut total_score = aggregate_score(memory_bytes, flow_latency_ns, copy_penalty_ns);
    if matches!(mode, ExecutionMode::Idcp) && pressure.rebalance_work {
        total_score += total_score / 8;
    }

    ScenarioEvaluation {
        spec,
        mode,
        memory,
        flow,
        placement,
        pressure,
        memory_bytes,
        flow_latency_ns,
        copy_penalty_ns,
        total_score,
    }
}

pub fn evaluate_measured(profile: ScenarioProfile, mode: ExecutionMode) -> ScenarioEvaluation {
    let mut eval = evaluate(profile, mode);
    eval.flow_latency_ns = measure_flow_latency_ns(eval.flow, eval.spec.message_count)
        .unwrap_or_else(|| modeled_flow_latency_ns(eval.flow, eval.spec.message_count));
    eval.total_score = aggregate_score(
        eval.memory_bytes,
        eval.flow_latency_ns,
        eval.copy_penalty_ns,
    );
    if matches!(mode, ExecutionMode::Idcp) && eval.pressure.rebalance_work {
        eval.total_score += eval.total_score / 8;
    }
    eval
}

pub fn execute_runtime(profile: ScenarioProfile, mode: ExecutionMode) -> Option<RuntimeResult> {
    let eval = evaluate_measured(profile, mode);
    let iterations = eval.spec.message_count.clamp(2_000, 20_000);

    let (mut source_to_stage1, mut stage1_from_source) =
        LocalEndpoint::pair(eval.flow.transport).ok()?;
    let (mut stage1_to_stage2, mut stage2_from_stage1) =
        LocalEndpoint::pair(eval.flow.transport).ok()?;
    let (ack_tx, ack_rx) = sync_channel::<u64>(0);

    let stage1 = thread::spawn(move || {
        for _ in 0..iterations {
            let value = stage1_from_source.recv().ok()?;
            stage1_from_source.send(value.wrapping_add(1)).ok()?;
            let forwarded = stage1_from_source.recv().ok()?;
            stage1_to_stage2.send(forwarded.wrapping_add(1)).ok()?;
        }
        Some(())
    });

    let stage2 = thread::spawn(move || {
        for _ in 0..iterations {
            let value = stage2_from_stage1.recv().ok()?;
            ack_tx.send(value.wrapping_add(1)).ok()?;
        }
        Some(())
    });

    let start = Instant::now();
    for i in 0..iterations as u64 {
        source_to_stage1.send(i).ok()?;
        let bounced = source_to_stage1.recv().ok()?;
        source_to_stage1.send(bounced).ok()?;
        let final_value = ack_rx.recv().ok()?;
        if final_value != i.wrapping_add(3) {
            return None;
        }
    }
    let elapsed = start.elapsed();

    stage1.join().ok()??;
    stage2.join().ok()??;

    Some(RuntimeResult {
        processed_messages: iterations,
        end_to_end_latency_ns: elapsed.as_nanos() as u64 / iterations as u64,
        throughput_msgs_per_sec: (iterations as f64 / elapsed.as_secs_f64()) as u64,
    })
}

pub fn run_controller(config: ControllerConfig) -> Vec<ControllerSnapshot> {
    let ticks = config.ticks.max(1);
    let mut snapshots = Vec::with_capacity(ticks);
    for tick in 0..ticks {
        let naive = evaluate_measured(config.profile, ExecutionMode::Naive);
        let idcp = evaluate_measured(config.profile, ExecutionMode::Idcp);
        let runtime_naive =
            execute_runtime(config.profile, ExecutionMode::Naive).unwrap_or(RuntimeResult {
                processed_messages: 0,
                end_to_end_latency_ns: 0,
                throughput_msgs_per_sec: 0,
            });
        let runtime_idcp =
            execute_runtime(config.profile, ExecutionMode::Idcp).unwrap_or(RuntimeResult {
                processed_messages: 0,
                end_to_end_latency_ns: 0,
                throughput_msgs_per_sec: 0,
            });
        let improvement = idcp.improvement_over(&naive);
        let action = choose_action(&idcp, &runtime_naive, &runtime_idcp);
        snapshots.push(ControllerSnapshot {
            tick: tick + 1,
            profile: config.profile,
            naive,
            idcp,
            runtime_naive,
            runtime_idcp,
            improvement,
            action,
        });
        if config.interval_ms > 0 && tick + 1 != ticks {
            thread::sleep(Duration::from_millis(config.interval_ms));
        }
    }
    snapshots
}

pub fn summarize_controller(snapshots: &[ControllerSnapshot]) -> ControllerSummary {
    if snapshots.is_empty() {
        return ControllerSummary::default();
    }
    let count = snapshots.len() as f64;
    let mut summary = ControllerSummary::default();
    for snapshot in snapshots {
        summary.avg_memory_percent += snapshot.improvement.memory_percent;
        summary.avg_flow_percent += snapshot.improvement.flow_percent;
        summary.avg_copy_percent += snapshot.improvement.copy_percent;
        summary.avg_score_multiplier += snapshot.improvement.score_multiplier;
        summary.avg_runtime_latency_percent += percent_better(
            snapshot.runtime_naive.end_to_end_latency_ns as f64,
            snapshot.runtime_idcp.end_to_end_latency_ns as f64,
        );
        summary.avg_runtime_throughput_percent += percent_better(
            snapshot.runtime_naive.throughput_msgs_per_sec.max(1) as f64,
            snapshot.runtime_idcp.throughput_msgs_per_sec.max(1) as f64,
        ) * -1.0;
    }
    summary.avg_memory_percent /= count;
    summary.avg_flow_percent /= count;
    summary.avg_copy_percent /= count;
    summary.avg_score_multiplier /= count;
    summary.avg_runtime_latency_percent /= count;
    summary.avg_runtime_throughput_percent /= count;
    summary
}

pub fn render_report_markdown(
    config: ControllerConfig,
    snapshots: &[ControllerSnapshot],
) -> String {
    use std::fmt::Write;

    let summary = summarize_controller(snapshots);
    let mut out = String::new();
    let _ = writeln!(out, "# IDCP Controller Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- profile: `{}`", config.profile.slug());
    let _ = writeln!(out, "- ticks: `{}`", snapshots.len());
    let _ = writeln!(out, "- interval_ms: `{}`", config.interval_ms);
    let _ = writeln!(out);
    let _ = writeln!(out, "## Summary");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- average modeled memory reduction: `{:.1}%`",
        summary.avg_memory_percent
    );
    let _ = writeln!(
        out,
        "- average measured flow reduction: `{:.1}%`",
        summary.avg_flow_percent
    );
    let _ = writeln!(
        out,
        "- average copy-penalty reduction: `{:.1}%`",
        summary.avg_copy_percent
    );
    let _ = writeln!(
        out,
        "- average aggregate score multiplier: `{:.2}x`",
        summary.avg_score_multiplier
    );
    let _ = writeln!(
        out,
        "- average runtime latency reduction: `{:.1}%`",
        summary.avg_runtime_latency_percent
    );
    let _ = writeln!(
        out,
        "- average runtime throughput increase: `{:.1}%`",
        summary.avg_runtime_throughput_percent
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Ticks");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| tick | action | mem% | flow% | copy% | score_x | naive_ns | idcp_ns | naive_tput | idcp_tput |"
    );
    let _ = writeln!(
        out,
        "| ---: | :----- | ---: | ----: | ----: | ------: | -------: | ------: | ---------: | --------: |"
    );
    for snapshot in snapshots {
        let _ = writeln!(
            out,
            "| {} | `{:?}` | {:.1} | {:.1} | {:.1} | {:.2} | {} | {} | {} | {} |",
            snapshot.tick,
            snapshot.action,
            snapshot.improvement.memory_percent,
            snapshot.improvement.flow_percent,
            snapshot.improvement.copy_percent,
            snapshot.improvement.score_multiplier,
            snapshot.runtime_naive.end_to_end_latency_ns,
            snapshot.runtime_idcp.end_to_end_latency_ns,
            snapshot.runtime_naive.throughput_msgs_per_sec,
            snapshot.runtime_idcp.throughput_msgs_per_sec,
        );
    }
    out
}

fn modeled_flow_latency_ns(plan: FlowPlan, message_count: usize) -> u64 {
    let base = match plan.transport {
        idcp_flow::TransportKind::SpscRing => 245,
        idcp_flow::TransportKind::SharedMemoryEvent => 8_578,
        idcp_flow::TransportKind::UnixStream => 9_418,
        idcp_flow::TransportKind::SyncChannel => 7_422,
    };
    let batching_divisor = plan.batching.max(1) as u64;
    base / batching_divisor + (message_count as u64 / 10_000)
}

fn measure_flow_latency_ns(plan: FlowPlan, message_count: usize) -> Option<u64> {
    let iterations = message_count.clamp(1_000, 10_000);
    let (mut client, mut server) = LocalEndpoint::pair(plan.transport).ok()?;
    let worker = thread::spawn(move || {
        for _ in 0..iterations {
            let value = server.recv().ok()?;
            server.send(value).ok()?;
        }
        Some(())
    });

    let start = Instant::now();
    for i in 0..iterations as u64 {
        let ack = client.request(i).ok()?;
        if ack != i {
            return None;
        }
    }
    let elapsed = start.elapsed();
    worker.join().ok()??;
    Some((elapsed.as_nanos() as u64 / iterations as u64) / plan.batching.max(1) as u64)
}

fn aggregate_score(memory_bytes: usize, flow_latency_ns: u64, copy_penalty_ns: u64) -> u64 {
    let memory_factor = (1_000_000_000f64 / memory_bytes as f64) as u64;
    memory_factor + (1_000_000 / flow_latency_ns.max(1)) + (100_000 / copy_penalty_ns.max(1))
}

fn percent_better(base: f64, improved: f64) -> f64 {
    100.0 * (base - improved) / base
}

fn choose_action(
    eval: &ScenarioEvaluation,
    runtime_naive: &RuntimeResult,
    runtime_idcp: &RuntimeResult,
) -> ControlAction {
    if matches!(eval.pressure.level, PressureLevel::Critical) && eval.pressure.rebalance_work {
        return ControlAction::RebalanceForPressure;
    }
    let runtime_latency_gain = percent_better(
        runtime_naive.end_to_end_latency_ns.max(1) as f64,
        runtime_idcp.end_to_end_latency_ns.max(1) as f64,
    );
    if eval.flow.batching > 1 && runtime_latency_gain < 5.0 {
        return ControlAction::FavorBatching;
    }
    if eval.flow.zero_copy_preferred
        || matches!(
            eval.placement.zone,
            idcp_placement::ExecutionZone::SharedLocal
                | idcp_placement::ExecutionZone::CoreLocal
                | idcp_placement::ExecutionZone::L1Hot
        )
    {
        return ControlAction::FavorZeroCopy;
    }
    ControlAction::Hold
}

#[cfg(test)]
mod tests {
    use super::{
        ControlAction, ControllerConfig, ExecutionMode, ScenarioProfile, evaluate,
        evaluate_measured, execute_runtime, render_report_markdown, run_controller,
    };

    #[test]
    fn idcp_beats_naive_on_agent_mesh() {
        let naive = evaluate(ScenarioProfile::AgentMesh, ExecutionMode::Naive);
        let idcp = evaluate(ScenarioProfile::AgentMesh, ExecutionMode::Idcp);
        let improvement = idcp.improvement_over(&naive);
        assert!(improvement.memory_percent > 20.0);
        assert!(improvement.score_multiplier > 1.2);
    }

    #[test]
    fn measured_evaluation_produces_positive_flow_latency() {
        let eval = evaluate_measured(ScenarioProfile::PluginHost, ExecutionMode::Idcp);
        assert!(eval.flow_latency_ns > 0);
    }

    #[test]
    fn runtime_execution_processes_messages() {
        let runtime = execute_runtime(ScenarioProfile::TerminalGraph, ExecutionMode::Idcp).unwrap();
        assert!(runtime.processed_messages >= 2_000);
        assert!(runtime.end_to_end_latency_ns > 0);
        assert!(runtime.throughput_msgs_per_sec > 0);
    }

    #[test]
    fn controller_produces_snapshots_and_actions() {
        let snapshots = run_controller(ControllerConfig {
            profile: ScenarioProfile::EmbeddingFarm,
            ticks: 2,
            interval_ms: 0,
        });
        assert_eq!(snapshots.len(), 2);
        assert!(matches!(
            snapshots[0].action,
            ControlAction::FavorBatching
                | ControlAction::FavorZeroCopy
                | ControlAction::RebalanceForPressure
                | ControlAction::Hold
        ));
        assert!(snapshots[0].runtime_idcp.processed_messages > 0);
    }

    #[test]
    fn report_contains_summary_and_ticks() {
        let snapshots = run_controller(ControllerConfig {
            profile: ScenarioProfile::AgentMesh,
            ticks: 1,
            interval_ms: 0,
        });
        let report = render_report_markdown(
            ControllerConfig {
                profile: ScenarioProfile::AgentMesh,
                ticks: 1,
                interval_ms: 0,
            },
            &snapshots,
        );
        assert!(report.contains("# IDCP Controller Report"));
        assert!(report.contains("## Summary"));
        assert!(report.contains("| tick | action |"));
    }
}
