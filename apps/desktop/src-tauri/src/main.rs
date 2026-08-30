// MusicGlass 桌面壳入口 (Tauri 2)
// 把前端 IPC 调用桥接到 workspace 的 Rust crate（detector/audio/plugins/metadata/task-manager）
// 任务书 §7 插件架构：前端只通过 Plugin::find 统一分发，不写 if ncm/if qmc

use musicglass_audio::{ConversionRequest};
use musicglass_core::{logger, Format, Metadata};
use musicglass_detector::detect;
use musicglass_metadata::{read_metadata, write_metadata};
use musicglass_plugins::Plugin;
use musicglass_task_manager::History;
use serde::Serialize;
use std::path::PathBuf;

/// 转换验证结果（任务书铁律三：转完必须验证）
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReportDto {
    pub audio_ok: bool,
    pub metadata_ok: bool,
    pub cover_ok: bool,
    pub details: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResultDto {
    /// 转换后输出文件完整路径
    pub output_path: String,
    /// 验证报告（None 表示配置关闭验证）
    pub verification: Option<VerifyReportDto>,
}

#[derive(Serialize)]
pub struct DetectedFileDto {
    pub path: String,
    pub format: String,
    pub confidence: f32,
    pub is_proprietary: bool,
    pub supported: bool,
    pub duration_secs: Option<f64>,
    pub size_bytes: u64,
}

#[derive(Serialize)]
pub struct AudioInfoDto {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: Option<u16>,
    pub bitrate: Option<u64>,
    pub duration_secs: f64,
    pub lossless: bool,
    pub file_size: u64,
}

/// 检测单个文件格式（任务书 §10 格式识别）
#[tauri::command]
fn detect_file(path: String) -> Result<DetectedFileDto, String> {
    let p = PathBuf::from(&path);
    let det = detect(&p).map_err(|e| e.to_string())?;
    let mut dto = DetectedFileDto {
        path,
        format: format_name(det.format),
        confidence: det.confidence,
        is_proprietary: det.is_proprietary(),
        supported: det.is_supported_input(),
        duration_secs: None,
        size_bytes: p.metadata().map(|m| m.len()).unwrap_or(0),
    };
    // 专有格式或已支持格式都尝试探测时长（失败不影响检测）
    if let Ok(info) = musicglass_audio::inspect(&p) {
        dto.duration_secs = Some(info.duration_secs);
    }
    Ok(dto)
}

/// 检测多个文件（拖入批量）
#[tauri::command]
fn detect_files(paths: Vec<String>) -> Result<Vec<DetectedFileDto>, String> {
    paths.into_iter().map(detect_file).collect()
}

/// 探测音频信息（任务书 §6 Pipeline: Source Inspection）
#[tauri::command]
fn inspect_audio(path: String) -> Result<AudioInfoDto, String> {
    let info = musicglass_audio::inspect(&PathBuf::from(&path)).map_err(|e| e.to_string())?;
    Ok(AudioInfoDto {
        codec: info.codec,
        sample_rate: info.sample_rate,
        channels: info.channels,
        bit_depth: info.bit_depth,
        bitrate: info.bitrate,
        duration_secs: info.duration_secs,
        lossless: info.lossless,
        file_size: info.file_size,
    })
}

/// 解析专有格式容器，提取内部音频到临时文件，返回可处理的原始路径
/// 任务书 §7 / §8：NCM/QMC 经插件解密
#[tauri::command]
fn extract_proprietary(path: String, out_dir: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    let plugin = Plugin::find(&p).ok_or_else(|| format!("未找到可处理 '{}' 的插件", path))?;
    let stream = plugin
        .extract_audio(&p)
        .map_err(|e| format!("解密失败: {}", e))?;
    let ext = match stream.codec.as_str() {
        "flac" => "flac",
        "mp3" => "mp3",
        "aac" => "m4a",
        "alac" => "m4a",
        _ => "dat",
    };
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let out = PathBuf::from(&out_dir).join(format!("{}_extracted.{}", stem, ext));
    std::fs::write(&out, &stream.data).map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().into_owned())
}

/// 提取 Metadata（任务书 §20）
#[tauri::command]
fn extract_metadata(path: String) -> Result<Metadata, String> {
    let p = PathBuf::from(&path);
    // 优先走插件（专有格式内部 metadata）
    if let Some(plugin) = Plugin::find(&p) {
        if let Ok(m) = plugin.extract_metadata(&p) {
            return Ok(m);
        }
    }
    // 否则按普通格式读标签
    musicglass_metadata::read_metadata(&p).map_err(|e| e.to_string())
}

/// 执行一次转换（任务书 §6 / §26 验证在 musicglass_audio::convert 内未完成，这里仅调用转换）
#[tauri::command]
fn convert_audio(
    input: String,
    output: String,
    target_format: String,
    verify: Option<bool>,
) -> Result<ConvertResultDto, String> {
    let fmt = parse_format(&target_format).ok_or_else(|| format!("未知目标格式: {}", target_format))?;

    // 专有格式（NCM/QMC/MGG…）必须先解密提取内部音频，再交给 ffmpeg 转码
    let work_input = if let Some(plugin) = Plugin::find(&PathBuf::from(&input)) {
        let stream = plugin
            .extract_audio(&PathBuf::from(&input))
            .map_err(|e| format!("解密失败: {}", e))?;
        let ext = match stream.codec.as_str() {
            "flac" => "flac",
            "mp3" => "mp3",
            "aac" => "m4a",
            "alac" => "m4a",
            _ => "dat",
        };
        let tmp = std::env::temp_dir().join(format!("musicglass_{}.{}", std::process::id(), ext));
        std::fs::write(&tmp, &stream.data).map_err(|e| format!("解密临时写出失败: {}", e))?;
        tmp.to_string_lossy().into_owned()
    } else {
        input.clone()
    };

    let req = ConversionRequest {
        target_format: fmt,
        bitrate: None,
        sample_rate: None,
        bit_depth: None,
        metadata: None,
    };
    // 确保输出目录存在（否则 ffmpeg 写入失败）；目录创建失败要明确报错，避免假成功
    if let Some(parent) = PathBuf::from(&output).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("无法创建输出目录 {}: {}", parent.display(), e))?;
    }

    let convert_result = musicglass_audio::convert(&PathBuf::from(&work_input), &PathBuf::from(&output), &req);
    let status_str: &str;
    let err_msg: Option<String>;
    match &convert_result {
        Ok(()) => {
            status_str = "Completed";
            err_msg = None;
        }
        Err(e) => {
            status_str = "Failed";
            err_msg = Some(format!("转换失败: {}", e));
        }
    }
    // 无论成败都写入历史（Phase 10 恢复：失败任务可被 query_failed 找回）
    if let Some(hist) = open_history() {
        let src_fmt = detect(&PathBuf::from(&input)).map(|d| d.format).unwrap_or(Format::Unknown);
        let _ = hist.record(
            &PathBuf::from(&input),
            &PathBuf::from(&output),
            src_fmt,
            fmt,
            status_str,
            PathBuf::from(&input).metadata().map(|m| m.len()).unwrap_or(0),
            PathBuf::from(&output).metadata().map(|m| m.len()).unwrap_or(0),
            None,
            err_msg.as_deref(),
        );
    }
    convert_result.map_err(|e| format!("转换失败: {}", e))?;

    // 铁律二：Metadata 尽可能完整保留——读源标签并嵌入输出文件
    let src_meta = if let Some(plugin) = Plugin::find(&PathBuf::from(&input)) {
        plugin
            .extract_metadata(&PathBuf::from(&input))
            .unwrap_or_default()
    } else {
        read_metadata(&PathBuf::from(&input)).unwrap_or_default()
    };
    if src_meta.title.is_some() || src_meta.artist.is_some() || src_meta.album.is_some() {
        if let Err(e) = write_metadata(&PathBuf::from(&output), &src_meta, fmt) {
            // 元数据写入失败不应让整条任务失败（音频已转好），记录到详情但不阻断
            eprintln!("metadata 嵌入警告: {}", e);
        }
    }

    // 铁律三：转换完成必须验证（重新读取输出核对音频参数 + 元数据）
    let verification = if verify.unwrap_or(true) {
        // 以源（或解密后临时文件）的音频参数作为期望值基准
        let expected_info = musicglass_audio::inspect(&PathBuf::from(&work_input))
            .or_else(|_| musicglass_audio::inspect(&PathBuf::from(&input)));
        match expected_info {
            Ok(info) => match musicglass_audio::verify::verify(
                &PathBuf::from(&output),
                &info,
                &src_meta,
            ) {
                Ok(report) => Some(VerifyReportDto {
                    audio_ok: report.audio_ok,
                    metadata_ok: report.metadata_ok,
                    cover_ok: report.cover_ok,
                    details: report.details,
                }),
                Err(e) => Some(VerifyReportDto {
                    audio_ok: false,
                    metadata_ok: false,
                    cover_ok: false,
                    details: vec![format!("验证失败: {}", e)],
                }),
            },
            Err(e) => Some(VerifyReportDto {
                audio_ok: false,
                metadata_ok: false,
                cover_ok: false,
                details: vec![format!("无法获取源音频参数，跳过验证: {}", e)],
            }),
        }
    } else {
        None
    };

    Ok(ConvertResultDto {
        output_path: output.clone(),
        verification,
    })
}

/// 在文件资源管理器中定位文件（Windows 打开所在文件夹；macOS open -R）
#[tauri::command]
fn reveal_file(path: String) -> Result<(), String> {
    // 统一分隔符为反斜杠
    let norm = path.replace('/', "\\");
    // 取文件所在目录
    let folder = std::path::Path::new(&norm)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| norm.clone());
    #[cfg(windows)]
    {
        // 直接打开文件所在文件夹（避免 OneDrive 重解析点下 /select 跳转到文档库的怪癖）
        std::process::Command::new("explorer")
            .arg(&folder)
            .spawn()
            .map_err(|e| format!("无法打开资源管理器: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&norm)
            .spawn()
            .map_err(|e| format!("无法打开文件管理器: {}", e))?;
    }
    Ok(())
}
#[tauri::command]
fn doctor() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "ffmpeg_available": musicglass_audio::ffmpeg_available(),
    }))
}

/// 历史数据库路径：exe 同级目录下的 `musicglass_history.db`。
fn history_path() -> PathBuf {
    if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("musicglass_history.db"))) {
        exe
    } else {
        PathBuf::from("musicglass_history.db")
    }
}

/// 打开历史库（失败返回 None，不影响转换主流程）。
fn open_history() -> Option<History> {
    History::open(&history_path()).ok()
}

/// Phase 10 恢复：返回上次未成功（Failed/Cancelled）的转换任务，供前端提示重试。
#[tauri::command]
fn get_failed_history() -> Result<Vec<serde_json::Value>, String> {
    let hist = open_history().ok_or_else(|| "无法打开历史库".to_string())?;
    let entries = hist.query_failed().map_err(|e| e.to_string())?;
    let out = entries
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "input": e.input,
                "output": e.output,
                "inputFormat": e.input_format,
                "outputFormat": e.output_format,
                "status": e.status,
                "error": e.error,
                "createdAt": e.created_at,
            })
        })
        .collect();
    Ok(out)
}

fn format_name(f: Format) -> String {
    f.as_str().to_string()
}

fn parse_format(s: &str) -> Option<Format> {
    match s.to_lowercase().as_str() {
        "mp3" => Some(Format::Mp3),
        "flac" => Some(Format::Flac),
        "wav" => Some(Format::Wav),
        "m4a" | "aac" => Some(Format::M4a),
        "ogg" => Some(Format::Ogg),
        "opus" => Some(Format::Opus),
        _ => None,
    }
}

fn main() {
    // Phase 10 稳定性：初始化日志落盘 + 全局 panic hook（崩溃写入日志而非静默丢失）
    logger::init_logger();
    log::info!("MusicGlass 桌面壳启动");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            detect_file,
            detect_files,
            inspect_audio,
            extract_proprietary,
            extract_metadata,
            convert_audio,
            doctor,
            reveal_file,
            get_failed_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MusicGlass");
}
