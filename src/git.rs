use anyhow::{Ok, Result};
use colored::Colorize;
use std::process::{Command, Output};

pub fn run(args: &[&str]) -> Result<Output> {
  let log = format!("+> git {}", args.join(" "));
  println!("{}", log.bright_black());

  let output = Command::new("git").args(args).output()?;

  Ok(output)
}

/// sets the user to github actions bot.
pub fn configure() -> Result<()> {
  run(&["config", "user.name", "github-actions[bot]"])?;
  run(&[
    "config",
    "user.email",
    "github-actions[bot]@users.noreply.github.com",
  ])?;

  Ok(())
}
