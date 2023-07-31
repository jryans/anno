use std::path::PathBuf;

use clap::{CommandFactory, Parser, error::ErrorKind};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// File to annotate
    file: PathBuf,
    /// Add an annotation data producer
    #[arg(short, long = "producer", id = "PRODUCER")]
    producers: Vec<String>,
    /// Add an annotation data source
    #[arg(short, long = "source", id = "SOURCE")]
    sources: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let mut command = Cli::command();

    println!("{:?}", cli);

    if cli.producers.len() != cli.sources.len() {
        command.error(
            ErrorKind::WrongNumberOfValues,
            "Expected matching number of producers and sources",
        ).exit()
    }
}
