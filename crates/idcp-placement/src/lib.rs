#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionZone {
    L1Hot,
    CoreLocal,
    SharedLocal,
    CrossProcess,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementRequest {
    pub hot_bytes: usize,
    pub shared_fraction_percent: u8,
    pub latency_sensitive: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementDecision {
    pub zone: ExecutionZone,
    pub estimated_copy_penalty_ns: u64,
    pub affinity_score: u8,
}

pub fn choose_placement(request: PlacementRequest) -> PlacementDecision {
    if request.hot_bytes <= 32 * 1024 && request.latency_sensitive {
        return PlacementDecision {
            zone: ExecutionZone::L1Hot,
            estimated_copy_penalty_ns: 40,
            affinity_score: 95,
        };
    }
    if request.shared_fraction_percent >= 60 {
        return PlacementDecision {
            zone: ExecutionZone::SharedLocal,
            estimated_copy_penalty_ns: 180,
            affinity_score: 82,
        };
    }
    if request.hot_bytes <= 512 * 1024 {
        return PlacementDecision {
            zone: ExecutionZone::CoreLocal,
            estimated_copy_penalty_ns: 110,
            affinity_score: 88,
        };
    }
    PlacementDecision {
        zone: ExecutionZone::CrossProcess,
        estimated_copy_penalty_ns: 320,
        affinity_score: 71,
    }
}
