use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn producer_only() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("anno")?;
  cmd.args(["example.c", "-p", "computable-expressions:"]);
  cmd.assert().success();
  Ok(())
}

#[test]
fn producer_with_source() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("anno")?;
  cmd.args(["example.c", "-p", "dwarf-line-table:/example.dwarf"]);
  cmd.assert().success();
  Ok(())
}

#[test]
fn producer_with_source_and_params() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("anno")?;
  cmd.args(["example.c", "-p", "dwarf-line-table:/example.dwarf?function=bob"]);
  cmd.assert().success();
  Ok(())
}
