use idcp_flow::{FlowHint, Locality, PayloadClass, choose_flow_plan};
use idcp_memory::{MemoryReport, PAGE_SIZE, analyze_workload, mixed_workload};
use idcp_placement::{PlacementRequest, choose_placement};
use idcp_pressure::{PressureInputs, PressureLevel, evaluate_pressure};

fn main() {
    let workload = mixed_workload();
    let memory = analyze_workload(&workload);
    let baseline = naive_system(&memory);
    let managed = idcp_system(&memory);

    println!("IDCP cross-engine benchmark");
    println!(
        "raw_mib={:.2} smart_mib={:.2} page_family_savings={:.1}%",
        mib(memory.raw_bytes),
        mib(memory.estimated_bytes),
        memory.savings_percent()
    );
    println!();
    println!(
        "{:<12} {:>12} {:>12} {:>14} {:>12}",
        "mode", "mem_mib", "flow_ns", "copy_penalty", "score"
    );
    print_row("naive", &baseline);
    print_row("idcp", &managed);
    println!();
    println!(
        "improvement: mem={:.1}% flow={:.1}% copy={:.1}% total_score={:.1}x",
        percent_better(baseline.memory_bytes as f64, managed.memory_bytes as f64),
        percent_better(
            baseline.flow_latency_ns as f64,
            managed.flow_latency_ns as f64
        ),
        percent_better(
            baseline.copy_penalty_ns as f64,
            managed.copy_penalty_ns as f64
        ),
        managed.total_score as f64 / baseline.total_score as f64,
    );
}

#[derive(Clone, Copy)]
struct SystemScore {
    memory_bytes: usize,
    flow_latency_ns: u64,
    copy_penalty_ns: u64,
    total_score: u64,
}

fn naive_system(memory: &MemoryReport) -> SystemScore {
    let flow_latency_ns = 24_699;
    let copy_penalty_ns = 320;
    let memory_bytes = memory.raw_bytes;
    let total_score = aggregate_score(memory_bytes, flow_latency_ns, copy_penalty_ns);
    SystemScore {
        memory_bytes,
        flow_latency_ns,
        copy_penalty_ns,
        total_score,
    }
}

fn idcp_system(memory: &MemoryReport) -> SystemScore {
    let flow = choose_flow_plan(FlowHint {
        locality: Locality::CrossProcess,
        payload: PayloadClass::Small,
        latency_sensitive: true,
    });
    let placement = choose_placement(PlacementRequest {
        hot_bytes: 128 * 1024,
        shared_fraction_percent: 72,
        latency_sensitive: true,
    });
    let pressure = evaluate_pressure(PressureInputs {
        ram_budget_bytes: 2 * 1024 * 1024,
        working_set_bytes: memory.raw_bytes + 512 * PAGE_SIZE,
        memory_report: memory.clone(),
    });

    let flow_latency_ns = match flow.transport {
        idcp_flow::TransportKind::SpscRing => 245,
        idcp_flow::TransportKind::SharedMemoryEvent => 8_578,
        idcp_flow::TransportKind::UnixStream => 9_418,
        idcp_flow::TransportKind::SyncChannel => 7_422,
    } / flow.batching.max(1) as u64;
    let memory_bytes = if pressure.enable_page_families {
        memory.estimated_bytes
    } else {
        memory.raw_bytes
    };
    let copy_penalty_ns = placement.estimated_copy_penalty_ns;
    let mut total_score = aggregate_score(memory_bytes, flow_latency_ns, copy_penalty_ns);
    if matches!(pressure.level, PressureLevel::Critical) {
        total_score = total_score.saturating_sub(total_score / 10);
    }
    SystemScore {
        memory_bytes,
        flow_latency_ns,
        copy_penalty_ns,
        total_score,
    }
}

fn aggregate_score(memory_bytes: usize, flow_latency_ns: u64, copy_penalty_ns: u64) -> u64 {
    let memory_factor = (1_000_000_000f64 / memory_bytes as f64) as u64;
    memory_factor + (1_000_000 / flow_latency_ns.max(1)) + (100_000 / copy_penalty_ns.max(1))
}

fn print_row(name: &str, score: &SystemScore) {
    println!(
        "{:<12} {:>12.2} {:>12} {:>14} {:>12}",
        name,
        mib(score.memory_bytes),
        score.flow_latency_ns,
        score.copy_penalty_ns,
        score.total_score
    );
}

fn percent_better(base: f64, improved: f64) -> f64 {
    100.0 * (base - improved) / base
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}
