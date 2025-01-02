use std::env::current_dir;
use std::path::PathBuf;

use crate::io;
use anyhow::{anyhow, Result};
use colored::Colorize;
use regex::Regex;

fn package_swift_path() -> Result<PathBuf> {
  Ok(current_dir()?.join("Package.swift"))
}

pub fn apply_version(old_version: &str, new_version: &str) -> anyhow::Result<()> {
  let old_content = io::read_file_as_string(package_swift_path()?)?;

  let url_regex = Regex::new("url: \"(.*)\",")?;
  let captures = url_regex.captures(&old_content).expect("url not found");

  let old_url = captures.get(1).expect("url not captured").as_str();

  let new_url = old_url.replace(old_version, new_version);

  // debug: show the difference
  println!("{}", format!("-url: \"{}\"", old_url).red());
  println!("{}", format!("+url: \"{}\"", new_url).green());

  if old_url == new_url {
    Err(anyhow!(
      "same url after trying to bump, probably incorrect versioning"
    ))
  } else {
    let new_content = old_content.replace(old_url, &new_url);
    io::write_string_to_file(package_swift_path()?, new_content)?;
    Ok(())
  }
}

pub fn apply_checksum(new_checksum: &str) -> anyhow::Result<()> {
  let old_content = io::read_file_as_string(package_swift_path()?)?;

  let checksum_regex = Regex::new(r#"checksum: "(.*)"\),"#)?;
  let captures = checksum_regex
    .captures(&old_content)
    .expect("checksum not found");

  let old_checksum = captures.get(1).expect("checksum not captured").as_str();

  // debug: show the difference
  println!("{}", format!("-checksum: \"{}\"", old_checksum).red());
  println!("{}", format!("+checksum: \"{}\"", new_checksum).green());

  let new_content = old_content.replace(old_checksum, new_checksum);
  io::write_string_to_file(package_swift_path()?, new_content)?;

  Ok(())
}
