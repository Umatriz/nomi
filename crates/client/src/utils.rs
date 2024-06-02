use std::future::Future;

use tokio::sync::mpsc::Sender;

pub fn spawn_tokio_future<T, Fut>(tx: Sender<T>, fut: Fut) -> tokio::task::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    tokio::spawn(async move {
        let data = fut.await;
        let _ = tx.send(data).await;
    })
}

pub fn spawn_future<T, Fut>(tx: std::sync::mpsc::Sender<T>, fut: Fut) -> std::thread::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    std::thread::spawn(move || {
        let data = pollster::block_on(fut);
        let _ = tx.send(data);
    })
}
