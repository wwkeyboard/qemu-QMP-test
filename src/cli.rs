use std::path::PathBuf;

use clap::{Parser, Subcommand};

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

    #[command(subcommand)]
    command: Option<Commands>,
}

impl Args {
    pub fn load_from_cli() -> Args {
        Args::parse()
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// sends a single command on the socket
    Send {
        /// command to send
        #[arg(short, long)]
        payload: String,
    }
}