use args::Cli;
use clap::Parser;
use commands::process_args;

pub mod args;
pub mod commands;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    process_args(&args).await.unwrap();
}
