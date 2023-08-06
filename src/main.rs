use std::{
    cmp::max,
    fs,
    path::PathBuf,
    str::{FromStr, Lines},
};

use anyhow::{Context, Error, Ok, Result};
use clap::Parser;
use duct::cmd;
use log::debug;
use path_absolutize::*;
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
    producers: Vec<Producer>,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    // TODO: Customise parsing to allow producer without trailing `:`
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    debug!("CLI: {:?}", cli);

    if cli.producers.is_empty() {
        eprintln!("Warning: No producers, displaying file without annotations");
    }

    // Read the file first to check line count
    let target_path = cli.file.absolutize()?;
    let target_content = fs::read_to_string(&target_path).with_context(|| {
        format!(
            "Unable to read file to be annotated ({})",
            cli.file.display()
        )
    })?;
    let target_line_count = target_content.lines().count();
    debug!("Lines: {}", target_line_count);

    // Collect output from each producer
    // TODO: Run in parallel
    let mut produced_annotations = Vec::new();
    for producer in &cli.producers {
        debug!("Producer: {:?}", producer);
        let command_name = format!("anno-{}", producer.name());
        let mut command = cmd!(&command_name);
        // TODO: Should we pass both used-entered and absolute versions...?
        command = command.env("ANNO_TARGET", target_path.to_str().unwrap());
        command = command.env("ANNO_TARGET_LINES", target_line_count.to_string());
        command = command.env("ANNO_PRODUCER", producer.name());
        // TODO: Should this be absolute like `ANNO_TARGET`...?
        command = command.env("ANNO_SOURCE", producer.source());
        debug!("Command: {:?}", command);
        let data = command
            .read()
            .with_context(|| format!("Annotation producer `{}` failed", &command_name))?;
        let max_width = data.lines().fold(0, |acc, line| max(acc, line.len()));
        // For initial basic annotation format, ensure we have an annotation for every target line
        assert!(data.lines().count() == target_line_count);
        let annotations = Annotations { data, max_width };
        debug!("Annotations: {:?}", annotations);
        produced_annotations.push(annotations);
    }

    // Write header
    for producer_with_annotations in cli.producers.iter().zip(produced_annotations.iter()) {
        print!(
            "{:width$.width$} | ",
            producer_with_annotations.0.name(),
            width = producer_with_annotations.1.max_width
        );
    }
    println!("");

    // Write file content with annotations added
    let mut produced_table: Vec<(Lines, usize)> = produced_annotations
        .iter()
        .map(|a| (a.data.lines(), a.max_width))
        .collect();
    for line in target_content.lines() {
        for produced_lines in &mut produced_table {
            print!(
                "{:width$} | ",
                produced_lines.0.next().unwrap(),
                width = produced_lines.1
            );
        }
        println!("{}", line);
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Producer(Url);

impl Producer {
    fn parse(input: &str) -> Result<Producer> {
        Ok(Producer(Url::parse(input)?))
    }

    fn name(&self) -> &str {
        self.0.scheme()
    }

    fn source(&self) -> &str {
        self.0.path()
    }
}

impl FromStr for Producer {
    type Err = Error;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        Producer::parse(input)
    }
}

#[derive(Debug)]
struct Annotations {
    /// Raw data from the annotation producer
    data: String,
    /// Maximum width of `data` across all lines
    max_width: usize,
}
