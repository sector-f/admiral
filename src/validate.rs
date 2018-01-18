#[macro_use]
extern crate serde_derive;

extern crate toml;

use std::fs::File;
use std::io::Read;
use std::env::args_os;
use std::process::exit;

mod config;

fn main() {
    let filename = args_os().nth(1).expect("no file specified");
    let mut file = File::open(filename).expect("failed to open file");
    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf).expect("failed to read file");
    match toml::de::from_str::<config::ConfigFile>(&buf) {
        Ok(config) => {
            println!("{:#?}", config);
        },
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        },
    }
}
