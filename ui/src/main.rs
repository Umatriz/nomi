// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use nomi_core::downloads::{vanilla::Vanilla, version::Version};
use tracing::Level;

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    // launch the dioxus app in a webview
    dioxus_desktop::launch(app);
}

// define a component that renders a div with the text "Hello, world!"
fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        div { "Hello, world!" }

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
    })
}
