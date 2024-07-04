use anyhow::Result;
use std::path::PathBuf;

use crate::theme::{fetch_all, fetch_all_installed, ThemeType};

pub fn reload_hyprctl() {
    std::process::Command::new("hyprctl")
        .arg("reload")
        .spawn()
        .expect("Failed to reload hyprctl");
}

pub fn sanitize_name(name: &str) -> String {
    let mut name = name.to_string();
    name = name.replace(" ", "-");
    name = name.replace("_", "-");
    name = name.replace(".", "-");
    name = name.replace("/", "-");
    name = name.replace("\\", "-");
    name = name.replace(":", "-");
    name = name.replace("*", "-");
    name = name.replace("?", "-");
    name = name.replace("\"", "-");
    name = name.replace("<", "-");
    name = name.replace(">", "-");
    name = name.replace("|", "-");
    return name;
}

pub async fn identify_theme(
    theme_id: &str,
    theme_dirs: &Vec<PathBuf>,
    theme_urls: &Vec<String>,
) -> Result<Box<dyn ThemeType>> {
    let mut matches = Vec::new();

    match fetch_all(theme_urls, theme_dirs).await {
        Ok(themes) => {
            for theme in themes {
                if theme.get_id().to_string().to_lowercase() == theme_id.to_lowercase() {
                    matches.clear();
                    matches.push(theme);
                    break;
                }
                if theme.get_name().to_lowercase() == theme_id.to_lowercase() {
                    matches.push(theme);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch themes: {:?}", e);
        }
    }

    return match matches.len() {
        0 => Err(anyhow::anyhow!("No themes found with id: {}", theme_id)),
        1 => Ok(matches.remove(0)),
        _ => Err(anyhow::anyhow!(
            "Multiple themes found with id: {}",
            theme_id
        )),
    };
}

pub async fn identify_offline_theme(
    theme_id: &str,
    theme_dirs: &Vec<PathBuf>,
) -> Result<Box<dyn ThemeType>> {
    let mut matches = Vec::new();

    match fetch_all_installed(theme_dirs).await {
        Ok(themes) => {
            for theme in themes {
                if theme.get_id().to_string().to_lowercase() == theme_id.to_lowercase() {
                    matches.push(theme);
                    break;
                }
                if theme.get_name().to_lowercase() == theme_id.to_lowercase() {
                    matches.push(theme);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch themes: {:?}", e);
        }
    }

    return match matches.len() {
        0 => Err(anyhow::anyhow!("No themes found with id: {}", theme_id)),
        1 => Ok(matches.remove(0)),
        _ => Err(anyhow::anyhow!(
            "Multiple themes found with id: {}",
            theme_id
        )),
    };
}
