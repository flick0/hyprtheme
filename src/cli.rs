use crate::consts::{THEME_DOWNLOAD_DIR, THEME_LIST};
use anyhow::{anyhow, Result};
use clap::Parser;
use expanduser::expanduser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, name = "hyprtheme")]
pub struct CliParser {
    /// The path to the hyprtheme config file.
    // #[arg(long,value_parser=parse_path,default_value=HYPRTHEME_CONFIG)]
    // pub config: PathBuf,

    /// path where themes are stored, can be repeated to add multiple directories
    #[arg(short, long, value_parser=parse_path, default_value=THEME_DOWNLOAD_DIR)]
    pub theme_dirs: Vec<PathBuf>,

    /// url to raw text files containing a list of themes in json, can be repeated to add multiple urls
    #[arg(short, long, default_value=THEME_LIST)]
    pub theme_urls: Vec<String>,

    #[command(subcommand)]
    pub commands: CliCommands,
}

#[derive(Parser, Clone)]
pub enum CliCommands {
    /// List all saved themes
    /// and all featured on the official Hyprtheme site
    List(List),

    /// Install a theme from a repository
    ///
    /// Accepted values are:
    /// - Theme name for themes featured on "https://hyprland-community/hyprtheme/browse"
    /// - A git url
    Install(InstallArgs),

    /// Uninstall the installed theme
    Uninstall(UninstallArgs),

    /// Update the installed theme
    Update(UpdateArgs),

    /// Enable an installed theme
    Enable(EnableArgs),

    /// Disable an installed theme
    Disable(DisableArgs),

    /// Source all enabled themes and modules
    Init,
}

#[derive(Parser, Clone)]
pub struct DisableArgs {
    /// uses theme name or theme id (theme_name:branch@repo) to identify the theme to disable
    #[arg()]
    pub theme_id: String,
}

#[derive(Parser, Clone)]
pub struct EnableArgs {
    /// uses theme name or theme id (theme_name:branch@repo) to identify the theme to enable
    #[arg()]
    pub theme_id: String,
}

#[derive(Parser, Clone)]
pub struct List {
    /// show installed themes
    #[arg(short, long)]
    pub installed: bool,

    /// show online themes excluding the ones already installed
    #[arg(short, long)]
    pub online: bool,

    /// whether to show already installed themes while listing online themes
    #[arg(short, long, requires = "online")]
    pub show_installed: bool,
}

#[derive(Parser, Clone)]
pub struct InstallArgs {
    #[arg(short, long, group = "source")]
    pub git: Option<String>,

    #[arg(short, long, requires = "git")]
    pub branch: Option<String>,

    /// uses theme name or theme id (theme_name:branch@repo) to identify the theme to install
    #[arg(group = "source")]
    pub theme_id: Option<String>,
}

#[derive(Parser, Clone)]
pub struct UninstallArgs {
    /// uses theme name or theme id (theme_name:branch@repo) to identify the theme to uninstall
    #[arg()]
    pub theme_id: String,
}

#[derive(Parser, Clone)]
pub struct UpdateArgs {
    /// uses theme name or theme id (theme_name:branch@repo) to identify the theme to update
    #[arg()]
    pub theme_id: String,
}

#[derive(Parser, Clone)]
pub struct CleanAllArgs {}

pub fn parse_path(path: &str) -> Result<PathBuf> {
    let path: PathBuf = expanduser(path).expect("Failed to expand path");
    if path.try_exists()? {
        Ok(path)
    } else {
        Err(anyhow!(format!("Path does not exist: {}", path.display())))
    }
}
