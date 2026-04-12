use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPurpose {
    Query,
    Analysis,
    Generation,
    Monitoring,
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Expired,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Running => write!(f, "Running"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Expired => write!(f, "Expired"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralTask {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub purpose: TaskPurpose,
    pub payload: Vec<u8>,
    pub energy_budget: u32,
    pub energy_consumed: u32,
    pub confidence: Option<f64>,
    pub status: TaskStatus,
    pub result: Option<Vec<u8>>,
    pub created_at: u64,
    pub expires_at: u64,
    pub ttl: u32,
    pub priority: u8,
    pub trust_required: f64,
}

impl EphemeralTask {
    pub fn new(
        id: u64,
        parent_id: Option<u64>,
        purpose: TaskPurpose,
        payload: Vec<u8>,
        energy_budget: u32,
        ttl: u32,
        priority: u8,
        trust_required: f64,
        current_cycle: u64,
    ) -> Self {
        Self {
            id,
            parent_id,
            purpose,
            payload,
            energy_budget,
            energy_consumed: 0,
            confidence: None,
            status: TaskStatus::Pending,
            result: None,
            created_at: current_cycle,
            expires_at: current_cycle + ttl as u64,
            ttl,
            priority: priority.min(9),
            trust_required,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnError {
    MaxConcurrent,
    EnergyExhausted,
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnError::MaxConcurrent => write!(f, "max concurrent tasks reached"),
            SpawnError::EnergyExhausted => write!(f, "energy pool exhausted"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    NotFound(u64),
    InvalidTransition { id: u64, from: TaskStatus, to: TaskStatus },
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::NotFound(id) => write!(f, "task {} not found", id),
            TaskError::InvalidTransition { id, from, to } => {
                write!(f, "task {} cannot transition from {} to {}", id, from, to)
            }
        }
    }
}
