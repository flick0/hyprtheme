use std::path::PathBuf;

use anyhow::Result;
use expanduser::expanduser;
use serde::Deserialize;

use crate::theme::{
    fetch_all_installed,
    installed::InstalledTheme,
    online::OnlineTheme,
    toml_config::{ConfigModule, ConfigTheme},
    ThemeType,
};

pub async fn get_enabled_themes(theme_dirs: &Vec<PathBuf>) -> Result<Vec<InstalledTheme>> {
    let mut enabled_themes = Vec::new();

    match fetch_all_installed(theme_dirs).await {
        Ok(themes) => {
            for theme in themes {
                match theme.as_any() {
                    t if t.is::<InstalledTheme>() => {
                        let theme = t.downcast_ref::<InstalledTheme>().unwrap().to_owned();

                        if theme.config.enabled {
                            enabled_themes.push(theme);
                        }
                    }
                    _ => return Err(anyhow::anyhow!("Invalid theme type")),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch themes: {:?}", e);
            return Err(e);
        }
    }

    return Ok(enabled_themes);
}

pub async fn get_all_source_paths(theme_dirs: &Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut source_paths = Vec::new();

    match get_enabled_themes(theme_dirs).await {
        Ok(themes) => {
            for theme in themes {
                source_paths.push(theme.config.theme.config.to_owned());

                for module in &theme.get_enabled_modules() {
                    source_paths.push(module.config.theme.config.to_owned());
                }

                // for hypr_module in theme.get_enabled_hypr_modules() {
                //     source_paths.push(hypr_module.to_owned());
                // }
            }
        }
        Err(e) => {
            eprintln!("Failed to get enabled themes: {:?}", e);
        }
    }

    Ok(source_paths)
}

pub async fn init(theme_dirs: &Vec<PathBuf>) -> Result<()> {
    match get_enabled_themes(theme_dirs).await {
        Ok(themes) => {
            for theme in themes {
                theme.load();
                theme.run_hyprctl_source();
            }
        }
        Err(e) => {
            eprintln!("Failed to get enabled themes: {:?}", e);
        }
    }

    return Ok(());
}
