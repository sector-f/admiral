extern crate toml;
extern crate clap;

use std::process::{Command, exit, Stdio};
use std::io::{stderr, Write, Read, BufRead, BufReader};
use std::sync::mpsc::{channel, Sender};
use std::fs::File;
use std::path::PathBuf;
use std::thread::{self, sleep};
use std::time::Duration;

use clap::{App, Arg};

mod configuration;

#[derive(Debug)]
struct BarItem {
    path: Vec<String>,
    duration: Option<u64>,
    position: usize,
}

#[derive(Debug)]
struct Update {
    position: usize,
    message: String,
}

fn execute_script(config_file: PathBuf, script: BarItem, sender: Sender<Update>,) {
    let path = if PathBuf::from(&script.path[0]).is_relative() {
        PathBuf::from(config_file.parent().unwrap().join(&script.path[0]))
    } else {
        PathBuf::from(&script.path[0])
    };

    let arguments = &script.path[1..];

    match script.duration {
        Some(time) => {
            loop {
                let output = Command::new(&path).args(arguments).output().expect(&format!("Failed to run {}", path.display()));
                let _ = sender.send(Update { position: script.position, message: String::from_utf8_lossy(&output.stdout).trim().to_owned(), });
                sleep(Duration::from_millis(time));
            }
        },
        None => {
            loop {
                let output = Command::new(&path).args(arguments).stdout(Stdio::piped()).spawn().expect(&format!("Failed to run {}", path.display()));
                let reader = BufReader::new(output.stdout.unwrap());
                for line in reader.lines().flat_map(Result::ok) {
                    let _ = sender.send(Update { position: script.position, message: line.trim().to_owned(), });
                }
                sleep(Duration::from_millis(10));
            }
        },
    }
}

fn main() {
    let matches = App::new("admiral")
        .arg(Arg::with_name("config")
             .help("Specify alternate config file")
             .short("c")
             .long("config-file")
             .takes_value(true))
        .get_matches();

    let config_file = match matches.value_of("config") {
        Some(file) => PathBuf::from(file),
        None => {
            match configuration::get_config_file() {
                Some(file) => file,
                None => {
                    let _ = stderr().write("Configuration file not found\n".as_bytes());
                    exit(1);
                },
            }
        }
    };

    if ! config_file.is_file() {
        let _ = stderr().write("Invalid configuration file specified\n".as_bytes());
        exit(1);
    }


    let mut buffer = String::new();
    if let Ok(mut file) = File::open(&config_file) {
        file.read_to_string(&mut buffer).expect("Could not read configuration file");
    }

    let config_toml = toml::Parser::new(&buffer).parse().unwrap();

    let admiral_config = config_toml.get("admiral").unwrap();
    let items = admiral_config.as_table().unwrap().get("items").unwrap().as_slice().unwrap().iter().map(|x| x.as_str().unwrap()).collect::<Vec<_>>();

    let (sender, receiver) = channel::<Update>();

    let mut message_vec: Vec<String> = Vec::new();
    let mut print_message = String::new();

    let mut position: usize = 0;
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
                let config_file = config_file.to_owned();
                let _ = thread::spawn(move || {
                    execute_script(config_file, command, clone);
                });

                message_vec.push(String::new());
            },
            None => {
                let _ = stderr().write(format!("No {} found\n", value).as_bytes());
                continue;
            },
        }
    }

    for line in receiver.iter() {
        let position = line.position;
        message_vec[position] = line.message;
        if print_message != message_vec.iter().cloned().collect::<String>() {
            print_message = message_vec.iter().cloned().collect::<String>();
            sleep(Duration::from_millis(5));
            println!("{}", print_message);
        }
    }
}
