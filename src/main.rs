mod versionning;
use versionning::{bump_version, get_current_version, version_to_string, kotlin, swift, rust};

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    #[arg(value_enum)]
    bump_type: BumpType
  }
}

#[derive(Debug, ValueEnum, Copy, Clone, PartialEq, Eq)]
enum BumpType {
  // indexes in the version array
  Major = 0,
  Minor = 1,
  Patch = 2,
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
      println!("from v{} to v{}", old_version, new_version);

      // apply to configuration files
      println!("applying to rust...");
      rust::apply_version(&old_version, &new_version)?;
      println!("applying to kotlin...");
      kotlin::apply_version(&old_version, &new_version)?;
      println!("applying to swift...");
      println!("note this is not going to update the checksum of FFI, it should be done later.");
      swift::apply_version(&old_version, &new_version)?;
    }
  }

  Ok(())
}
