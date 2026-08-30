//! Phase 10 稳定性：日志模块初始化与落盘验证。

use musicglass_core::logger;
use std::io::Read;

#[test]
fn logger_initializes_and_writes() {
    // 不应 panic，且重复调用安全。
    logger::init_logger();
    logger::init_logger();

    log::info!("Phase10 logger test info");
    log::warn!("Phase10 logger test warn");
    log::error!("Phase10 logger test error");

    // 日志文件应已创建并包含刚写入的内容。
    let path = if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("musicglass.log"))) {
        exe
    } else {
        std::path::PathBuf::from("musicglass.log")
    };
    let mut content = String::new();
    if let Ok(mut f) = std::fs::File::open(&path) {
        let _ = f.read_to_string(&mut content);
    }
    assert!(
        content.contains("Phase10 logger test") || content.contains("logger test"),
        "日志文件应含有测试写入内容：{path:?}"
    );
}
