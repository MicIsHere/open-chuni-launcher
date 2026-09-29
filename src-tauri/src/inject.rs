use std::path::Path;
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

pub fn build_command(game_dir: &Path, spec: &InjectSpec) -> BuiltInjection {
    let mut missing_dlls = Vec::new();
    let mut present_dlls = Vec::new();
    for dll in &spec.dlls {
        if game_dir.join(dll).is_file() {
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
