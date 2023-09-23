#![allow(non_snake_case)]
use dioxus::prelude::*;
use nomi_core::{downloads::version::DownloadVersion, loaders::vanilla::Vanilla};
use tracing::Level;
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
    Registry,
};

pub mod components;

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
                .pretty(),
        );

    tracing::subscriber::set_global_default(registry).unwrap();

    dioxus_desktop::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            style { include_str!("./style.css") }
            div { class: "test", "Hello, world!" }

            button {
                onclick: |_| {
                    cx.spawn(async {
                        let version = Vanilla::new("1.18.2").await.unwrap();
                        version.download("./minecraft").await.unwrap();
                        println!("FIN")
                    })
                },
                "Download libs"
            }
        }
    })
}
