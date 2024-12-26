use cargo_toml::Manifest;
use std::env::current_dir;
use anyhow::Result;

pub mod kotlin;
pub mod swift;
pub mod rust;

pub fn get_current_version () -> Result<Vec<u8>> {
  let path = rust::cargo_toml_path()?;
  let manifest = Manifest::from_path(path)?;
  let version = manifest.package().version();

  Ok(string_to_version(version))
}

pub fn bump_version (version: &[u8], index: usize) -> Vec<u8> {
  let mut version = version.to_vec();

  match index {
    // major
    0 => {
      version[0] += 1;
      version[1..].fill(0);
    },
    // minor
    1 => {
      version[1] += 1;
      version[2..].fill(0);
    },
    // patch
    2 => version[2] += 1,
    _ => unreachable!(),
  }

  version
}

pub fn version_to_string (version: Vec<u8>) -> String {
  version.iter().map(|part| part.to_string()).collect::<Vec<String>>().join(".")
}

pub fn string_to_version (version: &str) -> Vec<u8> {
  version.split(".").map(|part| {
    part
      .parse()
      .expect("only integers are allowed in version")
  }).collect()
}
