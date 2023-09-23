#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct ButtonProps {
    name: String,
}

pub fn Button(cx: Scope<ButtonProps>) -> Element {
    cx.render(rsx! {
        style { include_str!("./button.css") }
        button { class: "button", cx.props.name.clone() }
    })
}
