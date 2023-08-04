use std::path::PathBuf;

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
}

fn main() {
    env_logger::init();

    // TODO: Customise parsing to allow producer without trailing `:`
    let cli = Cli::parse();
    debug!("{:?}", cli);
}
