use std::{collections::HashSet, env, fs, path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};
use duct::cmd;
use log::debug;

fn main() -> Result<ExitCode> {
    env_logger::init();

    let source_file_path = PathBuf::from(env::var("ANNO_TARGET")?);
    if source_file_path.extension().unwrap() != "c" {
        eprintln!("Error: Only `.c` files are currently supported");
        return Ok(ExitCode::FAILURE);
    };

    let lines_with_computation = collect_lines(&source_file_path)?;

    let line_count: usize = env::var("ANNO_TARGET_LINES")?.parse()?;
    for i in 0..line_count {
        if lines_with_computation.contains(&(i + 1)) {
            println!("x");
        } else {
            println!(" ");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn collect_lines(source_file_path: &PathBuf) -> Result<HashSet<usize>> {
    // TODO: Change `dbgcov` to only print to stdout by default...?
    let preprocessed_file_path = source_file_path.with_extension("i");

    // Call `dbgcov` to report source code regions with computation
    // ${CC} $(~/Projects/dbgcov/bin/dbgcov-cflags) ${CFLAGS} -std=c99 -E -o example.i example.c
    let cc = env::var("CC").unwrap_or("cc".to_string());
    let dbgcov_cflags = cmd!("dbgcov-cflags")
        .read()
        .with_context(|| format!("Unable to collect flags from `dbgcov-cflags`"))?;
    let mut dbgcov_cflags_split: Vec<&str> = dbgcov_cflags.split_ascii_whitespace().collect();
    let cflags = env::var("CFLAGS").unwrap_or_default();
    let mut cflags_split: Vec<&str> = cflags.split_ascii_whitespace().collect();
    let mut dbgcov_args = Vec::new();
    dbgcov_args.append(&mut dbgcov_cflags_split);
    dbgcov_args.append(&mut cflags_split);
    dbgcov_args.append(&mut vec![
        "-std=c99",
        "-E",
        "-o",
        preprocessed_file_path.to_str().unwrap(),
        source_file_path.to_str().unwrap(),
    ]);
    let dbgcov_command = cmd(cc, &dbgcov_args);
    let dbg_command_debug = format!("{:?}", dbgcov_command);
    dbgcov_command.run().with_context(|| {
        format!(
            "Running `dbgcov` (via {}) to collect regions with computation failed",
            dbg_command_debug,
        )
    })?;

    let report_path = preprocessed_file_path.with_extension("i.dbgcov");
    let regions = fs::read_to_string(&report_path)
        .with_context(|| format!("Unable to read `dbgcov` report ({})", report_path.display()))?;

    // Collect set of lines with computation
    let mut lines_with_computation = HashSet::new();
    for regions_line in regions.lines() {
        // Line format:
        // start as `file:line:column`\t
        // end as `file:line:column`\t
        // kind (e.g. `Computation`)\t
        // expression type
        let mut region_line_parts = regions_line.split('\t');
        let region_start = region_line_parts.next().unwrap();
        let region_end = region_line_parts.next().unwrap();
        let region_kind = region_line_parts.next().unwrap();

        // Ignore non-computation regions
        if region_kind != "Computation" {
            continue;
        }

        let mut region_start_parts = region_start.split(':');
        let region_start_file = region_start_parts.next().unwrap();
        // Ignore regions from other files
        if region_start_file != source_file_path.to_str().unwrap() {
            continue;
        }

        debug!("Matching line: {}", regions_line);
        let region_start_line: usize = region_start_parts.next().unwrap().parse()?;
        let region_end_line: usize = region_end.split(':').nth(1).unwrap().parse()?;
        for line in region_start_line..=region_end_line {
            lines_with_computation.insert(line);
        }
    }

    Ok(lines_with_computation)
}
