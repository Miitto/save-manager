use dioxus::prelude::*;

pub const DIALOG_CLASS: &str =
    "bg-neutral-700 p-6 rounded shadow-lg w-96 border border-neutral-500 text-white";

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

    let class = class.unwrap_or_else(|| DIALOG_CLASS.to_string());

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
