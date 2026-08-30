//! Conversion history backed by SQLite (task book §38).
//!
//! Records every conversion attempt: input/output, formats, sizes, status,
//! timestamps, duration, and error. Pure logic in this crate; the Tauri layer
//! calls [`History::record`] after each task finishes.

use musicglass_core::{Format, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub input: String,
    pub output: String,
    pub input_format: String,
    pub output_format: String,
    pub input_size: u64,
    pub output_size: u64,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

pub struct History {
    conn: Connection,
}

impl History {
    /// Open (creating if needed) the history database at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| musicglass_core::AppError::Other(format!("history open: {e}")))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                input TEXT NOT NULL,
                output TEXT NOT NULL,
                input_format TEXT NOT NULL,
                output_format TEXT NOT NULL,
                input_size INTEGER NOT NULL DEFAULT 0,
                output_size INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                completed_at INTEGER,
                duration_ms INTEGER,
                error TEXT
            );",
        )
        .map_err(|e| musicglass_core::AppError::Other(format!("history init: {e}")))?;
        Ok(History { conn })
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    pub fn record(
        &self,
        input: &Path,
        output: &Path,
        input_format: Format,
        output_format: Format,
        status: &str,
        input_size: u64,
        output_size: u64,
        duration_ms: Option<u64>,
        error: Option<&str>,
    ) -> Result<()> {
        let completed = if status == "Completed" { Some(Self::now()) } else { None };
        self.conn
            .execute(
                "INSERT INTO history
                 (input, output, input_format, output_format, input_size, output_size,
                  status, created_at, completed_at, duration_ms, error)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    input.to_string_lossy(),
                    output.to_string_lossy(),
                    input_format.as_str(),
                    output_format.as_str(),
                    input_size,
                    output_size,
                    status,
                    Self::now(),
                    completed,
                    duration_ms,
                    error,
                ],
            )
            .map_err(|e| musicglass_core::AppError::Other(format!("history insert: {e}")))?;
        Ok(())
    }

    pub fn query_all(&self) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, input, output, input_format, output_format, input_size,
                        output_size, status, created_at, completed_at, duration_ms, error
                 FROM history ORDER BY id DESC",
            )
            .map_err(|e| musicglass_core::AppError::Other(format!("history query: {e}")))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(HistoryEntry {
                    id: r.get(0)?,
                    input: r.get(1)?,
                    output: r.get(2)?,
                    input_format: r.get(3)?,
                    output_format: r.get(4)?,
                    input_size: r.get(5)?,
                    output_size: r.get(6)?,
                    status: r.get(7)?,
                    created_at: r.get(8)?,
                    completed_at: r.get(9)?,
                    duration_ms: r.get(10)?,
                    error: r.get(11)?,
                })
            })
            .map_err(|e| musicglass_core::AppError::Other(format!("history map: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| musicglass_core::AppError::Other(format!("history row: {e}")))?);
        }
        Ok(out)
    }

    pub fn clear(&self) -> Result<()> {
        self.conn
            .execute("DELETE FROM history", [])
            .map_err(|e| musicglass_core::AppError::Other(format!("history clear: {e}")))?;
        Ok(())
    }

    /// 恢复机制（Phase 10）：列出上次未成功（Failed / Cancelled）的任务，
    /// 供 UI 启动时提示用户重试。
    pub fn query_failed(&self) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, input, output, input_format, output_format, input_size,
                        output_size, status, created_at, completed_at, duration_ms, error
                 FROM history WHERE status IN ('Failed','Cancelled') ORDER BY id DESC",
            )
            .map_err(|e| musicglass_core::AppError::Other(format!("history query_failed: {e}")))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(HistoryEntry {
                    id: r.get(0)?,
                    input: r.get(1)?,
                    output: r.get(2)?,
                    input_format: r.get(3)?,
                    output_format: r.get(4)?,
                    input_size: r.get(5)?,
                    output_size: r.get(6)?,
                    status: r.get(7)?,
                    created_at: r.get(8)?,
                    completed_at: r.get(9)?,
                    duration_ms: r.get(10)?,
                    error: r.get(11)?,
                })
            })
            .map_err(|e| musicglass_core::AppError::Other(format!("history map: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| musicglass_core::AppError::Other(format!("history row: {e}")))?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_query() {
        let dir = std::env::temp_dir().join("mg_hist_test.db");
        let _ = std::fs::remove_file(&dir);
        let h = History::open(&dir).unwrap();
        h.record(
            Path::new("a.flac"),
            Path::new("a.mp3"),
            Format::Flac,
            Format::Mp3,
            "Completed",
            1000,
            500,
            Some(1234),
            None,
        )
        .unwrap();
        h.record(
            Path::new("b.wav"),
            Path::new("b.flac"),
            Format::Wav,
            Format::Flac,
            "Failed",
            2000,
            0,
            None,
            Some("decode error"),
        )
        .unwrap();
        let all = h.query_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].status, "Failed");
        assert_eq!(all[0].error.as_deref(), Some("decode error"));
        assert_eq!(all[1].status, "Completed");
        h.clear().unwrap();
        assert_eq!(h.query_all().unwrap().len(), 0);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn query_failed_recovers_unfinished() {
        // Phase 10 恢复：Failed/Cancelled 任务能被 query_failed 列出。
        let dir = std::env::temp_dir().join("mg_hist_failed.db");
        let _ = std::fs::remove_file(&dir);
        let h = History::open(&dir).unwrap();
        h.record(Path::new("a.flac"), Path::new("a.mp3"), Format::Flac, Format::Mp3, "Completed", 1000, 500, Some(10), None).unwrap();
        h.record(Path::new("b.wav"), Path::new("b.flac"), Format::Wav, Format::Flac, "Failed", 2000, 0, None, Some("decode error")).unwrap();
        h.record(Path::new("c.ogg"), Path::new("c.mp3"), Format::Ogg, Format::Mp3, "Cancelled", 300, 0, None, None).unwrap();

        let failed = h.query_failed().unwrap();
        assert_eq!(failed.len(), 2, "应包含 1 Failed + 1 Cancelled");
        assert!(failed.iter().any(|e| e.status == "Failed" && e.error.as_deref() == Some("decode error")));
        assert!(failed.iter().any(|e| e.status == "Cancelled"));
        assert!(!failed.iter().any(|e| e.status == "Completed"));
        let _ = std::fs::remove_file(&dir);
    }
}
