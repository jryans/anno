use std::{
    borrow::{Borrow, Cow},
    env, fs, path::{PathBuf, Path},
};

use anyhow::{Context, Result};
use debuginfo_quality::{evaluate_info, Stats};
use linked_hash_set::LinkedHashSet;
use log::{debug, trace};
use object::{Object, ObjectSection};
use path_absolutize::Absolutize;
use typed_arena::Arena;

fn main() -> Result<()> {
    env_logger::init();

    let source_file_path = PathBuf::from(env::var("ANNO_TARGET")?);
    let debug_info_path = env::var("ANNO_SOURCE")?;
    let file = fs::File::open(&debug_info_path).with_context(|| {
        if debug_info_path.is_empty() {
            format!("Path to debug info is required")
        } else {
            format!("Unable to open debug info ({})", debug_info_path)
        }
    })?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;

    let line_count: usize = env::var("ANNO_TARGET_LINES")?.parse()?;
    let variable_locations = collect_variable_locations(&object);
    let defined_variables_per_line =
        defined_variables_per_line(&variable_locations, &source_file_path, line_count);

    for i in 0..line_count {
        let defined_variables = &defined_variables_per_line[i];
        if defined_variables.is_empty() {
            println!(" ");
            continue;
        }
        let collected_variables = defined_variables.iter().cloned().collect::<Vec<_>>();
        println!("{}", collected_variables.join(" "));
    }
    Ok(())
}

// Adapted from debuginfo-quality/src/main.rs
fn collect_variable_locations(file: &object::File) -> Stats {
    fn load_section<'a, 'file, 'input, S>(
        arena: &'a Arena<Cow<'file, [u8]>>,
        file: &'file object::File<'input>,
    ) -> S
    where
        S: gimli::Section<gimli::EndianSlice<'a, gimli::LittleEndian>>,
        'file: 'input,
        'a: 'file,
    {
        let data = match file.section_by_name(S::section_name()) {
            Some(ref section) => section
                .uncompressed_data()
                .unwrap_or(Cow::Borrowed(&[][..])),
            None => Cow::Borrowed(&[][..]),
        };
        let data_ref = (*arena.alloc(data)).borrow();
        S::from(gimli::EndianSlice::new(data_ref, gimli::LittleEndian))
    }

    let mut opt = debuginfo_quality::Opt::default();
    opt.variables = true;

    let mut stats = Stats::new(opt.clone());

    let arena = Arena::new();

    let debug_abbrev = &load_section(&arena, file);
    let debug_info = &load_section(&arena, file);
    let debug_ranges = load_section(&arena, file);
    let debug_rnglists = load_section(&arena, file);
    let rnglists = &gimli::RangeLists::new(debug_ranges, debug_rnglists).unwrap();
    let debug_str = &load_section(&arena, file);
    let debug_loc = load_section(&arena, file);
    let debug_loclists = load_section(&arena, file);
    let loclists = &gimli::LocationLists::new(debug_loc, debug_loclists).unwrap();
    let debug_line = &load_section(&arena, file);

    evaluate_info(
        debug_info,
        debug_abbrev,
        debug_str,
        rnglists,
        loclists,
        debug_line,
        stats.opt.no_entry_value,
        stats.opt.no_parameter_ref,
        &mut stats,
    );

    stats
}

fn defined_variables_per_line(
    variable_locations: &Stats,
    source_file_path: &PathBuf,
    line_count: usize,
) -> Vec<LinkedHashSet<String>> {
    let mut defined_variables_per_line: Vec<LinkedHashSet<String>> = Vec::new();
    defined_variables_per_line.resize_with(line_count, Default::default);

    for func in &variable_locations.output {
        for var in &func.variables {
            // Some paths are already absolute, others are relative to compilation
            let mut decl_file_path = if Path::new(&var.decl_dir).is_absolute() {
                Path::new(&var.decl_dir).join(&var.decl_file)
            } else {
                Path::new(&func.unit_dir)
                    .join(&var.decl_dir)
                    .join(&var.decl_file)
            };
            decl_file_path = decl_file_path.absolutize().unwrap().to_path_buf();
            trace!("Var: {}, Decl path: {}", &var.name, decl_file_path.display());
            // Skip variables from other source files
            if decl_file_path != *source_file_path {
                continue;
            }
            debug!("{}", &func.name);
            for inline in &var.inlines {
                debug!(", {}", &inline);
            }
            debug!(
                ", {}, decl {}:{}",
                &var.name, &var.decl_file, &var.decl_line
            );
            debug!("Source line set: {:?}", &var.extra.source_line_set_covered);
            // Lines are 1-based
            for line in &var.extra.source_line_set_covered {
                let defined_variables = &mut defined_variables_per_line[(line - 1) as usize];
                defined_variables.insert(var.name.clone());
            }
        }
    }

    defined_variables_per_line
}
