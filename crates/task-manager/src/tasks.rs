//! Conversion task model, queue, and concurrency (task book §12, §13, §14).
//!
//! A [`TaskManager`] owns a set of [`Task`]s. Tasks run on a bounded worker
//! pool (default `logical_cpus / 2`). One task failing must never take down
//! the others (§13). Cancellation is cooperative via shared atomic flags.

use musicglass_core::{Format, Metadata, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Lifecycle state of a single conversion task (task book §12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

impl TaskStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::Skipped
        )
    }
}

/// A single file conversion job.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub input: PathBuf,
    pub output: PathBuf,
    pub target_format: Format,
    pub metadata: Option<Metadata>,
    pub status: TaskStatus,
    pub error: Option<String>,
    /// Set while Processing; the worker checks it to abort early.
    pub cancel_flag: Option<Arc<AtomicBool>>,
}

impl Task {
    pub fn new(id: u64, input: PathBuf, output: PathBuf, target_format: Format) -> Self {
        Task {
            id,
            input,
            output,
            target_format,
            metadata: None,
            status: TaskStatus::Pending,
            error: None,
            cancel_flag: None,
        }
    }
}

/// Shared, thread-safe task store + worker pool.
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<u64, Task>>>,
    next_id: Arc<Mutex<u64>>,
    concurrency: usize,
}

impl TaskManager {
    pub fn new() -> Self {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self::with_concurrency(cpus / 2.max(1))
    }

    pub fn with_concurrency(concurrency: usize) -> Self {
        TaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            concurrency: concurrency.max(1),
        }
    }

    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Submit a task, returning its assigned id.
    pub fn submit(&self, input: PathBuf, output: PathBuf, target: Format) -> u64 {
        let mut id_guard = self.next_id.lock().unwrap();
        let id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(id, Task::new(id, input, output, target));
        id
    }

    pub fn get(&self, id: u64) -> Option<Task> {
        self.tasks.lock().unwrap().get(&id).cloned()
    }

    pub fn all(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }

    /// Request cancellation of a (non-terminal) task.
    pub fn cancel(&self, id: u64) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(t) = tasks.get_mut(&id) {
            if t.status.is_terminal() {
                return Err(musicglass_core::AppError::Other("task already finished".into()));
            }
            if let Some(flag) = &t.cancel_flag {
                flag.store(true, Ordering::SeqCst);
            }
            t.status = TaskStatus::Cancelled;
            Ok(())
        } else {
            Err(musicglass_core::AppError::Other("task not found".into()))
        }
    }

    /// Re-queue a failed/cancelled task for another attempt.
    pub fn retry(&self, id: u64) -> Result<u64> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(t) = tasks.get_mut(&id) {
            if !matches!(t.status, TaskStatus::Failed | TaskStatus::Cancelled) {
                return Err(musicglass_core::AppError::Other(
                    "only failed/cancelled tasks can be retried".into(),
                ));
            }
            let new_id = {
                let mut n = self.next_id.lock().unwrap();
                let v = *n;
                *n += 1;
                v
            };
            let mut nt = Task::new(new_id, t.input.clone(), t.output.clone(), t.target_format);
            nt.metadata = t.metadata.clone();
            tasks.insert(new_id, nt);
            Ok(new_id)
        } else {
            Err(musicglass_core::AppError::Other("task not found".into()))
        }
    }

    /// Run all `Pending` tasks to completion across the worker pool.
    ///
    /// `worker` is the per-task conversion closure. It receives the task and a
    /// cancel flag; returning `Err` marks the task `Failed` (others continue).
    pub fn run_all<F>(&self, worker: F)
    where
        F: Fn(&Task, &Arc<AtomicBool>) -> Result<()> + Send + Sync + 'static,
    {
        let worker = Arc::new(worker);
        let pending: Vec<u64> = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .values()
                .filter(|t| t.status == TaskStatus::Pending)
                .map(|t| t.id)
                .collect()
        };

        for chunk in pending.chunks(self.concurrency.max(1)) {
            let mut local: Vec<std::thread::JoinHandle<()>> = Vec::new();
            for &id in chunk {
                let tasks = Arc::clone(&self.tasks);
                let worker = Arc::clone(&worker);
                let flag = Arc::new(AtomicBool::new(false));
                {
                    let mut g = tasks.lock().unwrap();
                    if let Some(t) = g.get_mut(&id) {
                        t.status = TaskStatus::Processing;
                        t.cancel_flag = Some(Arc::clone(&flag));
                    }
                }
                let h = std::thread::spawn(move || {
                    let task_snapshot = { tasks.lock().unwrap().get(&id).cloned() };
                    if let Some(task) = task_snapshot {
                        let res = worker(&task, &flag);
                        let mut g = tasks.lock().unwrap();
                        if let Some(t) = g.get_mut(&id) {
                            // Don't override an explicit cancel that landed mid-run.
                            if t.status == TaskStatus::Processing {
                                match res {
                                    Ok(()) => t.status = TaskStatus::Completed,
                                    Err(e) => {
                                        t.status = TaskStatus::Failed;
                                        t.error = Some(e.to_string());
                                    }
                                }
                            }
                        }
                    }
                });
                local.push(h);
            }
            for h in local {
                let _ = h.join();
            }
        }
    }

    /// Count of tasks in each status.
    pub fn counts(&self) -> HashMap<TaskStatus, usize> {
        let tasks = self.tasks.lock().unwrap();
        let mut m = HashMap::new();
        for t in tasks.values() {
            *m.entry(t.status).or_insert(0) += 1;
        }
        m
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_and_retry() {
        let tm = TaskManager::with_concurrency(1);
        let id = tm.submit(PathBuf::from("a.flac"), PathBuf::from("a.mp3"), Format::Mp3);
        assert_eq!(tm.get(id).unwrap().status, TaskStatus::Pending);
        let nid = tm.retry(id).unwrap_err(); // pending can't retry
        let _ = nid;
        let _ = tm;
    }

    #[test]
    fn failure_is_isolated() {
        let tm = TaskManager::with_concurrency(2);
        let ok_id = tm.submit(PathBuf::from("good"), PathBuf::from("out1"), Format::Flac);
        let bad_id = tm.submit(PathBuf::from("bad"), PathBuf::from("out2"), Format::Flac);

        tm.run_all(|task, _flag| {
            if task.input.to_string_lossy().contains("bad") {
                Err(musicglass_core::AppError::Other("boom".into()))
            } else {
                Ok(())
            }
        });

        assert_eq!(tm.get(ok_id).unwrap().status, TaskStatus::Completed);
        assert_eq!(tm.get(bad_id).unwrap().status, TaskStatus::Failed);
        assert_eq!(tm.get(bad_id).unwrap().error.as_deref(), Some("boom"));
    }

    #[test]
    fn cancel_before_run() {
        let tm = TaskManager::with_concurrency(1);
        let id = tm.submit(PathBuf::from("x"), PathBuf::from("y"), Format::Mp3);
        tm.cancel(id).unwrap();
        assert_eq!(tm.get(id).unwrap().status, TaskStatus::Cancelled);
    }
}
