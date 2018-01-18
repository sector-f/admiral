#[macro_use]
extern crate serde_derive;

extern crate toml;

extern crate clap;
use clap::{App, Arg};

extern crate threadpool;
use threadpool::ThreadPool;

use std::process::{Command, exit, Stdio};
use std::io::{Read, BufRead, BufReader};
use std::sync::mpsc::{channel, Sender};
use std::fs::File;
use std::path::PathBuf;
use std::thread::{self, sleep};
use std::time::Duration;
use std::env;

mod config;
use config::{ConfigFile, Script};

#[derive(Debug)]
struct Update {
    position: usize,
    message: String,
}

fn run_command(command: Script, pos: usize, sender: Sender<Update>) {
    let shell = match command.shell {
        Some(shell) => PathBuf::from(shell),
        None => {
            match env::var_os("SHELL") {
                Some(sh) => {
                    PathBuf::from(sh)
                },
                None => {
                    PathBuf::from("/bin/sh")
                }
            }
        }
    };

    let arguments = &["-c", &*command.path];

    if let Some(true) = command.is_static {
        let output = Command::new(&shell).args(arguments).output().expect(&format!("Failed to run {}", &command.path));
        let _ = sender.send(Update { position: pos, message: String::from_utf8_lossy(&output.stdout).trim_matches(&['\r', '\n'] as &[_]).to_owned(), });
    } else {
        match command.reload {
            Some(time) => {
                let time = (time * 1000f64) as u64;
                loop {
                    let output = Command::new(&shell).args(arguments).output().expect(&format!("Failed to run {}", &command.path));
                    let _ = sender.send(Update { position: pos, message: String::from_utf8_lossy(&output.stdout).trim_matches(&['\r', '\n'] as &[_]).to_owned(), });
                    sleep(Duration::from_millis(time));
                }
            },
            None => {
                loop {
                    let output = Command::new(&shell).args(arguments).stdout(Stdio::piped()).spawn().expect(&format!("Failed to run {}", &command.path));
                    let reader = BufReader::new(output.stdout.unwrap());
                    for line in reader.lines().flat_map(Result::ok) {
                        let _ = sender.send(Update { position: pos, message: line.trim_matches(&['\r', '\n'] as &[_]).to_owned(), });
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
             .value_name("FILE")
             .takes_value(true))
        .get_matches();

    let config_file = match matches.value_of("config") {
        Some(file) => PathBuf::from(file),
        None => {
            match config::get_config_file() {
                Some(file) => file,
                None => {
                    eprintln!("Configuration file not found");
                    exit(1);
                },
            }
        }
    };

    if ! config_file.is_file() {
        eprintln!("Configuration file must be regular file");
        exit(1);
    }

    let config_root = PathBuf::from(&config_file.parent().unwrap());

    let mut buffer = String::new();
    match File::open(&config_file) {
        Ok(mut file) => {
            if let Err(e) = file.read_to_string(&mut buffer) {
                eprintln!("Could not read configuration file: {}", e);
                exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error reading configuration file: {}", e);
            exit(1);
        },
    }

    let scripts = match toml::from_str::<ConfigFile>(&buffer) {
        Ok(val) => val.scripts,
        Err(e) => {
            eprintln!("Syntax error in configuration file: {}", e);
            exit(1);
        }
    };

    let (sender, receiver) = channel::<Update>();

    let mut message_vec: Vec<String> = Vec::new();
    let mut print_message = String::new();
    let mut position: usize = 0;
    let _ = env::set_current_dir(&config_root);

    const THREAD_NUM: usize = 1;
    let pool = ThreadPool::new(THREAD_NUM);

    for script in scripts {
        let tx = sender.clone();

        if let Some(true) = script.is_static {
            pool.execute(move || {
                run_command(script, position, tx);
            });
        } else {
            let _ = thread::spawn(move || {
                run_command(script, position, tx);
            });
        }

        position += 1;
        message_vec.push(String::new());
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
