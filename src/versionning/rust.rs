use std::env::current_dir;
use std::path::PathBuf;
use std::io::Write;
use std::fs::File;

use anyhow::{anyhow, Result};
use colored::Colorize;
use crate::io;

pub fn cargo_toml_path () -> Result<PathBuf> {
  Ok(current_dir()?.join("Cargo.toml"))
}

pub fn apply_version (old_version: &str, new_version: &str) -> Result<()> {
  let old_content = io::read_file_as_string(cargo_toml_path()?)?;

  let from = format!("version = \"{}\"", old_version);
  let to = format!("version = \"{}\"", new_version);

  // debug: show the difference
  println!("{}", from.red());
  println!("{}", to.green());

  // only replace the first occurence (since the line should be at the very top after the package name)
  // prevents to replace the version in the dependencies...
  let new_content = old_content.replacen(&from, &to, 1);

  if old_content == new_content {
    Err(anyhow!("same version after trying to bump, probably incorrect versioning"))
  }
  else {
    let mut file = File::create(cargo_toml_path()?)?;
    file.write_all(new_content.as_bytes())?;
  
    Ok(())
  }
}