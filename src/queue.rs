use crate::task::EphemeralTask;

/// Simple priority queue backed by a sorted Vec. No heap dependency.
pub struct TaskQueue {
    tasks: Vec<EphemeralTask>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Insert task at the correct position by priority (higher first).
    pub fn push(&mut self, task: EphemeralTask) {
        let pos = self.tasks.iter().position(|t| task.priority > t.priority);
        match pos {
            Some(i) => self.tasks.insert(i, task),
            None => self.tasks.push(task),
        }
    }

    /// Remove and return the highest-priority task.
    pub fn pop(&mut self) -> Option<EphemeralTask> {
        if self.tasks.is_empty() {
            None
        } else {
            Some(self.tasks.remove(0))
        }
    }

    /// Remove up to `n` highest-priority tasks.
    pub fn drain(&mut self, n: usize) -> Vec<EphemeralTask> {
        let take = n.min(self.tasks.len());
        self.tasks.drain(..take).collect()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}
