use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

use anyhow::Result;

pub fn read_file_as_string (path: PathBuf) -> Result<String> {
  let mut file = File::open(path)?;

  let mut buffer = String::new();
  file.read_to_string(&mut buffer).unwrap();

  Ok(buffer)
}
