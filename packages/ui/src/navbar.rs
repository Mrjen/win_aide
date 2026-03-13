use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between px-5 py-3 bg-bg-card border-b border-border-default",
            {children}
        }
    }
}
