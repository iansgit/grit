use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "grit", about = "A git implementation in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new repository
    Init {
        /// Directory to initialize (defaults to current directory)
        path: Option<std::path::PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { path } => {
            let path = path.unwrap_or_else(|| std::path::PathBuf::from("."));
            grit::commands::init::run(&path)
        }
    }
}
