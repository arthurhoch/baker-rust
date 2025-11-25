use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;

pub const VERSION: &str = "0.1.0";

#[derive(Clone, Debug)]
pub struct Settings {
    pub debug: bool,
    pub encoding: String,
    pub recipe_case_sensitive: bool,
    pub repository: Option<String>,
    pub repository_type: Option<String>,
    pub repository_auth: Option<String>,
    pub repository_custom_pattern: Option<String>,
    pub storage_recipe: PathBuf,
    pub storage_recipe_index: PathBuf,
    pub storage_recipe_meta: PathBuf,
    pub storage_key_path: PathBuf,
    pub storage_templates: PathBuf,
    pub template_ext: Option<String>,
    pub custom_overrides: HashMap<String, String>,
}

impl Settings {
    pub fn load(verbose: bool) -> Result<Self, Box<dyn Error>> {
        let home = home_dir()?;
        let baker_dir = home.join(".baker");
        let bakerc_path = home.join(".bakerc");
        let mut values = Settings {
            debug: verbose,
            encoding: "utf-8".to_string(),
            recipe_case_sensitive: false,
            repository: None,
            repository_type: None,
            repository_auth: None,
            repository_custom_pattern: None,
            storage_recipe: baker_dir.join("recipes"),
            storage_recipe_index: baker_dir.join("index"),
            storage_recipe_meta: baker_dir.join("meta"),
            storage_key_path: baker_dir.join("baker.key"),
            storage_templates: baker_dir.join("templates"),
            template_ext: Some("tpl".to_string()),
            custom_overrides: HashMap::new(),
        };

        if bakerc_path.is_file() {
            let content = std::fs::read_to_string(&bakerc_path)?;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = trimmed.split_once('=') {
                    let key = key.trim().to_uppercase();
                    let val = value.trim();
                    values.custom_overrides.insert(key.clone(), val.to_string());
                    match key.as_str() {
                        "DEBUG" => values.debug = parse_bool(val).unwrap_or(verbose),
                        "ENCODING" => values.encoding = val.to_string(),
                        "RECIPE_CASE_SENSITIVE" => {
                            values.recipe_case_sensitive = parse_bool(val).unwrap_or(false)
                        }
                        "REPOSITORY" => values.repository = Some(val.to_string()),
                        "REPOSITORY_TYPE" => values.repository_type = Some(val.to_string()),
                        "REPOSITORY_AUTH" => values.repository_auth = Some(val.to_string()),
                        "REPOSITORY_CUSTOM_PATTERN" => {
                            values.repository_custom_pattern = Some(val.to_string())
                        }
                        "STORAGE_RECIPE" => values.storage_recipe = PathBuf::from(val),
                        "STORAGE_RECIPE_INDEX" => values.storage_recipe_index = PathBuf::from(val),
                        "STORAGE_RECIPE_META" => values.storage_recipe_meta = PathBuf::from(val),
                        "STORAGE_KEY_PATH" => values.storage_key_path = PathBuf::from(val),
                        "STORAGE_TEMPLATES" => values.storage_templates = PathBuf::from(val),
                        "TEMPLATE_EXT" => {
                            values.template_ext = match val.to_lowercase().as_str() {
                                "none" => None,
                                other => Some(other.to_string()),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(values)
    }

    pub fn values(&self, custom_only: bool) -> Vec<(String, String)> {
        if custom_only && !self.custom_overrides.is_empty() {
            return self
                .custom_overrides
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
        }

        vec![
            ("DEBUG".to_string(), self.debug.to_string()),
            ("ENCODING".to_string(), self.encoding.clone()),
            (
                "RECIPE_CASE_SENSITIVE".to_string(),
                self.recipe_case_sensitive.to_string(),
            ),
            (
                "REPOSITORY".to_string(),
                self.repository.clone().unwrap_or_else(|| "None".to_string()),
            ),
            (
                "REPOSITORY_TYPE".to_string(),
                self.repository_type
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
            ),
            (
                "REPOSITORY_AUTH".to_string(),
                self.repository_auth
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
            ),
            (
                "REPOSITORY_CUSTOM_PATTERN".to_string(),
                self.repository_custom_pattern
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
            ),
            (
                "STORAGE_RECIPE".to_string(),
                self.storage_recipe.display().to_string(),
            ),
            (
                "STORAGE_RECIPE_INDEX".to_string(),
                self.storage_recipe_index.display().to_string(),
            ),
            (
                "STORAGE_RECIPE_META".to_string(),
                self.storage_recipe_meta.display().to_string(),
            ),
            (
                "STORAGE_KEY_PATH".to_string(),
                self.storage_key_path.display().to_string(),
            ),
            (
                "STORAGE_TEMPLATES".to_string(),
                self.storage_templates.display().to_string(),
            ),
            (
                "TEMPLATE_EXT".to_string(),
                self.template_ext
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
            ),
        ]
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn home_dir() -> io::Result<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Ok(profile) = env::var("USERPROFILE") {
        return Ok(PathBuf::from(profile));
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Cannot find HOME directory",
    ))
}
