extern crate toml;

use std::collections::BTreeMap;
use std::io::{stderr, Write};
use std::path::PathBuf;
use std::env;

use toml::Value;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub path: String,
    pub shell: PathBuf,
    pub position: usize,
    pub reload: Option<u64>,
    pub is_static: bool,
}

pub fn get_command(section_name: &str, configuration: &toml::Table, position: usize) -> Command {
    Command {
        name: String::from(section_name),
        path: get_path(section_name, configuration),
        shell: get_shell(section_name, configuration),
        position: position,
        reload: get_reload(configuration),
        is_static: get_static(configuration),
    }
}

fn get_path(section_name: &str, configuration: &BTreeMap<String, toml::Value>) -> String {
    match configuration.get("path") {
        Some(value) => {
            let value = value.to_owned();
            match value {
                toml::Value::Array(_) => {
                    let _ = writeln!(stderr(), "Invalid path found for {}: arrays are deprecated - use a string instead\n", section_name);
                    panic!();
                },

                toml::Value::String(string) => {
                    string
                },

                _ => {
                    let _ = writeln!(stderr(), "Invalid path found for {}", section_name);
                    panic!();
                },
            }
        },
        None => {
            let _ = stderr().write(format!("No path found for {}\n", section_name).as_bytes());
            panic!();
        },
    }
}

fn get_reload(configuration: &BTreeMap<String, toml::Value>) -> Option<u64> {
    match configuration.get("reload") {
        Some(value) => {
            let value = value.to_owned();
            match value {
                toml::Value::Float(float) => {
                    Some((float * 1000f64) as u64)
                }
                toml::Value::Integer(int) => {
                    Some((int as f64 * 1000f64) as u64)
                },
                _ => None,
            }
        },
        None => None
    }
}

fn get_static(configuration: &BTreeMap<String, toml::Value>) -> bool {
    match configuration.get("static").and_then(Value::as_bool) {
        Some(value) => value,
        None => false,
    }
}

fn get_shell(section_name: &str, configuration: &BTreeMap<String, toml::Value>) -> PathBuf {
    match configuration.get("shell") {
        Some(value) => {
            let value = value.to_owned();
            match value {
                toml::Value::String(string) => {
                    PathBuf::from(string)
                },
                _ => {
                    let _ = stderr().write(format!("Invalid shell found for {}\n", section_name).as_bytes());
                    panic!()
                }
            }
        },
        None => {
            match env::var("SHELL").ok() {
                Some(sh) => {
                    PathBuf::from(sh)
                },
                None => {
                    let _ = stderr().write("Could not find your system's shell. Make sure the $SHELL variable is set.\n".as_bytes());
                    panic!()
                }
            }
        }
    }
}
