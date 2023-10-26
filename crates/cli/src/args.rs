use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    /// Download version
    Download {
        /// Profile name
        name: String,
        /// Game version
        version: String,
        /// Loader
        #[command(subcommand)]
        loader: Option<Loader>,
    },
    /// Launch game from profile
    Launch { profile_id: i32 },
    /// Create config with username
    Register {
        username: String,
        #[arg(long, short)]
        access_token: Option<String>,
        #[arg(long, short)]
        java_bin: Option<PathBuf>,
        #[arg(long, short)]
        uuid: Option<String>,
    },
    /// Show list of existing profiles
    List,
}

#[derive(Subcommand)]
pub enum Loader {
    Fabric {
        #[arg(long, short = 'v')]
        version: Option<String>,
    },
}
