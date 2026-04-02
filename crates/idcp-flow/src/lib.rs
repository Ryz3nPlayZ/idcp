pub use fabric_core::{FabricError, FabricResult, LocalEndpoint, TransportKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locality {
    InThread,
    InProcess,
    CrossProcess,
    CrossHost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadClass {
    Tiny,
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlowHint {
    pub locality: Locality,
    pub payload: PayloadClass,
    pub latency_sensitive: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlowPlan {
    pub transport: TransportKind,
    pub batching: usize,
    pub zero_copy_preferred: bool,
}

pub fn choose_flow_plan(hint: FlowHint) -> FlowPlan {
    match (hint.locality, hint.payload, hint.latency_sensitive) {
        (Locality::InThread, _, _) => FlowPlan {
            transport: TransportKind::SpscRing,
            batching: 1,
            zero_copy_preferred: true,
        },
        (Locality::InProcess, PayloadClass::Tiny | PayloadClass::Small, true) => FlowPlan {
            transport: TransportKind::SpscRing,
            batching: 1,
            zero_copy_preferred: true,
        },
        (Locality::CrossProcess, PayloadClass::Tiny | PayloadClass::Small, true) => FlowPlan {
            transport: TransportKind::SharedMemoryEvent,
            batching: 1,
            zero_copy_preferred: true,
        },
        (Locality::CrossProcess, PayloadClass::Medium | PayloadClass::Large, _) => FlowPlan {
            transport: TransportKind::SharedMemoryEvent,
            batching: 16,
            zero_copy_preferred: true,
        },
        (Locality::CrossHost, _, _) => FlowPlan {
            transport: TransportKind::UnixStream,
            batching: 32,
            zero_copy_preferred: false,
        },
        _ => FlowPlan {
            transport: TransportKind::UnixStream,
            batching: 8,
            zero_copy_preferred: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{FlowHint, Locality, PayloadClass, TransportKind, choose_flow_plan};

    #[test]
    fn cross_process_small_prefers_shared_memory() {
        let plan = choose_flow_plan(FlowHint {
            locality: Locality::CrossProcess,
            payload: PayloadClass::Small,
            latency_sensitive: true,
        });
        assert_eq!(plan.transport, TransportKind::SharedMemoryEvent);
        assert_eq!(plan.batching, 1);
    }
}
