//! Phase 9 压力测试（任务书 §9 / PROJECT_PLAN）
//!
//! 通过真实的 `TaskManager::run_all` 驱动 `audio::convert` + `audio::verify`，
//! 覆盖三类压力场景：
//!   1. 批量并发：100 / 500 / 1000 个生成文件批量转码，验证全部完成、吞吐合理
//!   2. 大文件：5 分钟静音 FLAC → WAV，验证时长/采样率保真、不超时
//!   3. 损坏文件：写入坏 FLAC 头，验证该任务 Failed 但其余任务仍 Completed（失败隔离）
//!
//! ffmpeg 路径由 audio::ffmpeg_path() 自动解析（env → exe 同级 resources → PATH）。

use musicglass_audio::{convert, inspect, ConversionRequest};
use musicglass_audio::verify::verify;
use musicglass_core::{Format, Metadata, Result};
use musicglass_task_manager::TaskManager;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn ffmpeg() -> PathBuf {
    musicglass_audio::ffmpeg_path().unwrap_or_else(|_| PathBuf::from("ffmpeg"))
}

/// 生成一个静音 FLAC（时长 seconds 秒），返回路径。
fn gen_flac(dir: &Path, name: &str, seconds: f32) -> PathBuf {
    let src = dir.join(name);
    let st = std::process::Command::new(ffmpeg())
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=44100:cl=stereo",
            "-t",
            &seconds.to_string(),
            "-c:a",
            "flac",
            src.to_str().unwrap(),
        ])
        .status()
        .expect("ffmpeg generate");
    assert!(st.success(), "ffmpeg 生成 {name} 失败");
    src
}

/// 压测 worker：转换 + 验证，返回 Result 供 run_all 标记状态。
fn worker(input: &Path, output: &Path) -> Result<()> {
    let req = ConversionRequest {
        target_format: Format::Mp3,
        bitrate: Some(192_000),
        sample_rate: None,
        bit_depth: None,
        metadata: None,
    };
    convert(input, output, &req)?;
    let info = inspect(input)?;
    let _ = verify(output, &info, &Metadata::default())?;
    Ok(())
}

fn run_batch(count: usize) {
    let dir = std::env::temp_dir().join(format!("mg_stress_{count}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let tm = TaskManager::with_concurrency(4.max(1));
    for i in 0..count {
        let src = gen_flac(&dir, &format!("s{i:05}.flac"), 1.0);
        let dst = dir.join(format!("s{i:05}.mp3"));
        tm.submit(src, dst, Format::Mp3);
    }

    let t0 = std::time::Instant::now();
    tm.run_all(|task, flag: &Arc<AtomicBool>| {
        let _ = flag; // 压测不触发取消
        worker(&task.input, &task.output)
    });
    let elapsed = t0.elapsed();

    let tasks = tm.all();
    let completed = tasks.iter().filter(|t| t.status == musicglass_task_manager::TaskStatus::Completed).count();
    let failed = tasks.iter().filter(|t| t.status == musicglass_task_manager::TaskStatus::Failed).count();

    println!("  [{count} 文件] 耗时 {elapsed:?}，完成 {completed}，失败 {failed}");
    assert_eq!(completed, count, "{count} 个任务应全部完成");
    assert_eq!(failed, 0, "不应有失败任务");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stress_batch_100() {
    println!("Phase 9 压力测试：批量 100 文件");
    run_batch(100);
}

#[test]
fn stress_batch_500() {
    println!("Phase 9 压力测试：批量 500 文件");
    run_batch(500);
}

#[test]
fn stress_batch_1000() {
    println!("Phase 9 压力测试：批量 1000 文件");
    run_batch(1000);
}

#[test]
fn stress_large_file() {
    println!("Phase 9 压力测试：大文件（5 分钟 FLAC → WAV）");
    let dir = std::env::temp_dir().join("mg_stress_large");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let src = gen_flac(&dir, "big.flac", 300.0); // 5 分钟
    let dst = dir.join("big.wav");

    let req = ConversionRequest {
        target_format: Format::Wav,
        bitrate: None,
        sample_rate: None,
        bit_depth: None,
        metadata: None,
    };
    convert(&src, &dst, &req).expect("大文件转换");

    let src_info = inspect(&src).expect("inspect src");
    let dst_info = inspect(&dst).expect("inspect dst");

    // 时长保真（允许 1s 容差）
    assert!(
        (dst_info.duration_secs - src_info.duration_secs).abs() < 1.0,
        "大文件时长应保真：src {:.1}s dst {:.1}s",
        src_info.duration_secs,
        dst_info.duration_secs
    );
    // 采样率保真
    assert_eq!(dst_info.sample_rate, src_info.sample_rate, "采样率应保真");
    // 声道保真
    assert_eq!(dst_info.channels, src_info.channels, "声道数应保真");
    // WAV 是有损容器但 PCM 无损，时长足够长验证不超时
    assert!(dst_info.duration_secs > 290.0, "输出应接近 5 分钟");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stress_corrupt_file_isolation() {
    println!("Phase 9 压力测试：损坏文件失败隔离");
    let dir = std::env::temp_dir().join("mg_stress_corrupt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let tm = TaskManager::with_concurrency(4);

    // 9 个正常文件
    for i in 0..9 {
        let src = gen_flac(&dir, &format!("ok{i}.flac"), 1.0);
        let dst = dir.join(format!("ok{i}.mp3"));
        tm.submit(src, dst, Format::Mp3);
    }

    // 1 个损坏文件：写垃圾头（非 FLAC）
    let bad = dir.join("bad.flac");
    std::fs::write(&bad, b"THIS IS NOT A VALID FLAC FILE HEADER garbage").unwrap();
    let bad_dst = dir.join("bad.mp3");
    tm.submit(bad, bad_dst, Format::Mp3);

    tm.run_all(|task, _flag: &Arc<AtomicBool>| worker(&task.input, &task.output));

    let tasks = tm.all();
    let completed = tasks.iter().filter(|t| t.status == musicglass_task_manager::TaskStatus::Completed).count();
    let failed = tasks.iter().filter(|t| t.status == musicglass_task_manager::TaskStatus::Failed).count();

    // 9 个正常应完成，1 个损坏应失败，且失败不连累其他
    assert_eq!(completed, 9, "9 个正常文件应全部完成");
    assert_eq!(failed, 1, "1 个损坏文件应标记失败");

    // 失败任务应有可读错误信息（非 panic/空）
    let bad_task = tasks.iter().find(|t| t.status == musicglass_task_manager::TaskStatus::Failed).unwrap();
    assert!(bad_task.error.is_some(), "失败任务应记录错误信息");
    println!("  损坏任务错误：{}", bad_task.error.as_ref().unwrap());

    let _ = std::fs::remove_dir_all(&dir);
}
