use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(verbatim_doc_comment)]
///     _   __                _
///    / | / /___  ____ ___  (_)
///   /  |/ / __ \/ __ `__ \/ /
///  / /|  / /_/ / / / / / / /  
/// /_/ |_/\____/_/ /_/ /_/_/   
/// CLI client
pub struct Cli {
    #[arg(long, short = 'g')]
    pub game_dir: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Download {
        name: String,
        version: String,
        #[command(subcommand)]
        loader: Option<Loader>,
    },

    Launch {
        profile_id: String,
    },

    List,
}

#[derive(Subcommand)]
pub enum Loader {
    Fabric {
        #[arg(long, short = 'v')]
        version: Option<String>,
    },
}
