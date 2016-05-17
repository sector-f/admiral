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
struct Update {
    position: usize,
    message: String,
}

fn execute_script(section_name: &str, config_root: PathBuf, configuration: Option<&toml::Table>, position: usize, sender: Sender<Update>,) {
    let configuration = configuration.expect(&format!("Failed to find valid section for {}", section_name));

    let path_vector = match configuration.get("path") {
        Some(value) => {
            let value = value.to_owned();
            match value {
                toml::Value::Array(array) => {
                    array.iter().flat_map(toml::Value::as_str).map(|x| x.to_owned()).collect::<Vec<_>>()
                },

                toml::Value::String(string) => {
                    vec![string]
                },

                _ => {
                    let _ = stderr().write(format!("Invalid path found for {}\n", section_name).as_bytes());
                    panic!();
                },
            }
        },
        None => {
            let _ = stderr().write(format!("No path found for {}\n", section_name).as_bytes());
            panic!();
        },
    };

    let mut path = PathBuf::from(&path_vector[0]);
    if path.is_relative() {
        path = config_root.join(path);
    }
    let arguments = &path_vector[1..];

    let duration: Option<u64> = match configuration.get("reload") {
        Some(value) => {
            let value = value.to_owned();
            match value {
                toml::Value::Float(float) => {
                    Some((float * 1000f64) as u64)
                },

                toml::Value::Integer(int) => {
                    Some((int as f64 * 1000f64) as u64)
                },
                _ => None,
            }
        },
        None => None,
    };

    match duration {
        Some(time) => {
            loop {
                let output = Command::new(&path).args(arguments).output().expect(&format!("Failed to run {}", path.display()));
                let _ = sender.send(Update { position: position, message: String::from_utf8_lossy(&output.stdout).trim().to_owned(), });
                sleep(Duration::from_millis(time));
            }
        },
        None => {
            loop {
                let output = Command::new(&path).args(arguments).stdout(Stdio::piped()).spawn().expect(&format!("Failed to run {}", path.display()));
                let reader = BufReader::new(output.stdout.unwrap());
                for line in reader.lines().flat_map(Result::ok) {
                    let _ = sender.send(Update { position: position, message: line.trim().to_owned(), });
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

    let config_root = PathBuf::from(&config_file.parent().unwrap());

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
                // Annoying stuff because of how ownership works with closures
                let script = script.to_owned();
                let value = value.to_owned();
                let config_root = config_root.clone();
                let clone = sender.clone();

                let _ = thread::spawn(move || {
                    execute_script(&value, config_root, script.as_table(), position, clone);
                });

                position += 1;
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
