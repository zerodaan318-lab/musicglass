//! 统一日志与崩溃防护（Phase 10 稳定性：错误日志 / 防崩溃）
//!
//! - `init_logger()`：把 `log` facade 接到一个轻量文件 logger，日志写入
//!   `musicglass.log`（位于 exe 同级目录，或当前目录兜底）。
//! - `install_panic_hook()`：捕获任何线程的 panic，写入日志文件而不是只打印到
//!   stderr；worker 线程 panic 时由调用方 `catch_unwind` 转成任务失败，不会带走主进程。
//!
//! 故意不引入 `env_logger` / `fern` 等重依赖，保持 Tauri 内嵌体积可控。

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// 计算日志文件路径：优先 exe 同级目录，失败则当前目录。
fn log_path() -> PathBuf {
    if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("musicglass.log"))) {
        if exe.parent().map(|p| p.exists()).unwrap_or(false) {
            return exe;
        }
    }
    PathBuf::from("musicglass.log")
}

struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl FileLogger {
    fn new() -> Option<Self> {
        let path = log_path();
        let file = OpenOptions::new().create(true).append(true).open(&path).ok()?;
        Some(Self { file: Mutex::new(file) })
    }

    fn line(&self, level: Level, target: &str, args: &std::fmt::Arguments) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(
            self.file.lock().unwrap(),
            "[{}] [{:<5}] [{}] {}",
            now,
            level.as_str(),
            target,
            args
        );
        // 同时镜像到 stderr，方便开发期看（release 下 stderr 仍可重定向）
        eprintln!("[{}] [{}] {}", level.as_str(), target, args);
    }
}

impl Log for FileLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.line(record.level(), record.target(), record.args());
    }

    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}

static LOGGER: std::sync::OnceLock<FileLogger> = std::sync::OnceLock::new();

/// 初始化日志。重复调用安全（只生效一次）。
pub fn init_logger() {
    let _ = chrono::Local::now(); // 确保 chrono 可用
    if let Some(logger) = FileLogger::new() {
        let _ = LOGGER.set(logger);
        if log::set_logger(LOGGER.get().unwrap()).is_ok() {
            log::set_max_level(LevelFilter::Info);
        }
    }
    install_panic_hook();
}

/// 安装全局 panic hook：把 panic 写入日志文件，避免只丢失在 stderr。
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Box<dyn Any>".into());
        log::error!("PANIC @ {loc}: {msg}");
        eprintln!("PANIC @ {loc}: {msg}");
    }));
}
