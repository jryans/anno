use std::{borrow, collections::HashSet, env, fs, path};

use anyhow::{Context, Result};
use log::{debug, trace};
use object::{Object, ObjectSection};

fn main() -> Result<()> {
    env_logger::init();

    let source_file_path = env::var("ANNO_TARGET")?;
    let debug_info_path = env::var("ANNO_SOURCE")?;
    let file = fs::File::open(&debug_info_path)
        .with_context(|| {
            if debug_info_path.is_empty() {
                format!("Path to debug info is required")
            } else {
                format!("Unable to open debug info ({})", debug_info_path)
            }
        })?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    let lines_present = collect_lines(&object, endian, &source_file_path)?;

    let line_count: usize = env::var("ANNO_TARGET_LINES")?.parse()?;
    for i in 0..line_count {
        if lines_present.contains(&((i + 1) as u64)) {
            println!("x");
        } else {
            println!(" ");
        }
    }
    Ok(())
}

// Adapted from https://github.com/gimli-rs/gimli/blob/master/examples/simple_line.rs
fn collect_lines(
    object: &object::File,
    endian: gimli::RunTimeEndian,
    source_file_path: &str,
) -> Result<HashSet<u64>> {
    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
        match object.section_by_name(id.name()) {
            Some(ref section) => Ok(section
                .uncompressed_data()
                .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }
    };

    // Load all of the sections.
    let dwarf_cow = gimli::Dwarf::load(&load_section)?;

    // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
    let borrow_section: &dyn for<'a> Fn(
        &'a borrow::Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(&*section, endian);

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_cow.borrow(&borrow_section);

    // Iterate over the compilation units.
    let mut lines_present = HashSet::new();
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        trace!(
            "Line number info for unit at <.debug_info+0x{:x}>",
            header.offset().as_debug_info_offset().unwrap().0
        );
        let unit = dwarf.unit(header)?;

        // Get the line program for the compilation unit.
        if let Some(program) = unit.line_program.clone() {
            let comp_dir = if let Some(ref dir) = unit.comp_dir {
                path::PathBuf::from(dir.to_string_lossy().into_owned())
            } else {
                path::PathBuf::new()
            };

            // Iterate over the line program rows.
            let mut rows = program.rows();
            while let Some((header, row)) = rows.next_row()? {
                if row.end_sequence() {
                    // End of sequence indicates a possible gap in addresses.
                    trace!("{:x} end-sequence", row.address());
                } else {
                    // Determine the path. Real applications should cache this for performance.
                    let mut path = path::PathBuf::new();
                    if let Some(file) = row.file(header) {
                        path = comp_dir.clone();

                        // The directory index 0 is defined to correspond to the compilation unit directory.
                        if file.directory_index() != 0 {
                            if let Some(dir) = file.directory(header) {
                                path.push(
                                    dwarf.attr_string(&unit, dir)?.to_string_lossy().as_ref(),
                                );
                            }
                        }

                        path.push(
                            dwarf
                                .attr_string(&unit, file.path_name())?
                                .to_string_lossy()
                                .as_ref(),
                        );
                    }

                    // Skip if path doesn't match source file path
                    if path.to_str().unwrap() != source_file_path {
                        continue;
                    }

                    // Determine line/column. DWARF line/column is never 0, so we use that
                    // but other applications may want to display this differently.
                    let line = match row.line() {
                        Some(line) => line.get(),
                        None => 0,
                    };
                    let column = match row.column() {
                        gimli::ColumnType::LeftEdge => 0,
                        gimli::ColumnType::Column(column) => column.get(),
                    };

                    debug!("{:x} {}:{}:{}", row.address(), path.display(), line, column);

                    // Add line to set of present lines
                    lines_present.insert(line);
                }
            }
        }
    }
    Ok(lines_present)
}
