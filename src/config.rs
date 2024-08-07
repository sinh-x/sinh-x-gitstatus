use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct General {
    pub database_path: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        let mut config_path: PathBuf = dirs::home_dir().unwrap();
        config_path.push(".local/share/applications/sinh-x/git-status/");

        Self {
            database_path: Some(config_path.to_str().unwrap().to_string()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: General,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General::default(),
        }
    }
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, toml::de::Error> {
        let contents = fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&contents)
    }

    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
