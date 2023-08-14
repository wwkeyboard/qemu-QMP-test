use std::path::PathBuf;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    pub path: PathBuf,

    /// Number of times to greet
    #[arg(short, long, default_value_t = true)]
    stream: bool,
}

impl Args {
    pub fn load_from_cli() -> Args {
        Args::parse()
    }
}