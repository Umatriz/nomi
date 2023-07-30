use owo_colors::OwoColorize;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let logs = std::env::current_dir()?.join("logs");

    let _ = std::fs::create_dir_all(logs);

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        // .chain(std::io::stdout())
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] [{}] {}",
                        chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]").yellow(),
                        record.level().blue(),
                        record.target().green(),
                        message
                    ))
                })
                .level(log::LevelFilter::Debug)
                .chain(std::io::stdout()),
        )
        .chain(fern::DateBased::new("logs/", "%Y-%m-%d-nomi.log"))
        .apply()?;
    Ok(())
}
