use crate::registry::TaskRegistry;
use crate::task::TaskStatus;

pub struct ExpiryReport {
    pub expired_ids: Vec<u64>,
    pub energy_reclaimed: u32,
}

/// Pick the highest-priority pending task that the vessel's trust level satisfies.
pub fn next_task(registry: &TaskRegistry, vessel_trust: f64) -> Option<u64> {
    registry.by_priority()
        .into_iter()
        .find(|t| vessel_trust >= t.trust_required)
        .map(|t| t.id)
}

/// Expire all tasks past their TTL and return a report.
pub fn check_expiry(registry: &mut TaskRegistry, current_cycle: u64) -> ExpiryReport {
    let ids = registry.expire(current_cycle);
    let energy_reclaimed = energy_reclaim(registry, TaskStatus::Expired);
    ExpiryReport {
        expired_ids: ids,
        energy_reclaimed,
    }
}

/// Sum energy_budget for all tasks matching the given status.
pub fn energy_reclaim(registry: &TaskRegistry, status: TaskStatus) -> u32 {
    registry.tasks.values()
        .filter(|t| t.status == status)
        .map(|t| t.energy_budget.saturating_sub(t.energy_consumed))
        .sum()
}
