use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-row [&>a]:text-white [&>a]:mr-5 [&>a]:no-underline [&>a]:transition-colors [&>a]:duration-200 hover:[&>a]:cursor-pointer hover:[&>a]:text-accent",
            {children}
        }
    }
}
