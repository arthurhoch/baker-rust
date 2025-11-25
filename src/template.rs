use crate::logger::Logger;
use crate::recipe::Instruction;
use crate::settings::Settings;
use crate::storage::{read_file, write_file};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

pub fn replace(
    instructions: &[Instruction],
    settings: &Settings,
    logger: &Logger,
) -> Result<(), Box<dyn Error>> {
    for instruction in instructions {
        let mut target = instruction.template.path.clone().unwrap_or_else(|| instruction.template.template.clone());
        if let Some(ext) = &settings.template_ext {
            let suffix = format!(".{}", ext);
            if target.ends_with(&suffix) {
                target = target[..target.len() - suffix.len()].to_string();
            }
        }

        let template_path = &instruction.template.template;
        let replaced = {
            let source = read_file(template_path)?;
            let template = BakerTemplate::new(&source, settings.recipe_case_sensitive);
            template.replace(&instruction.variables)?
        };

        write_file(&target, &replaced)?;
        apply_permissions(instruction, &target, logger);
        logger.log(&format!(
            "{} {} {}",
            instruction.name, template_path, target
        ));
    }
    Ok(())
}

fn apply_permissions(instruction: &Instruction, path: &str, logger: &Logger) {
    if instruction.template.user.is_some() || instruction.template.group.is_some() {
        logger.debug("User/group change not supported on this platform; ignoring");
    }
    if let Some(mode) = &instruction.template.mode {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(value) = u32::from_str_radix(mode, 8) {
                if let Err(err) = fs::set_permissions(path, fs::Permissions::from_mode(value)) {
                    logger.debug(&format!("Failed to set mode {} on {}: {}", mode, path, err));
                }
            }
        }
        #[cfg(not(unix))]
        logger.debug("Mode change not supported on this platform; ignoring");
    }
}

pub struct BakerTemplate {
    template: String,
    case_sensitive: bool,
}

impl BakerTemplate {
    pub fn new(template: &str, case_sensitive: bool) -> Self {
        Self {
            template: template.to_string(),
            case_sensitive,
        }
    }

    pub fn replace(&self, mapping: &HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        let chars: Vec<char> = self.template.chars().collect();
        let mut idx = 0;
        while idx < chars.len() {
            if chars[idx] == '{' && idx + 1 < chars.len() && chars[idx + 1] == '{' {
                idx += 2;
                let mut name = String::new();
                while idx + 1 < chars.len() && !(chars[idx] == '}' && chars[idx + 1] == '}') {
                    name.push(chars[idx]);
                    idx += 1;
                }
                if idx + 1 >= chars.len() {
                    return Err("Unclosed template variable".into());
                }
                idx += 2; // skip closing
                let trimmed = name.trim();
                if trimmed.starts_with('\\') {
                    output.push_str("{{");
                    continue;
                }
                let key = if self.case_sensitive {
                    trimmed.to_string()
                } else {
                    trimmed.to_lowercase()
                };
                let value = mapping
                    .get(&key)
                    .ok_or_else(|| format!("Missing variable {}", trimmed))?;
                output.push_str(value);
            } else {
                output.push(chars[idx]);
                idx += 1;
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_case_insensitive_by_default() {
        let tpl = BakerTemplate::new("host: {{ HOST }}", false);
        let mut map = HashMap::new();
        map.insert("host".to_string(), "dev".to_string());
        let out = tpl.replace(&map).unwrap();
        assert_eq!(out, "host: dev");
    }

    #[test]
    fn escapes_delimiter() {
        let tpl = BakerTemplate::new("{{\\ escape }} data", false);
        let map = HashMap::new();
        let out = tpl.replace(&map).unwrap();
        assert_eq!(out, "{{ data");
    }
}
