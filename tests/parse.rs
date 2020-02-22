mod util;

use assert_cmd::prelude::*;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn parse_squares() -> Result<(), Box<dyn std::error::Error>> {
  let movie_path = PathBuf::from("./tests/data/squares/squares.swf");

  let mut cmd = Command::cargo_bin("ofl")?;
  cmd.arg("parse").arg(movie_path);
  cmd.assert().success();
  Ok(())
}
