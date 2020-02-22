mod util;

use crate::util::assert_same_directory_content;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn dump() -> Result<(), Box<dyn std::error::Error>> {
  let root_dir = TempDir::new()?;

  let input_swf = root_dir.path().join("squares.swf");
  std::fs::copy("./tests/data/squares/squares.swf", &input_swf).unwrap();

  let output_dir = root_dir.path().join("squares");
  std::fs::create_dir(&output_dir).unwrap();

  let mut cmd = Command::cargo_bin("ofl")?;
  cmd.arg("dump").arg(input_swf).arg(&output_dir);
  cmd.assert().success();

  let expected_output_dir: &Path = Path::new("./tests/data/squares/dump");

  assert_same_directory_content(&output_dir, expected_output_dir);

  Ok(())
}

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("ofl")?;
  cmd.arg("dump").arg("missing.swf").arg("missing");
  cmd.assert().failure().stderr(predicate::str::contains(
    "Aborting because the parents of the output directory do not exist.",
  ));

  Ok(())
}
