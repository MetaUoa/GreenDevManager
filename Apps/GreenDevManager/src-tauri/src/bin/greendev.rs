use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn root_from(start: &Path) -> Option<PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join(r"Config\greendev\components.json").is_file() {
            return Some(candidate.to_path_buf());
        }
    }
    None
}
fn main() -> ExitCode {
    let root = env::var_os("FRAMEWORKS_HOME")
        .map(PathBuf::from)
        .filter(|path| path.join(r"Config\greendev\components.json").is_file())
        .or_else(|| {
            env::current_exe()
                .ok()
                .and_then(|path| root_from(&path).or_else(|| path.parent().and_then(root_from)))
        })
        .or_else(|| env::current_dir().ok().and_then(|path| root_from(&path)));
    let Some(root) = root else {
        eprintln!("Frameworks root was not found.");
        return ExitCode::from(2);
    };
    let script = root.join(r"Scripts\greendev-cli.ps1");
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(script)
        .args(env::args_os().skip(1))
        .current_dir(&root)
        .status();
    match status {
        Ok(value) => ExitCode::from(value.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!("CLI launch failed: {error}");
            ExitCode::from(1)
        }
    }
}
