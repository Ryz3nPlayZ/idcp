use idcp_flow::{FlowHint, Locality, PayloadClass, choose_flow_plan};
use idcp_memory::{analyze_workload, mixed_workload};
use idcp_placement::{PlacementRequest, choose_placement};
use idcp_pressure::{PressureInputs, evaluate_pressure};

fn main() {
    let memory = analyze_workload(&mixed_workload());
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
        working_set_bytes: memory.raw_bytes + 512 * idcp_memory::PAGE_SIZE,
        memory_report: memory,
    });

    println!("idcpd plan");
    println!(
        "flow transport={} batching={} zero_copy={}",
        flow.transport.label(),
        flow.batching,
        flow.zero_copy_preferred
    );
    println!(
        "placement zone={:?} affinity_score={} copy_penalty_ns={}",
        placement.zone, placement.affinity_score, placement.estimated_copy_penalty_ns
    );
    println!(
        "pressure level={:?} compress_cold={} enable_page_families={} rebalance_work={} estimated_relief_mib={:.2}",
        pressure.level,
        pressure.compress_cold,
        pressure.enable_page_families,
        pressure.rebalance_work,
        pressure.estimated_relief_bytes as f64 / 1024.0 / 1024.0
    );
}
