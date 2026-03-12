use dioxus::prelude::*;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div { class: "w-[360px] mx-auto mt-[50px] bg-bg-card p-5 rounded-[10px]",
            h4 { class: "m-0 mb-[15px]", "ServerFn Echo" }
            input {
                class: "border-0 border-b border-white bg-transparent text-white transition-colors duration-200 outline-none block pb-[5px] w-full focus:border-b-accent-focus",
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    let data = api::echo(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p { class: "mt-5 ml-auto",
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}
