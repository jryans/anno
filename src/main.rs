use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use clap::Parser;
use log::debug;
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// File to annotate
    file: PathBuf,
    /// Add an annotation data producer URI
    ///
    /// Several different URI formats are accepted, for example:
    ///
    /// - Producer only
    ///   `producer:`
    /// - Producer with data source
    ///   `producer:/path/to/data/source`
    /// - Producer with data source and additional arguments
    ///   `producer:/path/to/data/source?param=value`
    #[arg(short, long = "producer", id = "PRODUCER", verbatim_doc_comment)]
    producers: Vec<Url>,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    // TODO: Customise parsing to allow producer without trailing `:`
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    debug!("{:?}", cli);

    if cli.producers.is_empty() {
        eprintln!("Warning: No producers, displaying file without annotations");
    }

    // Read the file first to check line count
    let target_content = std::fs::read_to_string(&cli.file).with_context(|| {
        format!(
            "Unable to read file to be annotated ({})",
            cli.file.display()
        )
    })?;
    let line_count = target_content.lines().count();
    debug!("Lines: {}", line_count);

    // TODO: Run producers

    // Write file content with annotations added
    for line in target_content.lines() {
        println!("{}", line);
    }

    Ok(())
}
