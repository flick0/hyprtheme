use std::{fmt::Display, path::PathBuf};

pub mod installed;
pub mod online;
pub mod toml_config;

use anyhow::Result;
use online::OnlineTheme;
use serde::Deserialize;
use std::any::Any;

use reqwest::Client;

use installed::InstalledTheme;

#[derive(Hash, Eq, Default, Clone)]
pub struct ThemeId {
    pub repo: String,
    pub branch: Option<String>,
}

impl ThemeId {
    pub fn new(repo: String, branch: Option<String>) -> ThemeId {
        ThemeId { repo, branch }
    }

    pub fn to_string(&self) -> String {
        match &self.branch {
            Some(branch) => format!("{}@{}", self.repo, branch),
            None => self.repo.clone(),
        }
    }

    pub fn from_theme(theme: Box<dyn ThemeType>) -> ThemeId {
        ThemeId {
            repo: theme.get_repo(),
            branch: theme.get_branch(),
        }
    }
}

impl Display for ThemeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

impl PartialEq for ThemeId {
    fn eq(&self, other: &Self) -> bool {
        if self.repo == "unknown" || other.repo == "unknown" {
            return false;
        }

        if self.repo == "" || other.repo == "" {
            return false;
        }

        self.repo == other.repo && self.branch == other.branch
    }
}

pub trait ThemeType: Any {
    fn as_any(&self) -> &dyn Any;

    fn get_id(&self) -> ThemeId;

    fn get_name(&self) -> String;
    fn get_repo(&self) -> String;
    fn get_branch(&self) -> Option<String>;
    fn get_desc(&self) -> String;
    fn get_images(&self) -> Vec<String>;
    fn get_type_string(&self) -> String;
}

impl Display for dyn ThemeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Theme: {}::{} [{}]",
            &self.get_name(),
            &self.get_id(),
            &self.get_type_string()
        );
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub repo: String,
    pub branch: Option<String>,
    pub config: Option<String>,
    pub desc: String,
    pub images: Vec<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: String::new(),
            repo: String::new(),
            branch: None,
            config: None,
            desc: String::new(),
            images: Vec::new(),
        }
    }
}

impl Theme {
    pub fn new(
        name: String,
        repo: String,
        branch: Option<String>,
        desc: String,
        images: Vec<String>,
    ) -> Theme {
        Theme {
            name,
            repo,
            branch,
            desc,
            images,
            ..Default::default()
        }
    }
}

pub async fn fetch_installed(themes_dir: &PathBuf) -> Result<Vec<InstalledTheme>> {
    let mut themes: Vec<InstalledTheme> = Vec::new();

    for entry in std::fs::read_dir(themes_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let config_path = path.join("hyprtheme.toml");
            if config_path.exists() && config_path.is_file() {
                match InstalledTheme::from_file(&config_path, None) {
                    Ok(theme) => {
                        themes.push(theme);
                    }
                    Err(e) => {
                        eprintln!("skipping broken theme config: {:?}", e);
                    }
                }
            }
        }
    }

    Ok(themes)
}

pub async fn fetch_online(
    urls: Vec<String>,
    blacklist_ids: Option<Vec<ThemeId>>,
) -> Result<Vec<OnlineTheme>> {
    let client = Client::new();

    let blacklist_ids = blacklist_ids.unwrap_or(Vec::new());

    #[derive(Deserialize, Debug)]
    struct ThemesData {
        themes: Vec<Theme>,
    }

    let mut themes: Vec<OnlineTheme> = Vec::new();

    for url in urls {
        match client.get(&url).send().await {
            Ok(response) => match serde_json::from_str::<ThemesData>(&response.text().await?) {
                Ok(data) => {
                    for theme in data.themes {
                        let theme_id = ThemeId::new(theme.repo.clone(), theme.branch.clone());
                        if blacklist_ids.contains(&theme_id) {
                            continue;
                        }
                        themes.push(OnlineTheme::from_theme(theme));
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to parse themes json({:?}) : {:?}",
                        &url,
                        e
                    ))
                }
            },
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to fetch themes json({:?}) : {:?}",
                    url,
                    e
                ))
            }
        }
    }
    Ok(themes)
}

pub async fn fetch_all_installed(directories: &Vec<PathBuf>) -> Result<Vec<Box<dyn ThemeType>>> {
    let mut themes: Vec<Box<dyn ThemeType>> = Vec::new();

    for dir in directories {
        match fetch_installed(&dir).await {
            Ok(installed_themes) => {
                for theme in installed_themes {
                    themes.push(Box::new(theme.clone()));
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch installed themes({:?}) : {:?}", dir, e);
            }
        }
    }

    Ok(themes)
}

pub async fn fetch_all(
    urls: &Vec<String>,
    directories: &Vec<PathBuf>,
) -> Result<Vec<Box<dyn ThemeType>>> {
    let mut themes: Vec<Box<dyn ThemeType>> = Vec::new();

    let mut theme_ids: Vec<ThemeId> = Vec::new();

    for dir in directories {
        match fetch_installed(&dir).await {
            Ok(installed_themes) => {
                for theme in installed_themes {
                    themes.push(Box::new(theme.clone()));
                    theme_ids.push(ThemeId::from_theme(Box::new(theme)));
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch installed themes({:?}) : {:?}", dir, e);
            }
        }
    }

    for theme in fetch_online(urls.to_vec(), Some(theme_ids)).await? {
        themes.push(Box::new(theme.clone()));
    }

    Ok(themes)
}
