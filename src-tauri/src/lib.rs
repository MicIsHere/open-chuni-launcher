mod commands;
mod inject;
mod launcher;
mod plugins;
mod process_guard;
mod segatools;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    process_guard::setup();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::LauncherState::default())
        .invoke_handler(tauri::generate_handler![
            commands::launch_game,
            commands::stop_game,
            commands::is_running,
            commands::list_plugin_dlls
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
