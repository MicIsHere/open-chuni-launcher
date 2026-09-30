use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::segatools;
use crate::{launcher, plugins};

#[derive(Default)]
pub struct LauncherState(pub Mutex<bool>);

#[derive(Serialize)]
pub struct LaunchReport {
    pub missing_dlls: Vec<String>,
}

#[derive(Deserialize)]
pub struct BuiltinPluginSource {
    pub dll: String,
    pub source: String,
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    game_dir: String,
    dlls: Vec<String>,
    builtin_plugins: Option<Vec<BuiltinPluginSource>>,
    segatools: Option<segatools::SegatoolsPatch>,
    launch_timeout_secs: Option<u64>,
) -> Result<LaunchReport, String> {
    if let Some(patch) = &segatools {
        if let Some(keychip) = &patch.keychip {
            if !segatools::is_valid_keychip(&keychip.id) {
                return Err("机台编号未填写或格式不正确，无法启动游戏".to_string());
            }
        }
    }
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
        run_session(
            &session_app,
            &game_dir,
            dlls,
            builtin_plugins,
            segatools,
            launch_timeout_secs.unwrap_or(15),
        )
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

#[tauri::command]
pub async fn list_plugin_dlls(
    app: AppHandle,
    directory: Option<String>,
) -> Result<Vec<plugins::PluginInfo>, String> {
    let directory = match directory {
        Some(directory) => PathBuf::from(directory),
        None => plugins::plugins_dir(&app).ok_or_else(|| "未找到内置插件目录".to_string())?,
    };
    tauri::async_runtime::spawn_blocking(move || Ok(plugins::scan_plugins(&directory)))
        .await
        .map_err(|error| format!("内部任务异常：{error}"))?
}

fn run_session(
    app: &AppHandle,
    game_root: &str,
    dlls: Vec<String>,
    builtin_plugins: Option<Vec<BuiltinPluginSource>>,
    segatools: Option<segatools::SegatoolsPatch>,
    launch_timeout_secs: u64,
) -> Result<LaunchReport, String> {
    let game_root = PathBuf::from(game_root);
    if !game_root.is_dir() {
        return Err(format!("游戏根目录不存在：{}", game_root.display()));
    }
    let bin_dir = launcher::resolve_bin_dir(&game_root);
    if !bin_dir.join(launcher::GAME_INJECTOR_X86).is_file() {
        return Err(format!(
            "游戏目录结构不正确：{} 中未找到 {}",
            bin_dir.display(),
            launcher::GAME_INJECTOR_X86
        ));
    }

    if let Some(builtin_plugins) = builtin_plugins {
        for plugin in builtin_plugins {
            let target = bin_dir.join(&plugin.dll);
            let source = PathBuf::from(&plugin.source);
            if source != target {
                std::fs::copy(&source, &target).map_err(|error| {
                    format!("无法安装内置插件 {}：{error}", plugin.dll)
                })?;
            }
        }
    }

    if let Some(patch) = &segatools {
        segatools::patch_ini(&bin_dir, &patch.sections())?;
        log(app, "已将配置写入 segatools.ini");
    }

    log(app, "正在清理残留的 amdaemon 进程…");
    launcher::kill_by_image(launcher::AMDAEMON_EXE);

    log(app, "正在启动 amdaemon…");
    let app_amdaemon = app.clone();
    launcher::spawn_amdaemon(&bin_dir, move |line| {
        let _ = app_amdaemon.emit("launch://log", line.to_string());
    })?;

    log(app, "正在启动游戏…");
    let app_game = app.clone();
    let (_injector, missing_dlls) = launcher::spawn_game(&bin_dir, dlls, move |line| {
        let _ = app_game.emit("launch://log", line.to_string());
    })?;

    let _ = app.emit("launch://state", true);

    log(app, "等待游戏进程出现…");
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(launch_timeout_secs.max(1));
    loop {
        if launcher::process_exists(launcher::GAME_EXE) {
            break;
        }
        if std::time::Instant::now() >= deadline {
            launcher::kill_by_image(launcher::GAME_EXE);
            launcher::kill_by_image(launcher::AMDAEMON_EXE);
            let _ = app.emit("launch://state", false);
            return Err("等待游戏进程出现超时，请检查游戏目录与注入配置".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    log(app, "游戏运行中，等待退出…");
    while launcher::process_exists(launcher::GAME_EXE) {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

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

