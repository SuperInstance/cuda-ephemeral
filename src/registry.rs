use crate::task::{EphemeralTask, SpawnError, TaskError, TaskPurpose, TaskStatus};
use std::collections::HashMap;

pub struct TaskRegistry {
    pub(crate) tasks: HashMap<u64, EphemeralTask>,
    next_id: u64,
    max_concurrent: usize,
    energy_pool: u32,
    current_cycle: u64,
}

impl TaskRegistry {
    pub fn new(max_concurrent: usize, energy_pool: u32) -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
            max_concurrent,
            energy_pool,
            current_cycle: 0,
        }
    }

    pub fn with_cycle(mut self, cycle: u64) -> Self {
        self.current_cycle = cycle;
        self
    }

    /// Count active (non-terminal) tasks.
    fn active_count(&self) -> usize {
        self.tasks.values().filter(|t| !Self::is_terminal(t.status)).count()
    }

    fn is_terminal(status: TaskStatus) -> bool {
        matches!(status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Expired | TaskStatus::Cancelled)
    }

    /// Total energy currently committed to active tasks.
    fn active_energy(&self) -> u32 {
        self.tasks.values()
            .filter(|t| !Self::is_terminal(t.status))
            .map(|t| t.energy_budget.saturating_sub(t.energy_consumed))
            .fold(0u32, |a, b| a.saturating_add(b))
    }

    pub fn spawn(
        &mut self,
        purpose: TaskPurpose,
        payload: Vec<u8>,
        ttl: u32,
        energy_budget: u32,
    ) -> Result<u64, SpawnError> {
        if self.active_count() >= self.max_concurrent {
            return Err(SpawnError::MaxConcurrent);
        }

        let available = self.energy_pool.saturating_sub(self.active_energy());
        if energy_budget > available {
            return Err(SpawnError::EnergyExhausted);
        }

        let id = self.next_id;
        self.next_id += 1;

        let task = EphemeralTask::new(
            id, None, purpose, payload, energy_budget, ttl, 5, 0.0, self.current_cycle,
        );
        self.tasks.insert(id, task);
        Ok(id)
    }

    pub fn spawn_with_options(
        &mut self,
        purpose: TaskPurpose,
        payload: Vec<u8>,
        ttl: u32,
        energy_budget: u32,
        parent_id: Option<u64>,
        priority: u8,
        trust_required: f64,
    ) -> Result<u64, SpawnError> {
        if self.active_count() >= self.max_concurrent {
            return Err(SpawnError::MaxConcurrent);
        }

        let available = self.energy_pool.saturating_sub(self.active_energy());
        if energy_budget > available {
            return Err(SpawnError::EnergyExhausted);
        }

        let id = self.next_id;
        self.next_id += 1;

        let task = EphemeralTask::new(
            id, parent_id, purpose, payload, energy_budget, ttl, priority, trust_required, self.current_cycle,
        );
        self.tasks.insert(id, task);
        Ok(id)
    }

    pub fn complete(&mut self, id: u64, result: Vec<u8>) -> Result<(), TaskError> {
        let task = self.tasks.get_mut(&id).ok_or(TaskError::NotFound(id))?;
        if task.status != TaskStatus::Pending && task.status != TaskStatus::Running {
            return Err(TaskError::InvalidTransition {
                id,
                from: task.status,
                to: TaskStatus::Completed,
            });
        }
        task.status = TaskStatus::Completed;
        task.result = Some(result);
        task.energy_consumed = task.energy_budget; // assume full consumption on completion
        Ok(())
    }

    pub fn cancel(&mut self, id: u64) -> Result<(), TaskError> {
        let task = self.tasks.get_mut(&id).ok_or(TaskError::NotFound(id))?;
        if Self::is_terminal(task.status) {
            return Err(TaskError::InvalidTransition {
                id,
                from: task.status,
                to: TaskStatus::Cancelled,
            });
        }
        task.status = TaskStatus::Cancelled;
        Ok(())
    }

    pub fn expire(&mut self, current_cycle: u64) -> Vec<u64> {
        let mut expired = Vec::new();
        for task in self.tasks.values_mut() {
            if !Self::is_terminal(task.status) && task.expires_at <= current_cycle {
                task.status = TaskStatus::Expired;
                expired.push(task.id);
            }
        }
        expired
    }

    pub fn advance_cycle(&mut self) {
        self.current_cycle += 1;
    }

    pub fn set_cycle(&mut self, cycle: u64) {
        self.current_cycle = cycle;
    }

    pub fn get(&self, id: u64) -> Option<&EphemeralTask> {
        self.tasks.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut EphemeralTask> {
        self.tasks.get_mut(&id)
    }

    pub fn pending(&self) -> Vec<&EphemeralTask> {
        self.tasks.values().filter(|t| t.status == TaskStatus::Pending).collect()
    }

    pub fn by_priority(&self) -> Vec<&EphemeralTask> {
        let mut tasks: Vec<&EphemeralTask> = self.tasks.values()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect();
        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        tasks
    }

    pub fn energy_total_consumed(&self) -> u32 {
        self.tasks.values().map(|t| t.energy_consumed).sum()
    }

    pub fn completion_rate(&self) -> f64 {
        let total = self.tasks.len();
        if total == 0 {
            return 0.0;
        }
        let completed = self.tasks.values().filter(|t| t.status == TaskStatus::Completed).count();
        completed as f64 / total as f64
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}
