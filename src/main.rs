use std::path::PathBuf;

use clap::Parser;

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

    println!("{:?}", cli);
}
