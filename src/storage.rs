use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn read_file(path: &str) -> std::io::Result<String> {
    fs::read_to_string(path)
}

pub fn write_file(path: &str, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

pub fn write_bytes(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

pub fn read_json(path: &Path) -> std::io::Result<HashMap<String, Value>> {
    match fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(HashMap::new()),
        Err(err) => Err(err),
    }
}

pub fn write_json(path: &Path, data: &HashMap<String, Value>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, serialized)
}
