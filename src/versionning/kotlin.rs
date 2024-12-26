use std::io::{Write, Read};
use std::env::current_dir;
use std::path::PathBuf;
use std::fs::File;

fn build_grade_kts_path () -> std::io::Result<PathBuf> {
  Ok(current_dir()?.join("kotlin/build.gradle.kts"))
}

fn read_build_gradle_kts () -> std::io::Result<String> {
  let mut file = File::open(build_grade_kts_path()?)?;

  let mut buffer = String::new();
  file.read_to_string(&mut buffer).unwrap();

  Ok(buffer)
}

pub fn apply_version (old_version: &str, new_version: &str) -> std::io::Result<()> {
  let content = read_build_gradle_kts()?;

  let from = format!("version = \"{}\"", old_version);
  let to = format!("version = \"{}\"", new_version);

  let content = content.replace(&from, &to);

  let mut file = File::create(build_grade_kts_path()?)?;
  file.write_all(content.as_bytes())?;

  Ok(())
}
