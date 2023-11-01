use std::{fmt::Display, io::Write};

pub trait Crash<T> {
    fn crash(self) -> T;
}

impl<T, E> Crash<T> for Result<T, E>
where
    E: Display + Send + Sync + 'static,
{
    fn crash(self) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                let mut file = std::fs::File::create("./CRASH_REPORT.txt").unwrap();
                let _ = file.write_all(
                    format!("Nomi crashed with the following error:\n{}", err).as_bytes(),
                );
                std::process::exit(1);
            }
        }
    }
}

impl<T> Crash<T> for Option<T> {
    fn crash(self) -> T {
        match self {
            Some(value) => value,
            None => {
                let mut file = std::fs::File::create("./CRASH_REPORT.txt").unwrap();
                let _ = file.write_all(
                    "Nomi crashed with the following error:\nValue is None"
                        .to_string()
                        .as_bytes(),
                );
                std::process::exit(1);
            }
        }
    }
}
