mod element;
mod map;

use std::{collections::HashMap, env, fs, path::PathBuf, str::FromStr};

#[must_use]
pub fn get_path_from_config() -> String {
    let mut path = get_olympus_config_path();
    path.push("config.json");
    let data: HashMap<String, serde_json::Value> =
        serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();
    data.get("installs")
        .unwrap()
        .get(0)
        .unwrap()
        .get("path")
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}

#[must_use]
pub fn get_olympus_config_path() -> PathBuf {
    const NAME: &str = "Olympus";
    if let Ok(path) = env::var("OLYMPUS_CONFIG") {
        if fs::metadata(&path).is_ok_and(|meta| meta.is_dir()) {
            return PathBuf::from_str(&path).unwrap();
        }
    }

    if cfg!(target_os = "windows") {
        let appdata = env::var("LocalAppData").unwrap();
        let mut pathbuf = PathBuf::from_str(&appdata).unwrap();
        pathbuf.push(NAME);
        pathbuf
    } else if cfg!(target_os = "linux") {
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            let mut pathbuf = PathBuf::from_str(&xdg_config).unwrap();
            pathbuf.push(NAME);
            pathbuf
        } else {
            let home = env::var("HOME").unwrap();
            let mut pathbuf = PathBuf::from_str(&home).unwrap();
            pathbuf.push(".config");
            pathbuf.push(NAME);
            pathbuf
        }
    } else if cfg!(target_os = "macos") {
        let home = env::var("HOME").unwrap();
        let mut pathbuf = PathBuf::from_str(&home).unwrap();
        pathbuf.push("Library");
        pathbuf.push("Application Support");
        pathbuf.push(NAME);
        pathbuf
    } else {
        unreachable!("this device likely doesn't even support celeste what are you doing")
    }
}

pub use element::*;
pub use map::*;
