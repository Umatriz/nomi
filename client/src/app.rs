use freya::prelude::*;
use nomi_core::loaders::{instance::InstanceBuilder, vanilla::Vanilla};

use crate::theme::colors;

pub fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        rect {
            rect { width: "100%", height: "50", background: colors::secondary, display: "center", direction: "both",
                label { color: colors::primary, font_size: "20", "🚧The app is WIP🚧" }
            }

            Button {
                onclick: |_| {
                    cx.spawn(async {
                        let instance = InstanceBuilder::new()
                            .version("1.18.2")
                            .game("./minecraft")
                            .libraries("./minecraft/libraries")
                            .version_path("./minecraft/versions/1.18.2")
                            .instance(async { Vanilla::new("1.18.2").await })
                            .build()
                            .await
                            .unwrap();
                        instance.download().await.unwrap();
                        tracing::info!("Finished dowloading {}", "1.18.2")
                    })
                },
                label { "Download vanilla 1.18.2" }
            }
        }
    })
}
