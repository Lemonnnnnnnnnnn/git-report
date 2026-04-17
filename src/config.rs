use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub filters: FilterConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FilterConfig {
    #[serde(default)]
    pub exclude_dirs: Vec<String>,
    #[serde(default)]
    pub exclude_extensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub repo: PathBuf,
    pub since: Option<String>,
    pub until: Option<String>,
    pub branch: Option<String>,
    pub no_merge: bool,
    pub exclude_dirs: Vec<String>,
    pub exclude_extensions: Vec<String>,
}

impl EffectiveConfig {
    pub fn should_filter(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        self.exclude_dirs.iter().any(|dir| path.contains(dir))
            || self
                .exclude_extensions
                .iter()
                .any(|ext| path.ends_with(ext))
    }
}

pub fn load_file_config(path: Option<&Path>) -> Result<FileConfig, String> {
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read config {}: {err}", path.display()))?;
    toml::from_str(&content)
        .map_err(|err| format!("failed to parse config {}: {err}", path.display()))
}
