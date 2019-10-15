#[macro_use]
extern crate serde_derive;

extern crate toml;

extern crate chan;
extern crate chan_signal;
use chan_signal::Signal;

extern crate clap;
use clap::{App, Arg};

extern crate threadpool;
use threadpool::ThreadPool;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

mod config;
use config::{ConfigFile, Script};

#[derive(Debug)]
enum Message {
    Update(Update),
    Signal(Signal),
}

#[derive(Debug)]
struct Update {
    position: usize,
    message: String,
}

fn run_static_script(
    script: Script,
    pos: usize,
    msg_tx: chan::Sender<Message>,
    interrupt_rx: mpsc::Receiver<()>,
) {
    let shell = script.shell();
    let arguments = &["-c", &script.path];
    let output = Command::new(&shell).args(arguments).output();

    if let Ok(output) = output {
        if let Err(_) = interrupt_rx.try_recv() {
            let _ = msg_tx.send(Message::Update(Update {
                position: pos,
                message: String::from_utf8_lossy(&output.stdout)
                    .trim_matches(&['\r', '\n'] as &[_])
                    .to_owned(),
            }));
        }
    } else if let Err(e) = output {
        eprintln!("Error running {}: {}", &script.path, e);
    } else {
        unreachable!();
    }
}

fn run_script(
    script: Script,
    pos: usize,
    msg_tx: chan::Sender<Message>,
    interrupt_rx: mpsc::Receiver<()>,
) {
    let shell = script.shell();
    let arguments = &["-c", &*script.path];

    match script.reload {
        Some(time) => {
            let time = (time * 1000f64) as u64;
            'cmd_loop1: loop {
                let output = Command::new(&shell)
                    .args(arguments)
                    .output()
                    .expect(&format!("Failed to run {}", &script.path));
                if let Ok(()) = interrupt_rx.try_recv() {
                    break 'cmd_loop1;
                }
                let _ = msg_tx.send(Message::Update(Update {
                    position: pos,
                    message: String::from_utf8_lossy(&output.stdout)
                        .trim_matches(&['\r', '\n'] as &[_])
                        .to_owned(),
                }));
                sleep(Duration::from_millis(time));
            }
        }
        None => 'cmd_loop2: loop {
            let output = Command::new(&shell)
                .args(arguments)
                .stdout(Stdio::piped())
                .spawn()
                .expect(&format!("Failed to run {}", &script.path));
            let reader = BufReader::new(output.stdout.unwrap());
            for line in reader.lines().flat_map(Result::ok) {
                if let Ok(()) = interrupt_rx.try_recv() {
                    break 'cmd_loop2;
                }
                let _ = msg_tx.send(Message::Update(Update {
                    position: pos,
                    message: line.trim_matches(&['\r', '\n'] as &[_]).to_owned(),
                }));
            }
            sleep(Duration::from_millis(10));
        },
    }
}

fn main() {
    let sig_rx = chan_signal::notify(&[Signal::USR1])
        .into_iter()
        .map(|s| Message::Signal(s));
    let (sender, receiver) = chan::async::<Message>();
    let tx = sender.clone();
    thread::spawn(move || {
        for signal in sig_rx {
            tx.send(signal);
        }
    });

    let matches = App::new("admiral")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version"))
        .arg(
            Arg::with_name("config")
                .help("Specify alternate config file")
                .short("c")
                .long("config-file")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();

    let config_file = match matches.value_of("config") {
        Some(file) => PathBuf::from(file),
        None => match config::get_config_file() {
            Some(file) => file,
            None => {
                eprintln!("Configuration file not found");
                exit(1);
            }
        },
    };

    if !config_file.is_file() {
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
        }
        Err(e) => {
            eprintln!("Error reading configuration file: {}", e);
            exit(1);
        }
    }

    let mut scripts = match toml::from_str::<ConfigFile>(&buffer) {
        Ok(val) => val.scripts,
        Err(e) => {
            eprintln!("Syntax error in configuration file: {}", e);
            exit(1);
        }
    };

    'main_loop: loop {
        let mut message_vec: Vec<String> = Vec::new();
        let mut print_message = String::new();
        let mut position: usize = 0;
        let _ = env::set_current_dir(&config_root);

        const THREAD_NUM: usize = 1;
        let pool = ThreadPool::new(THREAD_NUM);
        let mut interrupts = Vec::new();

        for script in &scripts {
            let script = script.to_owned();
            let tx = sender.clone();
            let (int_tx, int_rx) = mpsc::channel::<()>();
            interrupts.push(int_tx);

            if let Some(true) = script.is_static {
                pool.execute(move || {
                    run_static_script(script, position, tx, int_rx);
                });
            } else {
                thread::spawn(move || {
                    run_script(script, position, tx, int_rx);
                });
            }

            position += 1;
            message_vec.push(String::new());
        }

        'msg_loop: for message in receiver.iter() {
            match message {
                Message::Update(line) => {
                    message_vec[line.position] = line.message;
                    let new_message_string = message_vec.iter().cloned().collect::<String>();

                    if print_message != new_message_string {
                        print_message = new_message_string;
                        sleep(Duration::from_millis(5));
                        println!("{}", print_message);
                    }
                }
                Message::Signal(_) => {
                    break 'msg_loop;
                }
            }
        }

        for tx in interrupts {
            let _ = tx.send(());
        }

        let mut buffer = String::new();
        if let Ok(mut file) = File::open(&config_file) {
            if let Err(_) = file.read_to_string(&mut buffer) {
                continue 'main_loop;
            }
        }

        if let Ok(val) = toml::from_str::<ConfigFile>(&buffer) {
            scripts = val.scripts;
        }
    }
}
