use dioxus::prelude::*;

pub trait IntoElement {
    fn into_element(self) -> Element;
}

impl IntoElement for Element {
    fn into_element(self) -> Element {
        self
    }
}

impl IntoElement for &str {
    fn into_element(self) -> Element {
        rsx! { "{self}" }
    }
}

impl IntoElement for String {
    fn into_element(self) -> Element {
        rsx! { "{self}" }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub title: String,
    pub message: Element,
    pub icon: &'static Asset,
}

pub static TOASTS: GlobalSignal<Vec<Toast>> = Signal::global(Vec::new);

pub fn toast(title: impl Into<String>, message: impl IntoElement, icon: &'static Asset) {
    TOASTS.with_mut(|t| {
        t.push(Toast {
            title: title.into(),
            message: message.into_element(),
            icon,
        })
    });
}

#[inline]
pub fn toast_success(title: impl Into<String>, message: impl IntoElement) {
    toast(title, message, &crate::icons::CIRCLE_CHECK);
}

#[inline]
pub fn toast_error(title: impl Into<String>, message: impl IntoElement) {
    toast(title, message, &crate::icons::CIRCLE_X);
}

#[inline]
pub fn toast_info(title: impl Into<String>, message: impl IntoElement) {
    toast(title, message, &crate::icons::INFO);
}

#[component]
pub fn Toaster() -> Element {
    rsx! {
        div { class: "fixed bottom-4 left-4 flex flex-col gap-2 z-50",
            for toast in TOASTS() {
                ToastE { toast }
            }
        }
    }
}

#[component]
fn ToastE(toast: Toast) -> Element {
    let timer = dioxus_sdk_time::use_timeout(std::time::Duration::from_secs(5), move |()| {
        TOASTS.with_mut(|t| t.remove(0));
    });

    use_effect(move || {
        timer.action(());
    });

    rsx! {
        div { class: "bg-neutral-700 text-white p-4 rounded shadow-lg border border-neutral-500",
            div { class: "flex items-center justify-between",
                h3 { class: "font-bold", "{toast.title}" }
                img {
                    class: "invert",
                    width: "20",
                    height: "20",
                    src: *toast.icon,
                }
            }
            hr { class: "my-1" }
            {toast.message}
        }
    }
}
