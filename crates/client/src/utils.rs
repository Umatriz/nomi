use std::{fmt::Display, future::Future, io::Write, sync::mpsc::Sender};

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

pub fn spawn_tokio_future<T, Fut>(tx: Sender<T>, fut: Fut) -> tokio::task::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    tokio::spawn(async move {
        let data = fut.await;
        let _ = tx.send(data);
    })
}

pub fn spawn_future<T, Fut>(tx: Sender<T>, fut: Fut) -> std::thread::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    std::thread::spawn(move || {
        let data = pollster::block_on(fut);
        let _ = tx.send(data);
    })
}
