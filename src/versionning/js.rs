use std::env::current_dir;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::io;

fn package_json_path() -> Result<PathBuf> {
  Ok(current_dir()?.join("package.json"))
}

pub fn apply_version(old_version: &str, new_version: &str) -> Result<()> {
  let old_content = io::read_file_as_string(package_json_path()?)?;

  let from = format!("\"version\": \"{}\"", old_version);
  let to = format!("\"version\": \"{}\"", new_version);

  // debug: show the difference
  println!("{}", from.red());
  println!("{}", to.green());

  // only replace the first occurence (since the line should be at the very top after name)
  let new_content = old_content.replacen(&from, &to, 1);

  if old_content == new_content {
    Err(anyhow!(
      "same version after trying to bump, probably incorrect versioning"
    ))
  } else {
    io::write_string_to_file(package_json_path()?, new_content)?;
    Ok(())
  }
}
