use std::env::current_dir;
use std::path::PathBuf;
use std::io::Write;
use std::fs::File;

use anyhow::{anyhow, Result};
use colored::Colorize;
use regex::Regex;
use crate::io;

fn package_swift_path () -> Result<PathBuf> {
  Ok(current_dir()?.join("Package.swift"))
}

pub fn apply_version (old_version: &str, new_version: &str) -> anyhow::Result<()> {
  let old_content = io::read_file_as_string(package_swift_path()?)?;

  let url_regex = Regex::new("url: \"(.*)\",").unwrap();
  let captures = url_regex.captures(&old_content).unwrap();
  let old_url = captures.get(1).unwrap().as_str();
  let new_url = old_url.replace(old_version, new_version);

  // debug: show the difference
  println!("{}", format!("-url: \"{}\"", old_url).red());
  println!("{}", format!("+url: \"{}\"", new_url).green());

  if old_url == new_url {
    Err(anyhow!("same url after trying to bump, probably incorrect versioning"))
  }
  else {
    let new_content = old_content.replace(old_url, &new_url);
  
    let mut file = File::create(package_swift_path()?)?;
    file.write_all(new_content.as_bytes())?;
  
    Ok(())
  }
}
