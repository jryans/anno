use std::{
    collections::HashSet,
    env,
    fs::{self, ReadDir},
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};
use log::trace;

fn main() -> Result<()> {
    env_logger::init();

    let source_file_path = PathBuf::from(env::var("ANNO_TARGET")?);
    let klee_output_dir_path = env::var("ANNO_SOURCE")?;
    let klee_output_dir = fs::read_dir(&klee_output_dir_path).with_context(|| {
        if klee_output_dir_path.is_empty() {
            format!("Path to KLEE output directory is required")
        } else {
            format!(
                "Unable to open KLEE output directory ({})",
                klee_output_dir_path
            )
        }
    })?;

    let line_count: usize = env::var("ANNO_TARGET_LINES")?.parse()?;
    let covered_lines = collect_covered_lines(klee_output_dir, &source_file_path, line_count)?;

    for i in 0..line_count {
        if covered_lines.contains(&(i + 1)) {
            println!("x");
        } else {
            println!(" ");
        }
    }
    Ok(())
}

fn collect_covered_lines(
    klee_output_dir: ReadDir,
    source_file_path: &PathBuf,
    _line_count: usize,
) -> Result<HashSet<usize>> {
    let mut covered_lines = HashSet::new();

    let source_file_name = source_file_path.file_name().unwrap().to_string_lossy();

    // KLEE output contains a directory for each analysed function
    for entry in klee_output_dir {
        let dir_entry = entry?;
        if !dir_entry.file_type()?.is_dir() {
            continue;
        }

        // Get current function name from directory
        let dir_name = dir_entry.file_name();
        let function = dir_name.to_string_lossy();

        // Read stats file
        let stats_file_path = dir_entry.path().join("run.istats");
        let stats_data = fs::read_to_string(stats_file_path)?;
        let stats_lines = stats_data.lines();

        // Process section
        let mut file_found = false;
        let mut function_found = false;
        for line in stats_lines {
            trace!("Stats line: {}", &line);
            // Look for section of interest
            if line.contains("=") {
                if line.starts_with("ob=") {
                    continue;
                } else if line.starts_with("fl=") {
                    file_found = line == format!("fl={}", source_file_name);
                    trace!("File found: {}", &file_found);
                } else if line.starts_with("fn=") {
                    function_found = line == format!("fn={}", function);
                    trace!("Function found: {}", &file_found);
                } else {
                    return Err(anyhow!("Unexpected line: {}", line));
                }
                continue;
            }
            if file_found && function_found {
                // Examine cells of interest
                let mut line_parts = line.split(" ");
                // let asm_line: usize = line_parts.next().unwrap().parse()?;
                let src_line: usize = line_parts.nth(1).unwrap().parse()?;
                let covered_str = line_parts.next().unwrap();
                let covered = covered_str == "1";
                if covered {
                    trace!("Covered src line: {}", &src_line);
                    covered_lines.insert(src_line);
                }
            }
        }
    }

    Ok(covered_lines)
}
