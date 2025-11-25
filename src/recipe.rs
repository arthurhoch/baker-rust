use crate::secret::Crypto;
use crate::settings::Settings;
use crate::storage::{read_file, write_file};
use crate::utils::is_url;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub template: String,
    pub path: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub name: String,
    pub template: TemplateInfo,
    pub variables: HashMap<String, String>,
    pub secrets: HashMap<String, String>,
    pub is_remote: bool,
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub instructions: Vec<Instruction>,
    pub case_sensitive: bool,
    pub raw_lines: Vec<String>,
}

pub fn parse(file: &str, settings: &Settings, case_override: Option<bool>) -> Result<Recipe, Box<dyn Error>> {
    let content = read_file(file)?;
    let mut current_section: Option<String> = None;
    let mut raw_lines: Vec<String> = Vec::new();
    let mut partial: HashMap<String, PartialInstruction> = HashMap::new();
    let case_sensitive = case_override.unwrap_or(settings.recipe_case_sensitive);

    for line in content.lines() {
        raw_lines.push(line.to_string());
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let name = trimmed.trim_matches(|c| c == '[' || c == ']').to_string();
            current_section = Some(name);
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if let Some(section) = &current_section {
                let (inst_name, part) = split_section(section)?;
                let entry = partial.entry(inst_name.to_string()).or_default();
                entry.add_value(
                    part,
                    &key,
                    &value,
                    case_sensitive,
                )?;
            }
        }
    }

    let mut instructions = Vec::new();
    for (name, partial) in partial {
        instructions.push(partial.build(name, case_sensitive)?);
    }

    Ok(Recipe {
        instructions,
        case_sensitive,
        raw_lines,
    })
}

#[derive(Debug, Default)]
struct PartialInstruction {
    template: Option<TemplateInfo>,
    variables: HashMap<String, String>,
    secrets: HashMap<String, String>,
}

impl PartialInstruction {
    fn add_value(
        &mut self,
        part: &str,
        key: &str,
        value: &str,
        case_sensitive: bool,
    ) -> Result<(), Box<dyn Error>> {
        match part {
            "template" => {
                let lower = key.to_lowercase();
                let template = self.template.get_or_insert_with(|| TemplateInfo {
                    template: String::new(),
                    path: None,
                    user: None,
                    group: None,
                    mode: None,
                });
                match lower.as_str() {
                    "template" => template.template = value.to_string(),
                    "path" => template.path = Some(value.to_string()),
                    "user" => template.user = Some(value.to_string()),
                    "group" => template.group = Some(value.to_string()),
                    "mode" => template.mode = Some(value.to_string()),
                    other => {
                        return Err(format!("Unsupported attribute '{}' in recipe", other).into())
                    }
                }
            }
            "variables" => {
                let key = normalize_key(key, case_sensitive);
                self.variables.insert(key, value.to_string());
            }
            "secrets" => {
                let key = normalize_key(key, case_sensitive);
                self.secrets.insert(key, value.to_string());
            }
            other => return Err(format!("Unsupported section '{}'", other).into()),
        }
        Ok(())
    }

    fn build(self, name: String, case_sensitive: bool) -> Result<Instruction, Box<dyn Error>> {
        let template = self
            .template
            .ok_or_else(|| format!("Section [{}:template] is required", name))?;
        if template.template.is_empty() {
            return Err(format!("Attribute template is required for [{}]", name).into());
        }
        if is_url(&template.template) && template.path.is_none() {
            return Err(format!(
                "Remote template must have attribute 'path' for [{}]",
                name
            )
            .into());
        }

        Ok(Instruction {
            name,
            is_remote: is_url(&template.template),
            template,
            variables: self.variables,
            secrets: self.secrets,
        })
    }
}

fn split_section(section: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let mut parts = section.rsplitn(2, ':');
    let part = parts
        .next()
        .ok_or("Malformed section: missing part after ':'")?;
    let name = parts
        .next()
        .ok_or("Malformed section: missing instruction name")?;
    Ok((name, part))
}

fn normalize_key(key: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        key.to_string()
    } else {
        key.to_lowercase()
    }
}

pub fn decrypt_secrets(
    instructions: &mut [Instruction],
    crypto: &Crypto,
    case_sensitive: bool,
) -> Result<(), Box<dyn Error>> {
    for instruction in instructions {
        if instruction.secrets.is_empty() {
            continue;
        }
        if instruction.variables.is_empty() {
            instruction.variables = HashMap::new();
        }
        for (key, secret) in instruction.secrets.clone() {
            let decrypted_value = crypto.decrypt(&secret)?;
            instruction
                .variables
                .insert(normalize_key(&key, case_sensitive), decrypted_value);
        }
    }
    Ok(())
}

pub fn encrypt_recipe_file(path: &str, settings: &Settings, crypto: &Crypto) -> Result<(), Box<dyn Error>> {
    let recipe = parse(path, settings, Some(true))?;
    let mut encrypted = HashMap::new();
    for instr in recipe.instructions {
        if instr.secrets.is_empty() {
            continue;
        }
        for (key, secret) in instr.secrets {
            let cipher = crypto.encrypt(&secret)?;
            let map_key = format!("{}::{}", instr.name, key);
            encrypted.insert(map_key, cipher);
        }
    }

    let mut output = String::new();
    let mut current_name: Option<String> = None;
    let mut in_secrets = false;

    for line in recipe.raw_lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let (name, part) = split_section(trimmed.trim_matches(|c| c == '[' || c == ']'))?;
            current_name = Some(name.to_string());
            in_secrets = part == "secrets";
            output.push_str(&line);
            output.push('\n');
            continue;
        }

        if in_secrets {
            if let Some((key, _)) = trimmed.split_once('=') {
                if let Some(name) = &current_name {
                    let map_key = format!("{}::{}", name, key.trim());
                    if let Some(value) = encrypted.get(&map_key) {
                        output.push_str(&format!("{} = {}\n", key.trim(), value));
                        continue;
                    }
                }
            }
        }

        output.push_str(&line);
        output.push('\n');
    }

    write_file(path, output.trim_end_matches('\n'))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_splits_sections_and_variables() {
        let dir = std::env::temp_dir().join("baker_rust_recipe_test");
        let path = dir.join("dev.cfg");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &path,
            "[app:template]\ntemplate=tmpl\npath=out\n[app:variables]\nHOST=dev\n[app:secrets]\nPASS=val\n",
        )
        .unwrap();

        let settings = Settings::load(false).unwrap();
        let recipe = parse(path.to_str().unwrap(), &settings, Some(false)).unwrap();
        assert_eq!(recipe.instructions.len(), 1);
        let instr = &recipe.instructions[0];
        assert_eq!(instr.template.template, "tmpl");
        assert_eq!(instr.variables.get("host").unwrap(), "dev");
        assert_eq!(instr.secrets.get("pass").unwrap(), "val");
    }
}
