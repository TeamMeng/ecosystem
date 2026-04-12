use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub payload: String,
    pub status: TaskStatus,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed(String),
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub payload: String,
}
