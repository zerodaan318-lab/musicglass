// MusicGlass 桌面壳入口 (Tauri 2)
// 把前端 IPC 调用桥接到 workspace 的 Rust crate（detector/audio/plugins/metadata/task-manager）
// 任务书 §7 插件架构：前端只通过 Plugin::find 统一分发，不写 if ncm/if qmc

use musicglass_audio::{ConversionRequest};
use musicglass_core::{Format, Metadata};
use musicglass_detector::detect;
use musicglass_plugins::Plugin;
use serde::Serialize;
use std::path::PathBuf;

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
) -> Result<(), String> {
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

    musicglass_audio::convert(&PathBuf::from(&work_input), &PathBuf::from(&output), &req)
        .map_err(|e| format!("转换失败: {}", e))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running MusicGlass");
}
