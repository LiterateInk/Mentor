mod git;
mod io;
mod os;
mod strings;

mod versionning;
use versionning::{
  bump_version, get_current_version, get_library_name, js, kotlin, rust, swift, version_to_string,
};

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use regex::{Captures, Regex};
use std::{env::current_dir, fs::create_dir_all, process::Command};

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
    bump_type: BumpType,
  },
  CiPush {
    #[arg(value_enum, value_name = "type")]
    push_type: PushType,
  },
  MakeSwift,
  MakeKotlin,
  MakeJS,
  Version,
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

      println!("\nApplying to RUST");
      rust::apply_version(&old_version, &new_version)?;

      // update Cargo.lock because we changed the version in Cargo.toml
      let mut child = Command::new("cargo").args(["generate-lockfile"]).spawn()?;

      child.wait()?;

      println!("\nApplying to KOTLIN");
      kotlin::apply_version(&old_version, &new_version)?;

      println!("\nApplying to SWIFT");
      swift::apply_version(&old_version, &new_version)?;
      println!(
        "{}",
        "WARN: 'checksum' property was left intact, make sure to update it manually.".yellow()
      );

      println!("\nApplying to JS");
      js::apply_version(&old_version, &new_version)?;

      // update dependencies to latest version (if any)
      let mut child = Command::new("pnpm")
        .args([
          "add",
          "@literate.ink/utilities@latest",
          "@scure/base@latest",
        ])
        .spawn()
        .expect("failed to update dependencies, make sure pnpm is installed");

      child.wait()?;
    }
    Commands::CiPush { push_type } => {
      let version = version_to_string(get_current_version()?);
      git::configure()?;

      match push_type {
        PushType::Prepare => {
          git::run(&["add", "."])?;
          git::run(&[
            "commit",
            "-m",
            format!("chore: bump version to v{version} with latest dependencies").as_ref(),
          ])?;
          git::run(&["push"])?;
        }
        PushType::Swift => {
          git::run(&["add", "."])?;
          git::run(&[
            "commit",
            "-m",
            format!("chore: update swift bindings and checksum for v{version}").as_ref(),
          ])?;
          git::run(&["push"])?;
        }
        PushType::ReleaseTag => {
          git::run(&[
            "tag",
            "-a",
            &version,
            "-m",
            format!("Release v{version}").as_ref(),
          ])?;
          git::run(&["push", "origin", "main", "--tags"])?;
        }
      }
    }
    Commands::MakeSwift => {
      let library_name = get_library_name()?;

      let targets = [
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
        "x86_64-apple-ios",
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
      ];

      for target in targets {
        println!("\nbuilding for {target}...");

        let mut child = Command::new("cargo")
          .args([
            "build",
            "--release",
            "--target",
            target,
            "--features",
            "ffi",
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
          format!("target/aarch64-x86_64-apple-ios-sim/release/lib{library_name}.a").as_ref(),
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
          format!("target/aarch64-x86_64-apple-darwin/release/lib{library_name}.a").as_ref(),
        ])
        .spawn()?;

      child.wait()?;

      println!("updating bindings...");
      let mut child = Command::new("cargo")
        .args([
          "run",
          "--bin",
          "uniffi-bindgen-swift",
          "--features",
          "ffi",
          // will grab bindings from aarch64-apple-darwin, not sure if it's the best choice though...
          format!("target/aarch64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "swift",
          "--swift-sources",
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
          "run",
          "--bin",
          "uniffi-bindgen-swift",
          "--features",
          "ffi",
          // will grab bindings from aarch64-apple-darwin, not sure if it's the best choice though...
          format!("target/aarch64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "target/uniffi-xcframework-staging",
          "--headers",
          "--module-name",
          format!("{library_name}FFI").as_ref(),
          "--modulemap",
          "--modulemap-filename",
          "module.modulemap",
        ])
        .spawn()?;

      child.wait()?;

      println!("creating xcframework...");
      let mut child = Command::new("xcodebuild")
        .args([
          "-create-xcframework",
          // iOS simulator
          "-library",
          format!("target/aarch64-x86_64-apple-ios-sim/release/lib{library_name}.a").as_ref(),
          "-headers",
          "target/uniffi-xcframework-staging",
          // macOS
          "-library",
          format!("target/aarch64-x86_64-apple-darwin/release/lib{library_name}.a").as_ref(),
          "-headers",
          "target/uniffi-xcframework-staging",
          // iOS
          "-library",
          format!("target/aarch64-apple-ios/release/lib{library_name}.a").as_ref(),
          "-headers",
          "target/uniffi-xcframework-staging",
          "-output",
          format!("target/{library_name}FFI.xcframework").as_ref(),
        ])
        .spawn()?;

      child.wait()?;

      println!("zipping xcframework...");
      let mut child = Command::new("ditto")
        .args([
          "-c",
          "-k",
          "--sequesterRsrc",
          "--keepParent",
          format!("target/{library_name}FFI.xcframework").as_ref(),
          format!("target/{library_name}FFI.xcframework.zip").as_ref(),
        ])
        .spawn()?;

      child.wait()?;

      println!("applying zip checksum...");
      let output = Command::new("shasum")
        .args([
          "-a",
          "256",
          format!("target/{library_name}FFI.xcframework.zip").as_ref(),
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
          "-o",
          "kotlin/src/androidMain/jniLibs",
          "--manifest-path",
          "Cargo.toml",
          "-t",
          "armeabi-v7a", // armv7-linux-androideabi
          "-t",
          "arm64-v8a", // aarch64-linux-android
          "-t",
          "x86", // i686-linux-android
          "-t",
          "x86_64", // x86_64-linux-android
          "build",
          "--release",
          "--features",
          "ffi",
        ])
        .spawn()?;

      child.wait()?;

      // build debug version for the bindings
      let mut child = Command::new("cargo")
        .args(["build", "--features", "ffi"])
        .spawn()?;

      child.wait()?;

      let mut child = Command::new("cargo")
        .args([
          "run",
          "--bin",
          "uniffi-bindgen",
          "--features",
          "ffi",
          "generate",
          "--library",
          format!(
            "target/debug/lib{}.{}",
            get_library_name()?,
            os::dylib_or_so()
          )
          .as_ref(),
          "--out-dir",
          "kotlin/src/commonMain/kotlin",
          "--language",
          "kotlin",
          "--no-format", // prevent ktlint from formatting the generated code
        ])
        .spawn()?;

      child.wait()?;
    }
    Commands::MakeJS => {
      let _ = std::fs::remove_dir_all("target/wasm-js-staging");

      Command::new("wasm-pack")
        .args([
          "build",
          "--release",
          "--target",
          "web",
          "--out-name",
          "index",
          "--out-dir",
          "target/wasm-js-staging",
        ])
        .spawn()
        .expect("failed to build WASM, make sure wasm-pack is installed")
        .wait()?;

      let js_file_path = current_dir()?.join("target/wasm-js-staging/index.js");
      let js = io::read_file_as_string(js_file_path)?;
      let types_file_path = current_dir()?.join("target/wasm-js-staging/index.d.ts");
      let types = io::read_file_as_string(types_file_path)?;

      println!("\napplying defaultFetcher to JS bindings...");
      println!("=> adding CJS import for defaultFetcher");
      let js = format!(
        "{}{}",
        "const { defaultFetcher } = require(\"@literate.ink/utilities/fetcher\");", js
      );
      println!("=> rewriting every instance of fetcher to make it use defaultFetcher by default");
      let js = js.replace(", fetcher) {", ", fetcher = defaultFetcher) {");

      println!("\ncleaning up js bindings...");
      println!(
        "=> removing {} and {} exports",
        "initSync".bold(),
        "__wbg_init".bold()
      );
      let js = js
        .replace("export { initSync };", "")
        .replace("export default __wbg_init;", "");

      println!("=> removing {} function", "__wbg_init".bold());
      let js = strings::remove_from_until(
        &js,
        "async function __wbg_init",
        "return __wbg_finalize_init(instance, module);\n}",
        true,
      );

      println!(
        "=> removing {} old property",
        "__wbg_init.__wbindgen_wasm_module".bold()
      );
      let js = js.replace("__wbg_init.__wbindgen_wasm_module = module;", "");

      println!("=> removing dead parts in {} function", "initSync".bold());
      let js = strings::remove_from_until(
        &js,
        "if (typeof module !== 'undefined') {",
        "const imports = __wbg_get_imports();",
        false,
      );
      let js = strings::remove_from_until(
        &js,
        "if (!(module instanceof WebAssembly.Module)) {",
        "}\n",
        true,
      );

      println!(
        "=> rewriting module instanciation in {} function",
        "initSync".bold()
      );
      let js = js.replace(
        "new WebAssembly.Instance(module",
        "new WebAssembly.Instance(new WebAssembly.Module(module)",
      );

      println!("\nextracting and removing exports...");
      let mut exports: Vec<String> = Vec::new();

      let class_re = Regex::new(r"export class (\w+)").unwrap();
      let js = class_re
        .replace_all(&js, |caps: &Captures| {
          let class_name = &caps[1];
          println!("=> found class: {}", class_name.bold());
          exports.push(class_name.to_string());
          format!("class {}", class_name)
        })
        .to_string();

      let function_re = Regex::new(r"export function (\w+)").unwrap();
      let js = function_re
        .replace_all(&js, |caps: &Captures| {
          let function_name = &caps[1];
          println!("=> found function: {}", function_name.bold());
          exports.push(function_name.to_string());
          format!("function {}", function_name)
        })
        .to_string();

      let const_re = Regex::new(r"export const (\w+)").unwrap();
      let js = const_re
        .replace_all(&js, |caps: &Captures| {
          let const_name = &caps[1];
          println!("=> found constant: {}", const_name.bold());
          exports.push(const_name.to_string());
          format!("const {}", const_name)
        })
        .to_string();

      println!("\nembeding wasm as base64...");
      let wasm_file_path = current_dir()?.join("target/wasm-js-staging/index_bg.wasm");
      let wasm = io::read_file_as_base64url(wasm_file_path)?;
      let mut js = js;

      js += format!(
        r#"
        {}
        const _code = "{wasm}";
        void initSync(stringToBytes("base64url", _code));
      "#,
        "const { stringToBytes } = require(\"@scure/base\");"
      )
      .as_ref();

      println!(
        "=> wasm base64url length: {}",
        wasm.len().to_string().yellow()
      );

      println!("\nrewriting exports...");
      for export in exports {
        println!("=> exporting: {}", export.bold());
        js += format!("exports.{export}={export};\n").as_ref();
      }

      // add the type import at the top
      println!("\napplying optional and correct type 'fetcher'...");
      let types = format!(
        "{}\n\n{}",
        "import type { Fetcher } from \"@literate.ink/utilities/fetcher\";", types
      );

      // replacing types for `fetcher` argument
      let from = "fetcher: Function)";
      let to = "fetcher?: Fetcher)";
      let types = types.replace("fetcher: Function)", "fetcher?: Fetcher)");
      println!("=> replacing '{}' with '{}'", from.red(), to.green());

      println!("\ncleaning up types...");
      println!("=> removing useless exports");
      let types = strings::remove_from_until(
        &types,
        "export type InitInput",
        "Promise<InitOutput>;",
        true,
      );

      println!("\ncorrecting optional parameters types...");

      // Has been tested against these cases :
      //
      // nip: string | null | undefined
      // nip: Uint8Array | ArrayBuffer | null | undefined
      // (session: Session | null | undefined, nip: string | null | undefined, fetcher?: Fetcher): Promise<Uint8Array>;
      // (session: Session, nip: Uint8Array | ArrayBuffer | null | undefined, fetcher?: Fetcher): Promise<Uint8Array>;
      // (session: Session, nip: Uint8Array | ArrayBuffer | null | undefined, fetcher?: Fetcher): Promise<Uint8Array> | null | undefined;
      // () => string | null | undefined;
      // type Session = {
      //   url: string | null | undefined
      //   url: string | null | undefined;url: string | null | undefined
      //   name: SessionName | SessionNaming | null | undefined,
      //   name: SessionName | SessionNaming | null | undefined
      //   age: number
      // }
      let regex = Regex::new(r"(\b\w+\b): ([^,;\n]*) \| undefined").unwrap();
      let types = regex
        .replace_all(&types, |caps: &Captures| {
          let replacement = format!("{}?: {}", &caps[1], &caps[2]);
          println!(
            "=> replacing '{}' with '{}'",
            &caps[0].red(),
            &replacement.green()
          );
          replacement
        })
        .to_string();

      // cleanup
      let _ = std::fs::remove_dir_all(current_dir()?.join("js"));
      create_dir_all("js")?;

      println!("\noutputting types...");
      let types_file_path = current_dir()?.join("js/index.d.ts");
      io::write_string_to_file(types_file_path.clone(), types)?;
      println!(
        "=> types file path: {}",
        types_file_path.to_str().unwrap().underline()
      );

      println!("\noutputting bindings...");
      let js_file_path = current_dir()?.join("js/index.js");
      io::write_string_to_file(js_file_path.clone(), js)?;
      println!(
        "=> js file path: {}",
        js_file_path.to_str().unwrap().underline()
      );

      println!("\nminifying bindings...");
      let child = Command::new("terser")
        .args([
          "js/index.js",
          "-m", // mangle
          "-c", // compress
          "-o",
          "js/index.js",
        ])
        .spawn();

      if let Ok(mut child) = child {
        child.wait()?;
      } else {
        println!(
          "=> {}",
          "terser not found, skipping minification...".yellow()
        );
      }

      println!("\ndone ! you can now publish the package.");
    }
    Commands::Version => {
      let version = version_to_string(get_current_version()?);
      println!("{version}");
    }
  }

  Ok(())
}
