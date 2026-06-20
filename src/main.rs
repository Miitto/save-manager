use dioxus::{
    html::input::{multiple, required},
    prelude::*,
};

pub mod icons {
    use dioxus::prelude::*;

    pub const TRASH: Asset = asset!("/assets/trash-2.svg");
    pub const DOWNLOAD: Asset = asset!("/assets/download.svg");
    #[cfg(feature = "desktop")]
    pub const INSTALL: Asset = asset!("/assets/hard-drive-download.svg");
    pub const CIRCLE_PLUS: Asset = asset!("/assets/circle-plus.svg");

    pub const CHEVRON_DOWN: Asset = asset!("/assets/chevron-down.svg");
    pub const CHEVRON_UP: Asset = asset!("/assets/chevron-up.svg");
    pub const CHEVRON_UP_DOWN: Asset = asset!("/assets/chevrons-up-down.svg");
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static USER: GlobalSignal<Option<api::UserPreview>> = Signal::global(|| None);

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    api::launch_server(App);
}

use saves::Saves;
use versions::SaveDetails;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Saves {},
    #[route("/save/:id")]
    SaveDetails { id: i32 },
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

mod saves;
mod versions;

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    let user_rsx = if let Some(user) = USER() {
        rsx! { "{user.username}" }
    } else {
        rsx! {
            Link { to: Route::Saves {}, "Login" }
        }
    };

    rsx! {
        div { class: "flex justify-between items-center px-4 py-2 bg-neutral-900 text-white",
            div { id: "navbar",
                Link { to: Route::Saves {}, "Saves" }
            }

            {user_rsx}
        }

        Outlet::<Route> {}
    }
}

#[component]
pub fn Dialog(
    open: Signal<bool>,
    backdrop_color: Option<String>,
    class: Option<String>,
    children: Element,
) -> Element {
    let backdrop_class = match backdrop_color {
        Some(color) => format!("bg-{}", color),
        None => "bg-black/50".to_string(),
    };

    let class = class.unwrap_or_else(|| {
        "bg-neutral-700 p-6 rounded shadow-lg w-96 border border-neutral-500 text-white".to_string()
    });

    rsx! {
        dialog {
            open: open(),
            class: if open() { "flex" },
            class: "items-center justify-center {backdrop_class} z-50 w-screen h-screen top-0 left-0",
            onclick: move |_| open.set(false),
            div { class, onclick: |e| e.stop_propagation(), {children} }
        }
    }
}

#[component]
pub fn ConfirmDialog(
    open: Signal<bool>,
    title: String,
    message: String,
    on_confirm: EventHandler<()>,
) -> Element {
    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-bold mb-4 text-white", {title} }
            p { class: "mb-4 text-white", {message} }

            div { class: "flex justify-between gap-2",
                button {
                    class: "px-4 py-2 bg-gray-400 rounded cursor-pointer hover:bg-gray-500",
                    onclick: move |_| open.set(false),
                    "Cancel"
                }
                button {
                    class: "px-4 py-2 bg-red-500 rounded text-white cursor-pointer hover:bg-red-600",
                    onclick: move |_| {
                        on_confirm.call(());
                        open.set(false);
                    },
                    "Confirm"
                }
            }
        }
    }
}

#[component]
pub fn Input(
    value: Option<String>,
    oninput: Option<EventHandler<String>>,
    placeholder: Option<String>,
    name: Option<String>,
    mul: Option<bool>,
    r#type: Option<String>,
    req: Option<bool>,
) -> Element {
    let placeholder = placeholder.unwrap_or_else(|| "".to_string());

    rsx! {
        input {
            class: "bg-neutral-700 text-white border border-neutral-500 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-500",
            name,
            value,
            placeholder,
            multiple: mul,
            r#type,
            required: req,
            oninput: move |e| {
                if let Some(oninput) = &oninput {
                    oninput.call(e.value());
                }
            },
        }
    }
}
