use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between px-4 py-3 border-b border-gray-700",
            {children}
        }
    }
}
