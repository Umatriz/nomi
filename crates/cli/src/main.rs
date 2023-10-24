use args::Cli;
use clap::Parser;
use commands::process_args;

pub mod args;
pub mod commands;
pub mod error;

#[tokio::main]
async fn main() {
    let sub = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(sub).unwrap();

    let args = Cli::parse();
    process_args(&args).await.unwrap();
}
