use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

pub fn read_file_as_string(path: PathBuf) -> Result<String> {
  let mut file = File::open(path)?;

  let mut buffer = String::new();
  file.read_to_string(&mut buffer)?;

  Ok(buffer)
}

pub fn write_string_to_file(path: PathBuf, content: String) -> Result<()> {
  let mut file = File::create(path)?;
  file.write_all(content.as_bytes())?;

  Ok(())
}

pub fn read_file_as_base64url(path: PathBuf) -> Result<String> {
  let mut file = File::open(path)?;

  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer)?;

  Ok(general_purpose::URL_SAFE.encode(buffer))
}
