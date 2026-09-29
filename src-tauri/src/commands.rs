use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{launcher, plugins};

#[derive(Default)]
pub struct LauncherState(pub Mutex<bool>);

#[derive(Serialize)]
pub struct LaunchReport {
    pub missing_dlls: Vec<String>,
}

/// 随启动器打包的内置插件：启动时由启动器复制到游戏目录
#[derive(Deserialize)]
pub struct BuiltinPluginSource {
    /// 注入时使用的 DLL 文件名
    pub dll: String,
    /// 资源目录中的 DLL 绝对路径
    pub source: String,
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    game_dir: String,
    dlls: Vec<String>,
    builtin_plugins: Option<Vec<BuiltinPluginSource>>,
) -> Result<LaunchReport, String> {
    {
        let state = app.state::<LauncherState>();
        let mut running = state.0.lock().map_err(|_| "内部状态异常".to_string())?;
        if *running {
            return Err("已有游戏会话正在运行".to_string());
        }
        *running = true;
    }

    let session_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        run_session(&session_app, &game_dir, dlls, builtin_plugins)
    })
    .await
    .map_err(|error| format!("内部任务异常：{error}"))?;

    if let Ok(mut running) = app.state::<LauncherState>().0.lock() {
        *running = false;
    }
    result
}

#[tauri::command]
pub fn is_running(state: State<'_, LauncherState>) -> bool {
    *state.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub fn stop_game() -> Result<(), String> {
    launcher::kill_by_image(launcher::GAME_EXE);
    launcher::kill_by_image(launcher::AMDAEMON_EXE);
    Ok(())
}

/// 列出插件目录中的 DLL 插件（含同名 JSON 清单信息）。
/// `directory` 缺省时扫描随启动器打包的内置插件目录。
#[tauri::command]
pub async fn list_plugin_dlls(
    app: AppHandle,
    directory: Option<String>,
) -> Result<Vec<plugins::PluginInfo>, String> {
    let directory = match directory {
        Some(directory) => PathBuf::from(directory),
        None => plugins::plugins_dir(&app).ok_or_else(|| "未找到内置插件目录".to_string())?,
    };
    tauri::async_runtime::spawn_blocking(move || {
        let paths = scan_plugin_dlls(&directory)?;
        Ok(paths
            .iter()
            .filter_map(|path| plugins::read_plugin_info(&directory, Path::new(path)))
            .collect())
    })
    .await
    .map_err(|error| format!("内部任务异常：{error}"))?
}

fn scan_plugin_dlls(directory: &Path) -> Result<Vec<String>, String> {
    let entries =
        std::fs::read_dir(directory).map_err(|error| format!("无法读取插件文件夹：{error}"))?;
    let mut dlls = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| format!("无法读取插件文件：{error}"))?
            .path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
        {
            dlls.push(path.to_string_lossy().into_owned());
        }
    }
    dlls.sort_by_cached_key(|path| path.to_lowercase());
    Ok(dlls)
}

fn run_session(
    app: &AppHandle,
    game_dir: &str,
    dlls: Vec<String>,
    builtin_plugins: Option<Vec<BuiltinPluginSource>>,
) -> Result<LaunchReport, String> {
    let game_dir = PathBuf::from(game_dir);
    if !game_dir.is_dir() {
        return Err(format!("游戏目录不存在：{}", game_dir.display()));
    }

    // 启用的内置插件：从资源目录复制到游戏目录（对应 bat 的 if exist duolinguo.dll 前置条件）
    if let Some(builtin_plugins) = builtin_plugins {
        for plugin in builtin_plugins {
            let target = game_dir.join(&plugin.dll);
            let source = PathBuf::from(&plugin.source);
            if source != target {
                std::fs::copy(&source, &target).map_err(|error| {
                    format!("无法安装内置插件 {}：{error}", plugin.dll)
                })?;
            }
        }
    }

    log(app, "正在清理残留的 amdaemon 进程…");
    launcher::kill_by_image(launcher::AMDAEMON_EXE);

    log(app, "正在启动 amdaemon…");
    let app_amdaemon = app.clone();
    launcher::spawn_amdaemon(&game_dir, move |line| {
        let _ = app_amdaemon.emit("launch://log", line.to_string());
    })?;

    log(app, "正在启动游戏…");
    let app_game = app.clone();
    let (mut game, missing_dlls) = launcher::spawn_game(&game_dir, dlls, move |line| {
        let _ = app_game.emit("launch://log", line.to_string());
    })?;

    log(app, "游戏已启动，等待退出…");
    let _ = app.emit("launch://state", true);
    let _ = game.wait();

    log(app, "正在清理 amdaemon…");
    launcher::kill_by_image(launcher::AMDAEMON_EXE);

    if !missing_dlls.is_empty() {
        log(
            app,
            &format!(
                "以下 DLL 文件不存在，已跳过：{}",
                missing_dlls.join(", ")
            ),
        );
    }
    log(app, "游戏进程已全部结束");
    let _ = app.emit("launch://state", false);

    Ok(LaunchReport { missing_dlls })
}

fn log(app: &AppHandle, message: &str) {
    let _ = app.emit("launch://log", message.to_string());
}

#[cfg(test)]
mod tests {
    use super::scan_plugin_dlls;

    #[test]
    fn scan_imports_only_dll_files_in_stable_order() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("launcher-plugins-{unique}"));
        std::fs::create_dir(&directory).unwrap();
        for name in ["z_plugin.dll", "a plugin.DLL", "notes.txt"] {
            std::fs::write(directory.join(name), b"").unwrap();
        }
        std::fs::create_dir(directory.join("nested.dll")).unwrap();
        std::fs::write(directory.join("nested.dll/ignored.dll"), b"").unwrap();

        let result = scan_plugin_dlls(&directory);
        std::fs::remove_dir_all(&directory).unwrap();
        assert_eq!(
            result.unwrap(),
            ["a plugin.DLL", "z_plugin.dll"]
                .map(|name| directory.join(name).to_string_lossy().into_owned())
        );
        assert!(scan_plugin_dlls(&directory).is_err());
    }
}
