use std::path::Path;
use std::process::{Child, Command};

use crate::inject::{build_command, InjectSpec};

pub const AMDAEMON_EXE: &str = "amdaemon.exe";
pub const GAME_EXE: &str = "chusanApp.exe";

const OPENSSL_IA32CAP: &str = ":~0x20000000";

fn amdaemon_spec() -> InjectSpec {
    InjectSpec {
        injector: "inject_x64.exe",
        exe: AMDAEMON_EXE.to_string(),
        dlls: vec!["chusanamhook.dll".to_string()],
        target_args: [
            "-f",
            "-c",
            "config_common.json",
            "config_server.json",
            "config_client.json",
            "config_cvt.json",
            "config_sp.json",
            "config_hook.json",
        ]
        .map(String::from)
        .to_vec(),
    }
}

fn game_spec(dlls: Vec<String>) -> InjectSpec {
    InjectSpec {
        injector: "inject_x86.exe",
        exe: GAME_EXE.to_string(),
        dlls,
        target_args: vec![],
    }
}

fn stylize(command: &mut Command) {
    command.env("OPENSSL_ia32cap", OPENSSL_IA32CAP);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub fn kill_by_image(image: &str) {
    let mut command = Command::new("taskkill");
    command.args(["/F", "/IM", image]);
    stylize(&mut command);
    let _ = command.output();
}

pub fn spawn_amdaemon(game_dir: &Path) -> Result<Child, String> {
    let mut built = build_command(game_dir, &amdaemon_spec());
    stylize(&mut built.command);
    built
        .command
        .spawn()
        .map_err(|error| format!("启动 amdaemon 失败：{error}"))
}

pub fn spawn_game(game_dir: &Path, dlls: Vec<String>) -> Result<(Child, Vec<String>), String> {
    let mut built = build_command(game_dir, &game_spec(dlls));
    stylize(&mut built.command);
    let child = built
        .command
        .spawn()
        .map_err(|error| format!("启动游戏失败：{error}"))?;
    Ok((child, built.missing_dlls))
}
