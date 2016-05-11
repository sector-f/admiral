extern crate toml;

use std::process::{Command, exit};
use std::io::{stdout, Write, Read};
use std::sync::mpsc::{channel, Sender};
use std::fs::File;
use std::path::PathBuf;
use std::thread::{self, sleep};
use std::time::Duration;

mod configuration;

#[derive(Debug)]
struct BarItem {
    path: PathBuf,
    duration: Option<u64>,
}

fn execute_script(dir: PathBuf, script: BarItem, _: Sender<Vec<u8>>,) {
    let path = if script.path.is_relative() {
        dir.join(script.path)
    } else {
        script.path
    };
    match script.duration {
        Some(time) => {
            loop {
                let output = Command::new(&path).spawn().unwrap();
                // let _ = sender.send(output.stdout);
                sleep(Duration::from_secs(time));
            }
        },
        None => {
            let output = Command::new(&path).spawn().unwrap();
        },
    }
}

fn main() {
    let mut buffer = String::new();
    let config_directory = match configuration::get_config_directory() {
        Some(file) => file,
        None => {
            println!("Configuration directory not found");
            exit(1);
        },
    };

    if let Ok(mut file) = File::open(config_directory.join("admiral.toml")) {
        file.read_to_string(&mut buffer).expect("Could not read configuration file");
    }

    let config_toml = toml::Parser::new(&buffer).parse().unwrap();

    let (sender, receiver) = channel::<Vec<u8>>();

    for (_key, value) in config_toml {
        let script = BarItem {
            path: PathBuf::from(value.as_table().unwrap().get("path").unwrap().as_str().unwrap()),
            duration: value.as_table().unwrap().get("reload").and_then(|x| x.as_integer()).map(|x| x as u64),
        };

        // Annoying stuff because of how ownership works with closures
        let clone = sender.clone();
        let config_directory = config_directory.to_owned();
        let _ = thread::spawn(move || {
            execute_script(config_directory, script, clone);
        });
    }

    for line in receiver.iter() {
        let _ = stdout().write(&line).unwrap();
    }
}
