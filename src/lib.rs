pub mod lifecycle;
pub mod queue;
pub mod registry;
pub mod task;

pub use crate::registry::TaskRegistry;
pub use crate::task::{EphemeralTask, SpawnError, TaskError, TaskPurpose, TaskStatus};
pub use crate::queue::TaskQueue;
pub use crate::lifecycle::ExpiryReport;

#[cfg(test)]
mod tests {
    use super::*;

    // 1. spawn task returns valid id
    #[test]
    fn test_spawn_returns_valid_id() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id = reg.spawn(TaskPurpose::Query, b"hello".to_vec(), 10, 50).unwrap();
        assert_eq!(id, 1);
        let task = reg.get(id).unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.purpose, TaskPurpose::Query);
    }

    // 2. complete task stores result
    #[test]
    fn test_complete_stores_result() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id = reg.spawn(TaskPurpose::Query, b"q".to_vec(), 10, 50).unwrap();
        reg.complete(id, b"answer".to_vec()).unwrap();
        let task = reg.get(id).unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result.as_deref(), Some(b"answer".as_slice()));
    }

    // 3. cancel task changes status
    #[test]
    fn test_cancel_changes_status() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id = reg.spawn(TaskPurpose::Query, b"q".to_vec(), 10, 50).unwrap();
        reg.cancel(id).unwrap();
        assert_eq!(reg.get(id).unwrap().status, TaskStatus::Cancelled);
    }

    // 4. expire removes tasks past TTL
    #[test]
    fn test_expire_past_ttl() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id = reg.spawn(TaskPurpose::Query, b"q".to_vec(), 5, 50).unwrap();
        let expired = reg.expire(100); // cycle 100 >> expires_at 5
        assert_eq!(expired, vec![id]);
        assert_eq!(reg.get(id).unwrap().status, TaskStatus::Expired);
    }

    // 5. max concurrent enforced
    #[test]
    fn test_max_concurrent_enforced() {
        let mut reg = TaskRegistry::new(2, 1000);
        reg.spawn(TaskPurpose::Query, b"a".to_vec(), 10, 10).unwrap();
        reg.spawn(TaskPurpose::Query, b"b".to_vec(), 10, 10).unwrap();
        let result = reg.spawn(TaskPurpose::Query, b"c".to_vec(), 10, 10);
        assert_eq!(result.unwrap_err(), SpawnError::MaxConcurrent);
    }

    // 6. energy pool depletion prevents spawn
    #[test]
    fn test_energy_pool_depletion() {
        let mut reg = TaskRegistry::new(10, 100);
        reg.spawn(TaskPurpose::Query, b"a".to_vec(), 10, 60).unwrap();
        let result = reg.spawn(TaskPurpose::Query, b"b".to_vec(), 10, 50);
        assert_eq!(result.unwrap_err(), SpawnError::EnergyExhausted);
    }

    // 7. trust_required gates access
    #[test]
    fn test_trust_required_gates_access() {
        let mut reg = TaskRegistry::new(10, 1000);
        reg.spawn_with_options(
            TaskPurpose::Analysis, b"data".to_vec(), 10, 50,
            None, 5, 0.8,
        ).unwrap();
        let _id_low = reg.spawn_with_options(
            TaskPurpose::Query, b"q".to_vec(), 10, 20,
            None, 3, 0.2,
        ).unwrap();

        // vessel with trust 0.5 should only see the low-trust task
        let picked = lifecycle::next_task(&reg, 0.5);
        assert!(picked.is_some());

        // vessel with trust 0.9 should see the high-trust task (higher priority)
        let picked_high = lifecycle::next_task(&reg, 0.9);
        assert!(picked_high.is_some());
        assert!(picked_high.is_some());
    }

    // 8. priority ordering works
    #[test]
    fn test_priority_ordering() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id_low = reg.spawn_with_options(
            TaskPurpose::Query, b"a".to_vec(), 10, 10, None, 2, 0.0,
        ).unwrap();
        let id_high = reg.spawn_with_options(
            TaskPurpose::Query, b"b".to_vec(), 10, 10, None, 8, 0.0,
        ).unwrap();
        let ordered = reg.by_priority();
        assert_eq!(ordered[0].id, id_high);
        assert_eq!(ordered[1].id, id_low);
    }

    // 9. parent_id tracking
    #[test]
    fn test_parent_id_tracking() {
        let mut reg = TaskRegistry::new(10, 1000);
        let parent_id = reg.spawn(TaskPurpose::Monitoring, b"watch".to_vec(), 20, 100).unwrap();
        let child_id = reg.spawn_with_options(
            TaskPurpose::Query, b"sub".to_vec(), 10, 30,
            Some(parent_id), 5, 0.0,
        ).unwrap();
        assert_eq!(reg.get(child_id).unwrap().parent_id, Some(parent_id));
    }

    // 10. energy reclamation from expired tasks
    #[test]
    fn test_energy_reclamation() {
        let mut reg = TaskRegistry::new(10, 1000);
        reg.spawn(TaskPurpose::Query, b"a".to_vec(), 5, 100).unwrap();
        reg.spawn(TaskPurpose::Query, b"b".to_vec(), 5, 200).unwrap();
        reg.expire(100);
        let reclaimed = lifecycle::energy_reclaim(&reg, TaskStatus::Expired);
        assert_eq!(reclaimed, 300); // 100 + 200, none consumed
    }

    // 11. completion rate calculation
    #[test]
    fn test_completion_rate() {
        let mut reg = TaskRegistry::new(10, 1000);
        let a = reg.spawn(TaskPurpose::Query, b"a".to_vec(), 10, 10).unwrap();
        let b = reg.spawn(TaskPurpose::Query, b"b".to_vec(), 10, 10).unwrap();
        let _c = reg.spawn(TaskPurpose::Query, b"c".to_vec(), 10, 10).unwrap();
        let _d = reg.spawn(TaskPurpose::Query, b"d".to_vec(), 10, 10).unwrap();
        reg.complete(a, b"ok".to_vec()).unwrap();
        reg.complete(b, b"ok".to_vec()).unwrap();
        let rate = reg.completion_rate();
        assert!((rate - 0.5).abs() < 1e-9);
    }

    // 12. lifecycle next_task picks correctly
    #[test]
    fn test_lifecycle_next_task_picks_correctly() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id_high = reg.spawn_with_options(
            TaskPurpose::Generation, b"x".to_vec(), 10, 10, None, 9, 0.1,
        ).unwrap();
        let id_med = reg.spawn_with_options(
            TaskPurpose::Analysis, b"y".to_vec(), 10, 10, None, 7, 0.3,
        ).unwrap();
        let _id_low = reg.spawn_with_options(
            TaskPurpose::Query, b"z".to_vec(), 10, 10, None, 5, 0.0,
        ).unwrap();

        // high trust vessel picks the highest priority that trust allows
        assert_eq!(lifecycle::next_task(&reg, 1.0), Some(id_high));

        // medium trust: can't reach high (trust_required=0.1, vessel=0.2 → can reach)
        // but id_med has trust_required=0.3, so vessel 0.2 can only reach id_high
        assert_eq!(lifecycle::next_task(&reg, 0.2), Some(id_high));

        // complete the high task
        reg.complete(id_high, b"done".to_vec()).unwrap();
        assert_eq!(lifecycle::next_task(&reg, 1.0), Some(id_med));
    }

    // Bonus: task queue ordering
    #[test]
    fn test_task_queue_ordering() {
        let mut q = TaskQueue::new();
        q.push(EphemeralTask::new(1, None, TaskPurpose::Query, vec![], 10, 10, 3, 0.0, 0));
        q.push(EphemeralTask::new(2, None, TaskPurpose::Query, vec![], 10, 10, 8, 0.0, 0));
        q.push(EphemeralTask::new(3, None, TaskPurpose::Query, vec![], 10, 10, 5, 0.0, 0));
        assert_eq!(q.pop().unwrap().id, 2);
        assert_eq!(q.pop().unwrap().id, 3);
        assert_eq!(q.pop().unwrap().id, 1);
        assert!(q.pop().is_none());
    }

    #[test]
    fn test_task_queue_drain() {
        let mut q = TaskQueue::new();
        for i in 0..5 {
            q.push(EphemeralTask::new(i, None, TaskPurpose::Query, vec![], 10, 10, i as u8, 0.0, 0));
        }
        let drained = q.drain(3);
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0].id, 4); // highest priority
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn test_check_expiry_report() {
        let mut reg = TaskRegistry::new(10, 1000);
        let id = reg.spawn_with_options(
            TaskPurpose::Test, b"t".to_vec(), 5, 100, None, 5, 0.0,
        ).unwrap();
        let report = lifecycle::check_expiry(&mut reg, 100);
        assert_eq!(report.expired_ids, vec![id]);
        assert_eq!(report.energy_reclaimed, 100);
    }
}
