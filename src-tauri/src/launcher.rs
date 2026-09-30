use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::inject::{build_command, InjectSpec};

pub const AMDAEMON_EXE: &str = "amdaemon.exe";
pub const GAME_EXE: &str = "chusanApp.exe";
pub const GAME_INJECTOR_X86: &str = "inject_x86.exe";
pub const BIN_DIR: &str = "bin";

const OPENSSL_IA32CAP: &str = ":~0x20000000";

pub fn resolve_bin_dir(game_root: &Path) -> PathBuf {
    let bin = game_root.join(BIN_DIR);
    if bin.is_dir() {
        bin
    } else {
        game_root.to_path_buf()
    }
}

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
        injector: GAME_INJECTOR_X86,
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

pub fn process_exists(image: &str) -> bool {
    let mut command = Command::new("tasklist");
    command.args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"]);
    stylize(&mut command);
    let Ok(output) = command.output() else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .to_lowercase()
        .contains(&image.to_lowercase())
}

pub fn spawn_amdaemon(
    bin_dir: &Path,
    on_line: impl Fn(&str) + Send + Sync + Clone + 'static,
) -> Result<(), String> {
    let mut built = build_command(bin_dir, &amdaemon_spec());
    spawn_logged(&mut built.command, "amdaemon", on_line)
        .map_err(|error| format!("启动 amdaemon 失败：{error}"))
}

pub fn spawn_game(
    bin_dir: &Path,
    dlls: Vec<String>,
    on_line: impl Fn(&str) + Send + Sync + Clone + 'static,
) -> Result<Vec<String>, String> {
    let mut built = build_command(bin_dir, &game_spec(dlls));
    spawn_logged(&mut built.command, "game", on_line)
        .map_err(|error| format!("启动游戏失败：{error}"))?;
    Ok(built.missing_dlls)
}

/// 只 spawn 一次进程并把 stdout/stderr 转发为日志；
/// 进程随启动器挂在作业对象上，启动器退出时由系统连带结束
fn spawn_logged(
    command: &mut Command,
    source: &'static str,
    on_line: impl Fn(&str) + Send + Sync + Clone + 'static,
) -> std::io::Result<()> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;

    let stdout = child.stdout.take().map(BufReader::new);
    let stderr = child.stderr.take().map(BufReader::new);
    if let Some(stream) = stdout {
        let on_line = on_line.clone();
        std::thread::spawn(move || forward_lines(stream, source, &on_line));
    }
    if let Some(stream) = stderr {
        std::thread::spawn(move || forward_lines(stream, source, &on_line));
    }
    Ok(())
}

fn forward_lines<R: Read>(stream: R, source: &str, on_line: &impl Fn(&str)) {
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let text = String::from_utf8_lossy(&buffer);
                let line = text.trim_end_matches(['\n', '\r']);
                if !line.is_empty() {
                    on_line(&format!("[{source}] {line}"));
                }
            }
        }
    }
}
