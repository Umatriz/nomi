#![allow(non_snake_case)]
use dioxus::prelude::*;
use nomi_core::{downloads::version::DownloadVersion, loaders::vanilla::Vanilla};
use tracing::Level;

pub mod components;

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

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
