use crate::utils::{open_file, read_file, write_file};
use anyhow::Result;
use std::{fs::File, io};

pub fn open_cargo_toml() -> io::Result<File> {
  open_file("Cargo.toml")
}

pub fn get_current_version() -> Result<String> {
  let content = read_file(&mut open_cargo_toml()?)?;
  let content: toml::Value = toml::from_str(&content)?;

  let version = content
    .get("package")
    .and_then(|package| package.get("version"))
    .and_then(|version| version.as_str())
    .ok_or_else(|| anyhow::anyhow!("'Cargo.toml' is missing 'version' property."))?;

  Ok(version.to_string())
}

/// Edits the `Cargo.toml` file and updates the value of the `version` property.
pub fn bump_version(version: &str) -> Result<()> {
  let mut file = open_cargo_toml()?;

  let content = read_file(&mut file)?;
  let mut content: toml::Value = toml::from_str(&content)?;

  let version_property = content
    .get_mut("package")
    .and_then(|package| package.get_mut("version"))
    .ok_or_else(|| anyhow::anyhow!("'Cargo.toml' is missing 'version' property."))?;

  *version_property = toml::Value::String(version.to_string());

  write_file(&mut file, content.to_string())?;

  Ok(())
}
