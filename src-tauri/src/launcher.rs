use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};

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

/// 检查指定映像名的进程是否存在（注入器不等待游戏进程，须轮询游戏本体）
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

/// 启动 amdaemon（不等待其退出）；其输出经 `on_line` 逐行回调
pub fn spawn_amdaemon(
    game_dir: &Path,
    on_line: impl Fn(&str) + Send + Sync + Clone + 'static,
) -> Result<Child, String> {
    let mut built = build_command(game_dir, &amdaemon_spec());
    spawn_with_output(&mut built.command, "amdaemon", on_line);
    built
        .command
        .spawn()
        .map_err(|error| format!("启动 amdaemon 失败：{error}"))
}

/// 启动游戏（注入器进程会阻塞到游戏退出）；其输出经 `on_line` 逐行回调
pub fn spawn_game(
    game_dir: &Path,
    dlls: Vec<String>,
    on_line: impl Fn(&str) + Send + Sync + Clone + 'static,
) -> Result<(Child, Vec<String>), String> {
    let mut built = build_command(game_dir, &game_spec(dlls));
    spawn_with_output(&mut built.command, "game", on_line);
    let child = built
        .command
        .spawn()
        .map_err(|error| format!("启动游戏失败：{error}"))?;
    Ok((child, built.missing_dlls))
}

/// 管道化子进程 stdout/stderr，并各起一个转发线程
fn spawn_with_output(
    command: &mut Command,
    source: &'static str,
    on_line: impl Fn(&str) + Send + Sync + Clone + 'static,
) {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        // spawn 失败时无输出可转发，错误交由调用方处理
        Err(_) => return,
    };

    let stdout = child.stdout.take().map(BufReader::new);
    let stderr = child.stderr.take().map(BufReader::new);
    if let Some(stream) = stdout {
        let on_line = on_line.clone();
        std::thread::spawn(move || forward_lines(stream, source, &on_line));
    }
    if let Some(stream) = stderr {
        std::thread::spawn(move || forward_lines(stream, source, &on_line));
    }
}

/// 逐行读取流并回调；按字节切行后做有损 UTF-8 转换，兼容非 UTF-8 输出
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
