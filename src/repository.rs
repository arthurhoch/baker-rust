use crate::logger::Logger;
use crate::settings::Settings;
use crate::storage::{read_json, write_bytes, write_json};
use crate::utils::is_url;
use chrono::Local;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use ureq::Agent;

pub struct Repository<'a> {
    path: String,
    version: String,
    settings: &'a Settings,
    pub local_path: Option<String>,
}

impl<'a> Repository<'a> {
    pub fn new(name: &str, settings: &'a Settings) -> Result<Self, Box<dyn Error>> {
        if !name.contains(':') {
            return Err("Attr 'name' malformed. Use <path>:<version>".into());
        }
        let mut parts = name.splitn(2, ':');
        let path = parts.next().unwrap().to_string();
        let version = parts.next().unwrap().to_string();
        Ok(Self {
            path,
            version,
            settings,
            local_path: None,
        })
    }

    pub fn pull(&mut self, force: bool, logger: &Logger) -> Result<(), Box<dyn Error>> {
        let url = self.format_url()?;
        let filename = url
            .rsplit('/')
            .next()
            .ok_or("Cannot determine filename from url")?;
        let mut index = IndexRecipe::new(&self.path, &self.version, filename, self.settings)?;
        let target = self.settings.storage_recipe.join(&index.id);
        let file_path = download(&url, Some(target.clone()), force, self.settings, logger)?;
        self.local_path = Some(file_path.to_string_lossy().to_string());
        index.indexing(force)?;
        Ok(())
    }

    pub fn remove(recipe_id: &str, settings: &Settings, logger: &Logger) -> Result<(), Box<dyn Error>> {
        let location = &settings.storage_recipe_index;
        let mut index = read_json(location)?;
        let mut rid = recipe_id.to_string();
        if rid.len() != 64 {
            if let Some(found) = index.keys().find(|k| k.starts_with(recipe_id)) {
                rid = found.clone();
            }
        }
        if index.remove(&rid).is_some() {
            write_json(location, &index)?;
            logger.log(&format!("Removed recipe '{}'", rid));
        } else {
            return Err(format!("Recipe '{}' not found", recipe_id).into());
        }
        Ok(())
    }

    fn check_settings(&self) -> Result<(), Box<dyn Error>> {
        if self.settings.repository.is_none() || self.settings.repository_type.is_none() {
            return Err("REPOSITORY and REPOSITORY_TYPE must be set to download instructions".into());
        }
        let rtype = self.settings.repository_type.as_ref().unwrap();
        if rtype == "custom" {
            if self.settings.repository_custom_pattern.is_none() {
                return Err(
                    "REPOSITORY_CUSTOM_PATTERN must be set when REPOSITORY_TYPE is 'custom'".into(),
                );
            }
        } else if rtype != "github" && rtype != "bitbucket" {
            return Err(format!("REPOSITORY_TYPE '{}' is not supported", rtype).into());
        }
        Ok(())
    }

    fn format_url(&self) -> Result<String, Box<dyn Error>> {
        self.check_settings()?;
        let repository = self
            .settings
            .repository
            .as_ref()
            .ok_or("REPOSITORY must be set")?;
        let ext = "cfg";
        let rtype = self.settings.repository_type.as_ref().unwrap();
        let pattern = match rtype.as_str() {
            "custom" => self
                .settings
                .repository_custom_pattern
                .as_ref()
                .ok_or("REPOSITORY_CUSTOM_PATTERN must be set")?,
            "github" => "%(repository)s/%(version)s/%(path)s.%(ext)s",
            "bitbucket" => "%(repository)s/%(path)s.%(ext)s?at=%(version)s",
            other => return Err(format!("REPOSITORY_TYPE '{}' is not supported", other).into()),
        };
        Ok(pattern
            .replace("%(repository)s", repository)
            .replace("%(ext)s", ext)
            .replace("%(path)s", &self.path)
            .replace("%(version)s", &self.version))
    }
}

pub struct ListRecipes;

impl ListRecipes {
    pub fn list(all_info: bool, settings: &Settings, logger: &Logger) -> Result<(), Box<dyn Error>> {
        let recipes = read_json(&settings.storage_recipe_index)?;
        let mut meta = calc_length(&recipes);
        meta.id = if all_info { 64 } else { 9 };
        let id_len = meta.id;
        let extra_space = 8;
        let mut list_items = String::new();

        for key in recipes.keys() {
            if let Some(recipe) = recipes.get(key).and_then(|v| v.as_object()) {
                let recipe_id = &key[..std::cmp::min(id_len, key.len())];
                let created = recipe
                    .get("datetime")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let created = if all_info { created.to_string() } else { created.chars().take(19).collect() };

                list_items.push_str(recipe_id);
                list_items.push_str(&" ".repeat(meta.id + extra_space - recipe_id.len()));

                for attr in ["remote", "version", "filename"] {
                    let val = recipe
                        .get(attr)
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    list_items.push_str(val);
                    list_items.push_str(&" ".repeat(meta.get(attr) + extra_space - val.len()));
                }
                list_items.push_str(&created);
                list_items.push('\n');
            }
        }

        let header = list_header(&meta, extra_space);
        logger.log(&(header + &list_items));
        Ok(())
    }
}

pub fn download(
    url: &str,
    target: Option<PathBuf>,
    force: bool,
    settings: &Settings,
    logger: &Logger,
) -> Result<PathBuf, Box<dyn Error>> {
    if !is_url(url) {
        return Err(format!("Str '{}' is not a valid url.", url).into());
    }

    let storage_folder = target.unwrap_or_else(|| settings.storage_templates.clone());
    let auth = settings
        .repository_auth
        .clone()
        .unwrap_or_default()
        .replace('\'', "");
    let file_name = url
        .rsplit('/')
        .next()
        .ok_or("Cannot determine filename from url")?;
    let file_path = storage_folder.join(file_name);

    if force || !file_path.is_file() {
        fs::create_dir_all(&storage_folder)?;
        let agent = Agent::new();
        let mut request = agent.get(url);
        if !auth.is_empty() {
            request = request.set("Authorization", &auth);
        }
        let mut response = request.call()?;
        let mut bytes = Vec::new();
        response.into_reader().read_to_end(&mut bytes)?;
        write_bytes(&file_path, &bytes)?;
        logger.log(&format!("{} download DONE!", url));
    } else {
        logger.log(&format!("{} from CACHE!", url));
    }

    Ok(file_path)
}

struct IndexRecipe<'a> {
    remote: &'a str,
    version: &'a str,
    filename: &'a str,
    id: String,
    settings: &'a Settings,
    index: HashMap<String, Value>,
}

impl<'a> IndexRecipe<'a> {
    fn new(remote: &'a str, version: &'a str, filename: &'a str, settings: &'a Settings) -> Result<Self, Box<dyn Error>> {
        let id = generate_id(remote, version)?;
        let index = read_json(&settings.storage_recipe_index)?;
        Ok(Self {
            remote,
            version,
            filename,
            id,
            settings,
            index,
        })
    }

    fn indexing(&mut self, update: bool) -> Result<(), Box<dyn Error>> {
        if !self.index.contains_key(&self.id) || update {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            self.index.insert(
                self.id.clone(),
                json!({
                    "remote": self.remote,
                    "version": self.version,
                    "filename": self.filename,
                    "datetime": now,
                }),
            );
            write_json(&self.settings.storage_recipe_index, &self.index)?;
        }
        Ok(())
    }
}

fn generate_id(remote: &str, version: &str) -> Result<String, Box<dyn Error>> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(remote.as_bytes());
    hasher.update(version.as_bytes());
    let hash = hasher.finalize();
    Ok(crate::utils::hex_encode(&hash))
}

struct LengthMeta {
    id: usize,
    remote: usize,
    version: usize,
    filename: usize,
}

impl LengthMeta {
    fn get(&self, name: &str) -> usize {
        match name {
            "remote" => self.remote,
            "version" => self.version,
            "filename" => self.filename,
            _ => 0,
        }
    }
}

fn calc_length(recipes: &HashMap<String, Value>) -> LengthMeta {
    let mut lengths = LengthMeta {
        id: 9,
        remote: 6,
        version: 7,
        filename: 8,
    };

    for recipe in recipes.values() {
        if let Some(obj) = recipe.as_object() {
            for (attr_name, value) in obj {
                if let Some(str_val) = value.as_str() {
                    let len = str_val.len();
                    match attr_name.as_str() {
                        "remote" => lengths.remote = lengths.remote.max(len),
                        "version" => lengths.version = lengths.version.max(len),
                        "filename" => lengths.filename = lengths.filename.max(len),
                        _ => {}
                    }
                }
            }
        }
    }

    lengths
}

fn list_header(meta: &LengthMeta, extra_space: usize) -> String {
    let mut header = String::new();
    header.push_str("RECIPE ID");
    header.push_str(&" ".repeat(meta.id + extra_space - 9));
    header.push_str("REMOTE");
    header.push_str(&" ".repeat(meta.remote + extra_space - 6));
    header.push_str("VERSION");
    header.push_str(&" ".repeat(meta.version + extra_space - 7));
    header.push_str("FILENAME");
    header.push_str(&" ".repeat(meta.filename + extra_space - 8));
    header.push_str("CREATED \n");
    header
}
