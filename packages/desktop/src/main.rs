use dioxus::prelude::*;

mod config;
mod hotkey;
mod launcher;
mod tray;
mod views;

use config::AppConfig;
use views::Home;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut config = use_signal(|| config::load_config());

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "bg-bg-primary text-white font-sans min-h-screen",
            Home {
                config: config,
                on_config_changed: move |new_config: AppConfig| {
                    config.set(new_config);
                },
            }
        }
    }
}
