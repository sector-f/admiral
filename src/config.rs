use std::path::PathBuf;
use std::env;

pub fn get_config_file() -> Option<PathBuf> {
    let xdg_path = env::var("XDG_CONFIG_HOME").ok()
        .map(|v| PathBuf::from(v).join("admiral.d").join("admiral.toml"))
        .and_then(if_readable);

    let dot_home = env::var("HOME").ok()
        .map(|v| PathBuf::from(v).join(".config").join("admiral.d").join("admiral.toml"))
        .and_then(if_readable);

    xdg_path.or(dot_home)
}

fn if_readable(path: PathBuf) -> Option<PathBuf> { if path.exists() { Some(path) } else { None } }
