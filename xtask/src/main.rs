use args::Xtask;
use clap::Parser;

pub mod args;
pub mod config;
pub mod zip_dir;

pub type DynError = Box<dyn std::error::Error>;

fn main() {
    let args = Xtask::parse();
    dbg!(&args);
    if let Err(e) = process_args(args) {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn process_args(args: Xtask) -> Result<(), DynError> {
    match args.command {
        args::Commands::Build { zip, move_files } => args.build_release(zip, move_files)?,
    }
    Ok(())
}
