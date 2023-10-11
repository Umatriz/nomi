#![allow(non_snake_case)]
use app::App;
use freya::prelude::*;
use tracing::Level;
use tracing_subscriber::{
    filter::filter_fn,
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
    Layer as LayerFilterExt, Registry,
};

use crate::theme::colors;

mod app;
pub mod components;
mod theme;

fn main() {
    let file_appender = tracing_appender::rolling::hourly("./logs", "nomi.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let registry = Registry::default()
        .with(
            Layer::default()
                .with_writer(non_blocking.with_max_level(Level::INFO))
                .with_ansi(false)
                .compact(),
        )
        .with(
            Layer::default()
                .with_writer(std::io::stdout.with_max_level(Level::DEBUG))
                .pretty()
                .with_filter(filter_fn(|metadata| {
                    if !metadata.target().contains("nomi") && metadata.level() == &Level::DEBUG {
                        return false;
                    }
                    true
                })),
        );
    tracing::subscriber::set_global_default(registry).unwrap();
    launch(Main);
}
fn Main(cx: Scope) -> Element {
    cx.render(rsx! {
        ThemeProvider { theme: theme::NOMI_THEME_LIGHT, App {} }
    })
}
