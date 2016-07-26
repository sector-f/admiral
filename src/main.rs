extern crate toml;
extern crate clap;

use std::process::{Command, exit, Stdio};
use std::io::{stderr, Write, Read, BufRead, BufReader};
use std::sync::mpsc::{channel, Sender};
use std::fs::File;
use std::path::PathBuf;
use std::thread::{self, sleep};
use std::time::Duration;
use std::env;
use std::ffi::OsStr;

use clap::{App, Arg};

mod command;
mod config;

#[derive(Debug)]
struct Update {
    position: usize,
    message: String,
}

fn run_command(command: command::Command, sender: Sender<Update>) {
    let shell = OsStr::new(&command.shell);
    let arguments = &["-c", &*command.path];

    if command.is_static {
        let output = Command::new(&shell).args(arguments).output().expect(&format!("Failed to run {}", &command.path));
        let _ = sender.send(Update { position: command.position, message: String::from_utf8_lossy(&output.stdout).trim_matches(&['\r', '\n'] as &[_]).to_owned(), });
    } else {
        match command.reload {
            Some(time) => {
                loop {
                    let output = Command::new(&shell).args(arguments).output().expect(&format!("Failed to run {}", &command.path));
                    let _ = sender.send(Update { position: command.position, message: String::from_utf8_lossy(&output.stdout).trim_matches(&['\r', '\n'] as &[_]).to_owned(), });
                    sleep(Duration::from_millis(time));
                }
            },
            None => {
                loop {
                    let output = Command::new(&shell).args(arguments).stdout(Stdio::piped()).spawn().expect(&format!("Failed to run {}", &command.path));
                    let reader = BufReader::new(output.stdout.unwrap());
                    for line in reader.lines().flat_map(Result::ok) {
                        let _ = sender.send(Update { position: command.position, message: line.trim_matches(&['\r', '\n'] as &[_]).to_owned(), });
                    }
                    sleep(Duration::from_millis(10));
                }
            },
        }
    }
}

fn main() {
    let matches = App::new("admiral")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version"))
        .arg(Arg::with_name("config")
             .help("Specify alternate config file")
             .short("c")
             .long("config-file")
             .takes_value(true))
        .get_matches();

    let config_file = match matches.value_of("config") {
        Some(file) => PathBuf::from(file),
        None => {
            match config::get_config_file() {
                Some(file) => file,
                None => {
                    let _ = stderr().write("Configuration file not found\n".as_bytes());
                    exit(1);
                },
            }
        }
    };

    if ! config_file.is_file() {
        let _ = writeln!(stderr(), "Invalid configuration file specified");
        exit(1);
    }

    let config_root = PathBuf::from(&config_file.parent().unwrap());

    let mut buffer = String::new();
    match File::open(&config_file) {
        Ok(mut file) => {
            let _ = file.read_to_string(&mut buffer).expect("Could not read configuration file");
        },
        Err(e) => {
            let _ = writeln!(stderr(), "Error reading configuration file: {}", e);
        },
    }

    let config_toml = match toml::Parser::new(&buffer).parse() {
        Some(val) => val,
        None => {
            panic!("Syntax error in configuration file");
        }
    };

    let admiral_config = config_toml.get("admiral").expect("No [admiral] section found");
    let items = admiral_config
        .as_table().expect("admiral section is not valid TOML table")
        .get("items").expect("[admiral] section does not have an \"items\" section")
        .as_slice().expect("items section is not valid TOML array")
        .iter().map(|x| x.as_str().unwrap()).collect::<Vec<_>>();

    let (sender, receiver) = channel::<Update>();

    let mut message_vec: Vec<String> = Vec::new();
    let mut print_message = String::new();
    let mut position: usize = 0;
    let _ = env::set_current_dir(&config_root);

    for value in items {
        match config_toml.get(value) {
            Some(script) => {
                let table = match script.as_table() {
                    Some(table) => table.to_owned(),
                    None => {
                        let _ = writeln!(stderr(), "No {} found", value);
                        continue;
                    },
                };

                let value = value.to_owned();
                let clone = sender.clone();

                let command = command::get_command(&value, &table, position);

                let _ = thread::spawn(move || {
                    run_command(command, clone);
                });

                position += 1;
                message_vec.push(String::new());
            },
            None => {
                let _ = writeln!(stderr(), "No {} found.", value);
                continue;
            },
        }
    }

    for line in receiver.iter() {
        message_vec[line.position] = line.message;
        let new_message_string = message_vec.iter().cloned().collect::<String>();

        if print_message != new_message_string {
            print_message = new_message_string;
            sleep(Duration::from_millis(5));
            println!("{}", print_message);
        }
    }
}
