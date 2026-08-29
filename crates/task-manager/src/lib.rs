//! Task queue, concurrency, and history (task book §12, §13, §38).
//!
//! - `tasks` — task state machine, bounded concurrent queue, cancel/retry
//! - `history` — SQLite-backed conversion history

pub mod history;
pub mod tasks;

pub use tasks::{Task, TaskManager, TaskStatus};
