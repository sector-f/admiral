extern crate toml;

use std::process::{Command, exit, Stdio};
use std::io::{stderr, stdout, Write, Read, BufRead, BufReader};
use std::sync::mpsc::{channel, Sender};
use std::fs::File;
use std::path::PathBuf;
use std::thread::{self, sleep};
use std::time::Duration;

mod configuration;

#[derive(Debug)]
struct BarItem {
    path: Vec<String>,
    duration: Option<u64>,
    position: u32,
}

fn execute_script(dir: PathBuf, script: BarItem, sender: Sender<Vec<u8>>,) {
    let path = if PathBuf::from(&script.path[0]).is_relative() {
        PathBuf::from(dir.join(&script.path[0]))
    } else {
        PathBuf::from(&script.path[0])
    };

    let arguments = &script.path[1..];

    match script.duration {
        Some(time) => {
            loop {
                let output = Command::new(&path).args(arguments).output().unwrap();
                let _ = sender.send(output.stdout);
                sleep(Duration::from_secs(time));
            }
        },
        None => {
            let output = Command::new(&path).args(arguments).stdout(Stdio::piped()).spawn().unwrap();
            let reader = BufReader::new(output.stdout.unwrap());
            for line in reader.lines() {
                let _ = sender.send(format!("{}\n", line.unwrap()).into_bytes());
            }
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

    let admiral_config = config_toml.get("admiral").unwrap();
    let items = admiral_config.as_table().unwrap().get("items").unwrap().as_slice().unwrap().iter().map(|x| x.as_str().unwrap()).collect::<Vec<_>>();

    let (sender, receiver) = channel::<Vec<u8>>();

    let mut position: u32 = 0;
    for value in items {
        match config_toml.get(value) {
            Some(script) => {
                let command = BarItem {
                    path: {
                        match script.as_table().unwrap().get("path").unwrap().to_owned() {
                            toml::Value::Array(array) => {
                                array.iter().flat_map(|x| toml::Value::as_str(x)).map(|x| x.to_owned()).collect::<Vec<_>>()
                            },

                            toml::Value::String(string) => {
                                vec![string]
                            },

                            _ => {
                                let _ = stderr().write(format!("Invalid path used for {}\n", value).as_bytes()).unwrap();
                                continue
                            }
                        }
                    },
                        // script.as_table().unwrap()
                        //         .get("path").unwrap()
                        //         .as_slice().unwrap()
                        //         .iter().flat_map(|x| toml::Value::as_str(x)).map(|x| x.to_owned()).collect::<Vec<_>>()
                    duration: script.as_table().unwrap().get("reload").and_then(|x| x.as_integer()).map(|x| x as u64),
                    position: position,
                };

                position += 1;

                // Annoying stuff because of how ownership works with closures
                let clone = sender.clone();
                let config_directory = config_directory.to_owned();
                let _ = thread::spawn(move || {
                    execute_script(config_directory, command, clone);
                });
            },
            None => {
                let _ = stderr().write(format!("No {} found\n", value).as_bytes());
                continue;
            },
        }
    }

    for line in receiver.iter() {
        let _ = stdout().write(&line).unwrap();
    }
}
