extern crate toml;

use std::process::Command;
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

fn execute_script(script: BarItem, _: Sender<Vec<u8>>,) {
    let path = &script.path;
    match script.duration {
        Some(time) => {
            loop {
                let output = Command::new(path).spawn().unwrap();
                // let _ = sender.send(output.stdout);
                sleep(Duration::from_secs(time));
            }
        },
        None => {
            let output = Command::new(path).spawn().unwrap();
        },
    }
}

fn main() {
    let mut buffer = String::new();
    if let Some(mut file) = configuration::get_config_path().and_then(|p| File::open(p).ok()) {
        file.read_to_string(&mut buffer).expect("Could not read configuration file");
    }
    let config_toml = toml::Parser::new(&buffer).parse().unwrap();

    let (sender, receiver) = channel::<Vec<u8>>();

    for (key, value) in config_toml {
        let script = BarItem {
            path: PathBuf::from(value.as_table().unwrap().get("path").unwrap().as_str().unwrap()),
            duration: value.as_table().unwrap().get("reload").and_then(|x| x.as_integer()).map(|x| x as u64),
        };

        let clone = sender.clone();
        let _ = thread::spawn(move || {
            execute_script(script, clone);
        });
    }

    for line in receiver.iter() {
        let _ = stdout().write(&line).unwrap();
    }
}
