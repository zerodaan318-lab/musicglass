//! Task queue, concurrency, and history (task book §12, §13, §38).
//!
//! Phase 4 will implement the queue, pause/cancel/retry, and SQLite history.
//! This crate defines the task state machine types now.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub input: String,
    pub output: String,
    pub status: TaskStatus,
}
