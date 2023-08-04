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

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    // TODO: Customise parsing to allow producer without trailing `:`
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    debug!("{:?}", cli);
}
