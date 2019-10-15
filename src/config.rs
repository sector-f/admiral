extern crate serde;
use config::serde::de::Error;
use config::serde::de::{Deserialize, Deserializer};

use toml::value::Table;
use toml::Value;

use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ConfigFile {
    pub scripts: Vec<Script>,
}

impl<'de> Deserialize<'de> for ConfigFile {
    fn deserialize<D>(d: D) -> Result<ConfigFile, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(d)?;
        let mut table =
            Value::try_into::<Table>(value).map_err(|e| Error::custom(e.to_string()))?;
        let admiral = table
            .remove("admiral")
            .ok_or(Error::custom("missing \"admiral\" section".to_string()))?;
        let admiral =
            Value::try_into::<ItemList>(admiral).map_err(|e| Error::custom(e.to_string()))?;

        let mut scripts = Vec::new();
        for item in admiral.items {
            if let Some(v) = table.get(&item) {
                let s = Value::try_into::<Script>(v.to_owned())
                    .map_err(|e| Error::custom(e.to_string()))?;
                scripts.push(s);
            }
        }
        Ok(ConfigFile { scripts })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemList {
    items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub path: String,
    pub reload: Option<f64>,
    #[serde(rename = "static")]
    pub is_static: Option<bool>,
    pub shell: Option<String>,
}

impl Script {
    pub fn shell(&self) -> PathBuf {
        if let Some(ref shell) = self.shell {
            return PathBuf::from(&shell);
        }

        PathBuf::from(env::var("SHELL").unwrap_or("/bin/sh".into()))
    }
}

pub fn get_config_file() -> Option<PathBuf> {
    let xdg_path = env::var("XDG_CONFIG_HOME")
        .ok()
        .map(|v| PathBuf::from(v).join("admiral.d").join("admiral.toml"))
        .and_then(if_readable);

    let dot_home = env::var("HOME")
        .ok()
        .map(|v| {
            PathBuf::from(v)
                .join(".config")
                .join("admiral.d")
                .join("admiral.toml")
        })
        .and_then(if_readable);

    xdg_path.or(dot_home)
}

fn if_readable(path: PathBuf) -> Option<PathBuf> {
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
