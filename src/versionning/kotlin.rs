use std::env::current_dir;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::io;

fn build_grade_kts_path() -> Result<PathBuf> {
  Ok(current_dir()?.join("kotlin/build.gradle.kts"))
}

pub fn apply_version(old_version: &str, new_version: &str) -> Result<()> {
  let old_content = io::read_file_as_string(build_grade_kts_path()?)?;

  let from = format!("version = \"{}\"", old_version);
  let to = format!("version = \"{}\"", new_version);

  // debug: show the difference
  println!("{}", from.red());
  println!("{}", to.green());

  // only replace the first occurence (since the line should be at the very top after imports)
  // prevents to replace the version in the dependencies...
  let new_content = old_content.replacen(&from, &to, 1);

  if old_content == new_content {
    Err(anyhow!(
      "same version after trying to bump, probably incorrect versioning"
    ))
  } else {
    io::write_string_to_file(build_grade_kts_path()?, new_content)?;
    Ok(())
  }
}
