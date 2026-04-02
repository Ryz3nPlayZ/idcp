use idcp_memory::MemoryReport;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PressureLevel {
    Healthy,
    Elevated,
    Critical,
}

#[derive(Clone, Debug)]
pub struct PressureInputs {
    pub ram_budget_bytes: usize,
    pub working_set_bytes: usize,
    pub memory_report: MemoryReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PressurePlan {
    pub level: PressureLevel,
    pub compress_cold: bool,
    pub enable_page_families: bool,
    pub rebalance_work: bool,
    pub estimated_relief_bytes: usize,
}

pub fn evaluate_pressure(input: PressureInputs) -> PressurePlan {
    let oversubscription = input
        .working_set_bytes
        .saturating_sub(input.ram_budget_bytes);
    let relief = input
        .memory_report
        .raw_bytes
        .saturating_sub(input.memory_report.estimated_bytes);
    if oversubscription == 0 {
        return PressurePlan {
            level: PressureLevel::Healthy,
            compress_cold: false,
            enable_page_families: input.memory_report.savings_percent() > 25.0,
            rebalance_work: false,
            estimated_relief_bytes: relief,
        };
    }
    if oversubscription <= input.ram_budget_bytes / 10 {
        return PressurePlan {
            level: PressureLevel::Elevated,
            compress_cold: true,
            enable_page_families: true,
            rebalance_work: relief < oversubscription,
            estimated_relief_bytes: relief,
        };
    }
    PressurePlan {
        level: PressureLevel::Critical,
        compress_cold: true,
        enable_page_families: true,
        rebalance_work: true,
        estimated_relief_bytes: relief,
    }
}
