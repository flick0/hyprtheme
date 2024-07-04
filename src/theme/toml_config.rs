// use crate::theme::{helper::{create_theme_id, ThemeId}};
use serde::{Deserialize,Serialize};
use std::path::PathBuf;

use anyhow::Result;


#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    pub desc: String,
    pub version: String,
    pub author: String,
    pub repo: String,
    pub branch: Option<String>,

    #[serde(default)]
    pub enabled: bool,

    pub theme: ConfigTheme,

    // #[serde(default)]
    // pub hypr_module: Vec<ConfigHyprModule>,
    #[serde(default)]
    pub module: Vec<ConfigModule>,
    #[serde(default)]
    pub link: Vec<ConfigLink>,
}

#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct ConfigTheme {
    pub config: PathBuf,
    // pub minimum_hyprland_version: String,

    pub load: Option<String>,
    pub unload: Option<String>,
}

// #[derive(Deserialize,Serialize, Debug, Clone)]
// pub struct ConfigHyprModule {
//     pub name: String,
//     pub desc: String,

//     #[serde(default)]
//     pub enabled: bool,

//     pub config: PathBuf,

//     // pub minimum_hyprland_version: Option<String>,
// }

#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct ConfigModule {
    
    #[serde(default)]
    pub enabled:bool,

    pub config: PathBuf,

    // pub minimum_hyprland_version: Option<String>,
}

#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct ConfigLink {
    pub from: PathBuf,
    pub to: PathBuf,

    // #[serde(default)]
    // pub ignore: Vec<PathBuf>,
    // #[serde(default)]
    // pub include: Vec<PathBuf>,
}

impl Config {
    pub fn from_toml_str(toml: &str) -> Result<Self> {
        match toml::from_str(toml) {
            Ok(config) => Ok(config),
            Err(e) => Err(anyhow::anyhow!("Failed to parse theme config file: {}", e)),
        }
    }

    pub fn from_toml_file(file: &PathBuf) -> Result<Self> {
        match std::fs::read_to_string(file) {
            Ok(toml) => Self::from_toml_str(&toml),
            Err(e) => Err(anyhow::anyhow!("Failed to read theme config file: {}", e)),
        }
    }
}