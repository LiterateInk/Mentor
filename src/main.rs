mod io;
mod os;
mod git;

mod versionning;
use versionning::{bump_version, get_library_name, get_current_version, version_to_string, kotlin, swift, rust};

use std::{fs::create_dir_all, process::Command};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

#[derive(Debug, Parser)]
#[command(name = "mentor")]
#[command(about = "Handle heavy tasks with a single command.", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
  Bump {
    #[arg(value_enum, value_name = "type")]
    bump_type: BumpType
  },
  CiPush {
    #[arg(value_enum, value_name = "type")]
    push_type: PushType
  },
  MakeSwift,
  MakeKotlin,
  Version
}

#[derive(Debug, ValueEnum, Copy, Clone, PartialEq, Eq)]
enum BumpType {
  // indexes in the version array
  Major = 0,
  Minor = 1,
  Patch = 2,
}

#[derive(Debug, ValueEnum, Copy, Clone, PartialEq, Eq)]
enum PushType {
  Prepare,
  Swift,
  ReleaseTag,
}

fn main() -> anyhow::Result<()> {
  let args = Cli::parse();

  match args.command {
    Commands::Bump { bump_type } => {
      // apply to vectors (easier to handle)
      let old_version = get_current_version()?;
      let new_version = bump_version(&old_version, bump_type as usize);

      // to string !
      let old_version = version_to_string(old_version);
      let new_version = version_to_string(new_version);
      println!("Bumping version from '{old_version}' to '{new_version}'");

      // apply to configuration files
      println!("\nApplying to RUST");
      rust::apply_version(&old_version, &new_version)?;

      println!("\nApplying to KOTLIN");
      kotlin::apply_version(&old_version, &new_version)?;

      println!("\nApplying to SWIFT");
      swift::apply_version(&old_version, &new_version)?;
      println!("{}", "WARN: 'checksum' property was left intact, make sure to update it manually.".yellow());
    },
    Commands::CiPush { push_type } => {
      let version = version_to_string(get_current_version()?);
      git::configure()?;

      match push_type {
        PushType::Prepare => {
          git::run(&["add", "."])?;
          git::run(&["commit", "-m", format!("chore: bump version to v{version}").as_ref()])?;
          git::run(&["push"])?;
        },
        PushType::Swift => {
          git::run(&["add", "."])?;
          git::run(&["commit", "-m", format!("chore: update swift bindings and checksum for v{version}").as_ref()])?;
          git::run(&["push"])?;
        },
        PushType::ReleaseTag => {
          git::run(&["tag", "-a", &version, "-m", format!("Release v{version}").as_ref()])?;
          git::run(&["push", "origin", "main", "--tags"])?;
        }
      }
    },
    Commands::MakeSwift => {
      let library_name = get_library_name()?;

      let targets = [
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
        "x86_64-apple-ios",
        "aarch64-apple-darwin",
        "x86_64-apple-darwin"
      ];

      for target in targets {
        println!("\nbuilding for {target}...");

        let mut child = Command::new("cargo")
          .args([
            "build", "--release",
            "--target", target,
            "--features", "ffi"
          ])
          .spawn()?;
        
        child.wait()?;
      }

      // cleanup
      let _ = std::fs::remove_dir_all("target/aarch64-x86_64-apple-ios-sim/release");
      let _ = create_dir_all("target/aarch64-x86_64-apple-ios-sim/release");

      println!("merging libraries for iOS simulator...");
      let mut child = Command::new("lipo")
        .args([
          "-create",
          format!("target/aarch64-apple-ios-sim/release/lib{library_name}.a").as_ref(),
          format!("target/x86_64-apple-ios/release/lib{library_name}.a").as_ref(),
          "-output",
          format!("target/aarch64-x86_64-apple-ios-sim/release/lib{library_name}.a").as_ref()
        ])
        .spawn()?;

      child.wait()?;
      
      // cleanup
      let _ = std::fs::remove_dir_all("target/aarch64-x86_64-apple-darwin/release");
      let _ = create_dir_all("target/aarch64-x86_64-apple-darwin/release");

      println!("merging libraries for macOS...");
      let mut child = Command::new("lipo")
        .args([
          "-create",
          format!("target/aarch64-apple-darwin/release/lib{library_name}.a").as_ref(),
          format!("target/x86_64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "-output",
          format!("target/aarch64-x86_64-apple-darwin/release/lib{library_name}.a").as_ref()
        ])
        .spawn()?;

      child.wait()?;

      println!("updating bindings...");
      let mut child = Command::new("cargo")
        .args([
          "run", "--bin", "uniffi-bindgen-swift",
          "--features", "ffi",
          // will grab bindings from aarch64-apple-darwin, not sure if it's the best choice though...
          format!("target/aarch64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "swift", "--swift-sources"
        ])
        .spawn()?;
      
      child.wait()?;

      // cleanup
      let _ = std::fs::remove_dir_all("target/uniffi-xcframework-staging");
      let _ = std::fs::remove_dir_all(format!("target/{library_name}FFI.xcframework"));
      let _ = std::fs::remove_dir_all(format!("target/{library_name}FFI.xcframework.zip"));
      
      println!("creating headers and modulemap...");
      let mut child = Command::new("cargo")
        .args([
          "run", "--bin", "uniffi-bindgen-swift",
          "--features", "ffi",
          // will grab bindings from aarch64-apple-darwin, not sure if it's the best choice though...
          format!("target/aarch64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "target/uniffi-xcframework-staging",
          "--headers",
          "--module-name", format!("{library_name}FFI").as_ref(),
          "--modulemap", "--modulemap-filename", "module.modulemap"
        ])
        .spawn()?;
      
      child.wait()?;

      println!("creating xcframework...");
      let mut child = Command::new("xcodebuild")
        .args([
          "-create-xcframework",
          // iOS simulator
          "-library", format!("target/aarch64-x86_64-apple-ios-sim/release/lib{library_name}.a").as_ref(),
          "-headers", "target/uniffi-xcframework-staging",
          // macOS
          "-library", format!("target/aarch64-x86_64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "-headers", "target/uniffi-xcframework-staging",
          // iOS
          "-library", format!("target/aarch64-apple-ios/release/lib{library_name}.a").as_ref(),
          "-headers", "target/uniffi-xcframework-staging",

          "-output", format!("target/{library_name}FFI.xcframework").as_ref()
        ])
        .spawn()?;
      
      child.wait()?;

      println!("zipping xcframework...");
      let mut child = Command::new("ditto")
        .args([
          "-c", "-k", "--sequesterRsrc", "--keepParent",
          format!("target/{library_name}FFI.xcframework").as_ref(),
          format!("target/{library_name}FFI.xcframework.zip").as_ref()
        ])
        .spawn()?;
      
      child.wait()?;

      println!("applying zip checksum...");
      let output = Command::new("shasum")
        .args([
          "-a", "256",
          format!("target/{library_name}FFI.xcframework.zip").as_ref()
        ])
        .output()?;

      // e.g. format: <hash> <filename>
      let output = String::from_utf8_lossy(&output.stdout);
      let checksum = output.split_whitespace().next().unwrap();
      swift::apply_checksum(checksum)?;
    }
    Commands::MakeKotlin => {
      let mut child = Command::new("cargo")
        .args([
          "ndk",
          "-o", "kotlin/src/androidMain/jniLibs",
          "--manifest-path", "Cargo.toml",
          "-t", "armeabi-v7a", // armv7-linux-androideabi
          "-t", "arm64-v8a",   // aarch64-linux-android
          "-t", "x86",         // i686-linux-android
          "-t", "x86_64",      // x86_64-linux-android
          "build", "--release",
          "--features", "ffi"
        ])
        .spawn()?;
      
      child.wait()?;

      let mut child = Command::new("cargo")
        .args([
          "run",
          "--bin", "uniffi-bindgen",
          "--features", "ffi",
          "generate",
          "--library", format!("target/debug/lib{}.{}", get_library_name()?, os::dylib_or_so()).as_ref(),
          "--out-dir", "kotlin/src/commonMain/kotlin",
          "--language", "kotlin",
          "--no-format" // prevent ktlint from formatting the generated code
        ])
        .spawn()?;

      child.wait()?;
    },
    Commands::Version => {
      let version = version_to_string(get_current_version()?);
      println!("{version}");
    },
  }

  Ok(())
}
