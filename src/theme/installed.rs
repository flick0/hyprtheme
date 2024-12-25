use super::online::OnlineTheme;
use super::toml_config::Config;
use super::{Theme, ThemeId, ThemeType};

use expanduser::expanduser;

use anyhow::{anyhow, Result};
use git2::Repository;
use std::path::PathBuf;

use std::os::unix::fs::symlink;

#[derive(Debug, Clone)]
pub struct InstalledTheme {
    pub config: Config,
    pub path: PathBuf,
    pub partial: Theme,

    pub parent_dir: PathBuf,
    parent_config_path: Option<PathBuf>,
}

impl InstalledTheme {
    pub fn from_file(
        path: &PathBuf,
        parent_config_path: Option<PathBuf>,
    ) -> Result<InstalledTheme> {
        match Config::from_toml_file(path) {
            Ok(config) => Ok(InstalledTheme {
                config: config.clone(),
                path: path.clone(),
                parent_dir: path.parent().unwrap().to_path_buf(),
                partial: Theme::new(
                    config.name,
                    config.repo,
                    config.branch,
                    config.desc,
                    Vec::new(),
                ),
                parent_config_path,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn update(&self) -> Result<()> {
        let repo = Repository::open(&self.parent_dir)?;
        let mut remote = repo.find_remote("origin")?;
        match remote.fetch(
            &[&self.partial.branch.clone().unwrap_or("master".to_string())],
            None,
            None,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn uninstall(&self) -> Result<OnlineTheme> {
        let theme = OnlineTheme::from_theme(self.partial.clone());
        std::fs::remove_dir_all(&self.parent_dir)?;
        Ok(theme)
    }

    pub fn get_modules(&self) -> Vec<InstalledTheme> {
        let mut modules = Vec::new();
        for module in &self.config.module {
            let path = self.parent_dir.join(&module.config);
            match InstalledTheme::from_file(&path, Some(self.path.clone())) {
                Ok(theme) => modules.push(theme),
                Err(_) => continue,
            }
        }
        modules
    }

    pub fn get_enabled_modules(&self) -> Vec<InstalledTheme> {
        let mut modules = Vec::new();
        for module in &self.config.module {
            if module.enabled {
                let path = self.parent_dir.join(&module.config);
                match InstalledTheme::from_file(&path, Some(self.path.clone())) {
                    Ok(theme) => modules.push(theme),
                    Err(_) => continue,
                }
            }
        }
        modules
    }

    pub fn get_links(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut links = Vec::new();
        for link in &self.config.link {
            let from = {
                if link.from.starts_with("~") {
                    expanduser(&link.from.to_str().unwrap()).unwrap()
                } else {
                    match link.from.is_absolute() {
                        true => link.from.clone(),
                        false => self.parent_dir.join(&link.from),
                    }
                }
            };
            let to = {
                if link.to.starts_with("~") {
                    expanduser(&link.to.to_str().unwrap()).unwrap()
                } else {
                    match link.to.is_absolute() {
                        true => link.to.clone(),
                        false => self.parent_dir.join(&link.to),
                    }
                }
            };
            links.push((from, to));
        }
        links
    }

    // pub fn get_hypr_modules(&self) -> Vec<PathBuf> {
    //     let mut configs = Vec::new();
    //     for module in &self.config.hypr_module {
    //         let path = self.parent_dir.join(&module.config);
    //         configs.push(path);
    //     }
    //     configs
    // }

    // pub fn get_enabled_hypr_modules(&self) -> Vec<PathBuf> {
    //     let mut configs = Vec::new();
    //     for module in &self.config.hypr_module {
    //         let path = self.parent_dir.join(&module.config);
    //         if module.enabled {
    //             configs.push(path);
    //         }
    //     }
    //     configs
    // }

    pub fn get_hypr_config(&self) -> PathBuf {
        self.config.theme.config.clone()
    }

    pub fn load(&self) {
        if let Some(load) = &self.config.theme.load {
            let path = expanduser(self.parent_dir.join(load).to_str().unwrap()).unwrap();
            match std::process::Command::new(&path).arg(&self.parent_dir).output() {
                Ok(_) => {println!("run load script: {:?}", path);},
                Err(e) => eprintln!("Failed to run load script for theme: {}", e),
            }
        }
    }

    pub fn unload(&self) {
        if let Some(unload) = &self.config.theme.unload {
            let path = expanduser(self.parent_dir.join(unload).to_str().unwrap()).unwrap();
            match std::process::Command::new(&path).arg(&self.parent_dir).output() {
                Ok(_) => {println!("run unload script: {:?}", path);},
                Err(e) => eprintln!("Failed to run unload script for theme: {}", e),
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn is_module(&self) -> bool {
        self.parent_config_path.is_some()
    }

    pub fn save(&self) -> Result<()> {
        let content = match toml::to_string(&self.config) {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };
        match std::fs::write(&self.path, content) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn run_hyprctl_source(&self) {
        let mut path = self.parent_dir.join(&self.config.theme.config);

        path = expanduser(path.to_str().unwrap()).unwrap();

        let cmd = format!("hyprctl keyword source {}", path.display());

        println!("Running: {}", cmd);

        match std::process::Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(out) => println!("output: {}", String::from_utf8_lossy(&out.stdout)),
            Err(e) => eprintln!("Failed to run hyprctl source: {}", e),
        }
    }

    pub fn enable(&mut self) -> Result<()> {
        self.config.enabled = true;

        self.load();

        for (from,to) in &self.get_links() {
            // backup existing file
            if to.exists() {
                // rename to .bak
                let mut bak = to.clone().into_os_string();
                bak.push(".hyprtheme.bak");
                let bak = PathBuf::from(bak);
                match std::fs::rename(&to, &bak) {
                    Ok(_) => println!("Backed up file: ({:?})", bak),
                    Err(e) => {
                        println!("Error backing up file({:?}): {:?}", to, e);
                    },
                }
                
            }

            // create symlink
            match symlink(&from, &to) {
                Ok(_) => println!("Created symlink: {:?} -> {:?}", to, from),
                Err(e) => {
                    println!("Error creating symlink({:?} -> {:?}): {:?}",from , to, e);
                },
            }
            
        }

        if let Some(path) = &self.parent_config_path {
            match InstalledTheme::from_file(path, None) {
                Ok(mut parent) => {
                    for module in &mut parent.config.module {
                        if module.config == self.path {
                            module.enabled = true;
                        }
                    }
                    match parent.save() {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        match self.save() {
            Ok(_) => {}
            Err(e) => return Err(anyhow!("error saving file on enable: {}", e)),
        };

        self.run_hyprctl_source();

        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        self.config.enabled = false;

        self.unload();

        for (from,to) in &self.get_links() {
            // remove symlink
            match std::fs::remove_file(&to) {
                Ok(_) => println!("Removed symlink: {:?}", to),
                Err(e) => {
                    println!("Error removing symlink({:?}): {:?}",to, e);
                },
            }

            // restore backup
            let mut bak = to.clone().into_os_string();
            bak.push(".hyprtheme.bak");
            let bak = PathBuf::from(bak);
            if bak.exists() {
                match std::fs::rename(&bak, &to) {
                    Ok(_) => println!("Restored backup: {:?}", to),
                    Err(e) => {
                        println!("Error restoring backup({:?}): {:?}",bak , e);
                    },
                }
            } else {
                eprintln!("No backup found for link: {:?}", to);
            }
        }

        if let Some(path) = &self.parent_config_path {
            match InstalledTheme::from_file(path, None) {
                Ok(mut parent) => {
                    for module in &mut parent.config.module {
                        if module.config == self.path {
                            module.enabled = false;
                        }
                    }
                    match parent.save() {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        match self.save() {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

impl ThemeType for InstalledTheme {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_type_string(&self) -> String {
        "installed".to_string()
    }

    fn get_id(&self) -> ThemeId {
        ThemeId {
            repo: self.partial.repo.clone(),
            branch: self.partial.branch.clone(),
        }
    }

    fn get_name(&self) -> String {
        self.partial.name.clone()
    }

    fn get_repo(&self) -> String {
        self.partial.repo.clone()
    }

    fn get_branch(&self) -> Option<String> {
        self.partial.branch.clone()
    }

    // fn get_desc(&self) -> String {
    //     self.partial.desc.clone()
    // }

    // fn get_images(&self) -> Vec<String> {
    //     self.partial.images.clone()
    // }
}
