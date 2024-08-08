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

// Helper function to center arbitrary widgets. It works by measuring the width of the widgets after rendering, and
// then using that offset on the next frame.
pub fn centerer(ui: &mut eframe::egui::Ui, add_contents: impl FnOnce(&mut eframe::egui::Ui)) {
    ui.horizontal(|ui| {
        let id = ui.id().with("_centerer");
        let last_width: Option<f32> = ui.memory_mut(|mem| mem.data.get_temp(id));
        if let Some(last_width) = last_width {
            ui.add_space((ui.available_width() - last_width) / 2.0);
        }
        let res = ui
            .scope(|ui| {
                add_contents(ui);
            })
            .response;
        let width = res.rect.width();
        ui.memory_mut(|mem| mem.data.insert_temp(id, width));

        // Repaint if width changed
        match last_width {
            None => ui.ctx().request_repaint(),
            Some(last_width) if last_width != width => ui.ctx().request_repaint(),
            Some(_) => {}
        }
    });
}
