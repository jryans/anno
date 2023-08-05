use std::env;

use anyhow::Result;

fn main() -> Result<()> {
  let line_count: usize = env::var("ANNO_TARGET_LINES")?.parse()?;
  for i in 0..line_count {
    println!("{}", i + 1);
  }
  Ok(())
}
