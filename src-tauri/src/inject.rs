use std::path::{Path, PathBuf};
use std::process::Command;

pub struct InjectSpec {
    pub injector: &'static str,
    pub exe: String,
    pub dlls: Vec<String>,
    pub target_args: Vec<String>,
}

pub struct BuiltInjection {
    pub command: Command,
    pub missing_dlls: Vec<String>,
}

/// 解析 DLL 条目：绝对路径直接使用（内置插件位于资源目录），
/// 相对路径基于游戏目录。
fn resolve_dll(game_dir: &Path, dll: &str) -> PathBuf {
    let path = Path::new(dll);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        game_dir.join(path)
    }
}

pub fn build_command(game_dir: &Path, spec: &InjectSpec) -> BuiltInjection {
    let mut missing_dlls = Vec::new();
    let mut present_dlls = Vec::new();
    for dll in &spec.dlls {
        if resolve_dll(game_dir, dll).is_file() {
            present_dlls.push(dll.clone());
        } else {
            missing_dlls.push(dll.clone());
        }
    }

    let mut command = Command::new(game_dir.join(spec.injector));
    command
        .current_dir(game_dir)
        .arg("-d");
    for dll in &present_dlls {
        command.arg("-k").arg(dll);
    }
    command.arg(&spec.exe).args(&spec.target_args);

    BuiltInjection {
        command,
        missing_dlls,
    }
}
