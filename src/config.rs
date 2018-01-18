extern crate serde;
use config::serde::de::{Deserialize, Deserializer};
use config::serde::de::Error;

use toml::Value;
use toml::value::Table;

use std::path::PathBuf;
use std::env;

#[derive(Debug, Serialize)]
pub struct ConfigFile {
    pub scripts: Vec<Script>,
}

impl<'de> Deserialize<'de> for ConfigFile {
    fn deserialize<D>(d: D) -> Result<ConfigFile, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(d)?;
        match Value::try_into::<Table>(value) {
            Ok(mut table) => {
                match table.remove("admiral") {
                    Some(admiral) => {
                        match Value::try_into::<ItemList>(admiral) {
                            Ok(admiral) => {
                                let mut scripts = Vec::new();
                                for item in admiral.items {
                                    if let Some(v) = table.get(&item) {
                                        match Value::try_into::<Script>(v.to_owned()) {
                                            Ok(s) => {
                                                scripts.push(s);
                                            },
                                            Err(e) => {
                                                return Err(Error::custom(e.to_string()));
                                            },
                                        }
                                    }
                                }
                                Ok(
                                    ConfigFile {
                                        scripts: scripts,
                                    }
                                )
                            },
                            Err(e) => {
                                Err(Error::custom(e.to_string()))
                            },
                        }
                    },
                    None => {
                        Err(Error::custom("missing \"admiral\" section".to_string()))
                    },
                }
            },
            Err(e) => {
                Err(Error::custom(e.to_string()))
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemList {
    items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Script {
    pub path: String,
    pub reload: Option<f64>,
    #[serde(rename = "static")]
    pub is_static: Option<bool>,
    pub shell: Option<String>,
}

#[allow(dead_code)]
pub fn get_config_file() -> Option<PathBuf> {
    let xdg_path = env::var("XDG_CONFIG_HOME").ok()
        .map(|v| PathBuf::from(v).join("admiral.d").join("admiral.toml"))
        .and_then(if_readable);

    let dot_home = env::var("HOME").ok()
        .map(|v| PathBuf::from(v).join(".config").join("admiral.d").join("admiral.toml"))
        .and_then(if_readable);

    xdg_path.or(dot_home)
}

#[allow(dead_code)]
fn if_readable(path: PathBuf) -> Option<PathBuf> { if path.exists() { Some(path) } else { None } }
