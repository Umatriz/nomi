use std::path::PathBuf;

use freya::prelude::*;
use nomi_core::loaders::{instance::InstanceBuilder, vanilla::Vanilla};

use crate::theme::colors;

pub fn App(cx: Scope) -> Element {
    use_init_focus(cx);
    let version = use_state(cx, String::new);

    cx.render(rsx! {
        rect {
            rect { width: "100%", height: "50", background: colors::secondary, display: "center", direction: "both",
                label { color: colors::primary, font_size: "20", "🚧The app is WIP🚧" }
            }
            label { "Version: {version}" }
            Input { value: version.get().clone(), onchange: |e| version.set(e) }

            Button {
                onclick: |_| {
                    let v = version.to_string();
                    cx.spawn(async {
                        let instance = InstanceBuilder::new()
                            .version(&v)
                            .game("./minecraft")
                            .libraries("./minecraft/libraries")
                            .version_path(PathBuf::from("./minecraft/versions").join(&v))
                            .instance(async move { Vanilla::new(&v).await })
                            .build()
                            .await
                            .unwrap();
                        instance.download().await.unwrap();
                        tracing::info!("Finished dowloading")
                    })
                },
                label { "Download vanilla {version}" }
            }
        }
    })
}
