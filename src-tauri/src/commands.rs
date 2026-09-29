use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::launcher;

#[derive(Default)]
pub struct LauncherState(pub Mutex<bool>);

#[derive(Serialize)]
pub struct LaunchReport {
    pub missing_dlls: Vec<String>,
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    game_dir: String,
    dlls: Vec<String>,
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
        run_session(&session_app, &game_dir, dlls)
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

fn run_session(app: &AppHandle, game_dir: &str, dlls: Vec<String>) -> Result<LaunchReport, String> {
    let game_dir = PathBuf::from(game_dir);
    if !game_dir.is_dir() {
        return Err(format!("游戏目录不存在：{}", game_dir.display()));
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
