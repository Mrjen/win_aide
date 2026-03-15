use dioxus::prelude::*;
use futures_util::StreamExt;
use serde::Deserialize;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

/// GitHub Release API 响应
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// 更新状态机
#[derive(Debug, Clone, PartialEq, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    Available(UpdateInfo),
    Downloading { progress: f64 },
    Ready,
    Error(String),
}

/// UI 显示的更新信息
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub name: String,
    pub body: String,
    pub download_url: String,
    pub size: u64,
}

const GITHUB_API_URL: &str = "https://api.github.com/repos/Mrjen/win_aide/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

// ---------------------------------------------------------------------------
// 版本检查
// ---------------------------------------------------------------------------

pub async fn check_update(state: &mut Signal<UpdateState>) {
    state.set(UpdateState::Checking);
    match fetch_latest_release().await {
        Ok(Some(info)) => state.set(UpdateState::Available(info)),
        Ok(None) => state.set(UpdateState::Idle),
        Err(e) => {
            eprintln!("检查更新失败: {e}");
            state.set(UpdateState::Idle);
        }
    }
}

async fn fetch_latest_release() -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::Client::builder()
        .user_agent("win_aide-updater")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let release: ReleaseInfo = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?
        .json()
        .await
        .map_err(|e| format!("解析 JSON 失败: {e}"))?;

    let remote_ver_str = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let remote_ver =
        semver::Version::parse(remote_ver_str).map_err(|e| format!("解析远程版本号失败: {e}"))?;
    let current_ver =
        semver::Version::parse(CURRENT_VERSION).map_err(|e| format!("解析当前版本号失败: {e}"))?;

    if remote_ver <= current_ver {
        return Ok(None);
    }

    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".exe"))
        .ok_or_else(|| "Release 中未找到 .exe 文件".to_string())?;

    Ok(Some(UpdateInfo {
        version: remote_ver_str.to_string(),
        name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        body: release.body.unwrap_or_default(),
        download_url: asset.browser_download_url.clone(),
        size: asset.size,
    }))
}

// ---------------------------------------------------------------------------
// 下载更新
// ---------------------------------------------------------------------------

pub async fn download_update(
    state: &mut Signal<UpdateState>,
    download_url: &str,
    expected_size: u64,
) -> Result<(), String> {
    state.set(UpdateState::Downloading { progress: 0.0 });

    let client = reqwest::Client::builder()
        .user_agent("win_aide-updater")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {e}"))?;

    let total_size = response.content_length().unwrap_or(expected_size);
    let temp_path = get_temp_exe_path();

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("创建临时文件失败: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载数据失败: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入文件失败: {e}"))?;
        downloaded += chunk.len() as u64;
        let progress = (downloaded as f64 / total_size as f64).min(1.0);
        state.set(UpdateState::Downloading { progress });
    }

    file.flush()
        .await
        .map_err(|e| format!("刷新文件失败: {e}"))?;
    drop(file);

    // 校验文件大小
    let metadata = tokio::fs::metadata(&temp_path)
        .await
        .map_err(|e| format!("读取文件信息失败: {e}"))?;
    if metadata.len() != expected_size {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "文件大小不匹配: 期望 {} 字节，实际 {} 字节",
            expected_size,
            metadata.len()
        ));
    }

    state.set(UpdateState::Ready);
    Ok(())
}

fn get_temp_exe_path() -> PathBuf {
    std::env::temp_dir().join("win_aide_update.exe")
}

// ---------------------------------------------------------------------------
// 自替换（通过 bat 脚本实现）
// ---------------------------------------------------------------------------

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn apply_update() -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
    let current_exe_str = current_exe.to_string_lossy();
    let temp_exe = get_temp_exe_path();
    let temp_exe_str = temp_exe.to_string_lossy();

    let bat_content = format!(
        "@echo off\r\n\
         timeout /t 2 /nobreak >nul\r\n\
         copy /y \"{}\" \"{}\"\r\n\
         start \"\" \"{}\"\r\n\
         del \"%~f0\"\r\n",
        temp_exe_str, current_exe_str, current_exe_str
    );

    let bat_path = std::env::temp_dir().join("win_aide_updater.bat");
    std::fs::write(&bat_path, &bat_content).map_err(|e| format!("写入更新脚本失败: {e}"))?;

    std::process::Command::new("cmd")
        .args(["/C", &bat_path.to_string_lossy()])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("启动更新脚本失败: {e}"))?;

    std::process::exit(0);
}
