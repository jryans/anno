use std::{env, fs, path::PathBuf, process::ExitCode};

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

    let line_count: usize = env::var("ANNO_TARGET_LINES")?.parse()?;
    let defined_variables_per_line =
        collect_defined_variables_per_line(&source_file_path, line_count)?;

    for i in 0..line_count {
        let defined_variables = &defined_variables_per_line[i];
        if defined_variables.is_empty() {
            println!(" ");
            continue;
        }
        println!("{}", defined_variables.join(" "));
    }
    Ok(ExitCode::SUCCESS)
}

fn collect_defined_variables_per_line(
    source_file_path: &PathBuf,
    line_count: usize,
) -> Result<Vec<Vec<String>>> {
    // TODO: Change `dbgcov` to only print to stdout by default...?
    let preprocessed_file_path = source_file_path.with_extension("i");

    // Call `dbgcov` to report source code variable definition regions
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
            "Running `dbgcov` (via {}) to collect variable definition regions failed",
            dbg_command_debug,
        )
    })?;

    let report_path = preprocessed_file_path.with_extension("i.dbgcov");
    let regions = fs::read_to_string(&report_path)
        .with_context(|| format!("Unable to read `dbgcov` report ({})", report_path.display()))?;

    // Collect defined variables for each line
    let mut defined_variables_per_line: Vec<Vec<String>> = Vec::new();
    defined_variables_per_line.resize_with(line_count, Default::default);
    for regions_line in regions.lines() {
        // Line format:
        // start as `file:line:column`\t
        // end as `file:line:column`\t
        // kind (e.g. `MustBeDefined`)\t
        // variable as `<function>, <variable>, decl <file>:<line>, unit <file>`
        let mut region_line_parts = regions_line.split('\t');
        let region_start = region_line_parts.next().unwrap();
        let region_end = region_line_parts.next().unwrap();
        let region_kind = region_line_parts.next().unwrap();
        let variable_description = region_line_parts.next().unwrap();

        // Ignore non-definition regions
        if region_kind != "MustBeDefined" {
            continue;
        }

        let mut region_start_parts = region_start.split(':');
        let region_start_file = region_start_parts.next().unwrap();
        // Ignore regions from other files
        if region_start_file != source_file_path.to_str().unwrap() {
            continue;
        }

        let mut variable_description_parts = variable_description.split(", ");
        let variable_name = variable_description_parts.nth(1).unwrap();

        debug!("Matching line: {}", regions_line);
        // Lines are 1-based
        let region_start_line: usize = region_start_parts.next().unwrap().parse()?;
        let region_end_line: usize = region_end.split(':').nth(1).unwrap().parse()?;
        for line in region_start_line..=region_end_line {
            let defined_variables = &mut defined_variables_per_line[line - 1];
            defined_variables.push(variable_name.to_string());
        }
    }

    Ok(defined_variables_per_line)
}
