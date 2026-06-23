use dioxus::{prelude::*, router::RouterConfig};

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

    pub const USER_KEY: Asset = asset!("/assets/user-key.svg");

    pub const EYE: Asset = asset!("/assets/eye.svg");
    pub const PENCIL: Asset = asset!("/assets/pencil.svg");
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static USER: GlobalSignal<Option<api::UserPreview>> = Signal::global(|| None);

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub title: String,
    pub message: Element,
    pub at: std::time::Instant,
}

pub static TOASTS: GlobalSignal<Vec<Toast>> = Signal::global(Vec::new);

pub fn toast(title: String, message: Element) {
    TOASTS.with_mut(|t| {
        t.push(Toast {
            title,
            message,
            at: std::time::Instant::now(),
        })
    });
}

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
    #[end_layout]
    #[layout(AuthLayout)]
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {
            config: || {
                RouterConfig::default()
                    .on_update(|state| {
                        if USER().is_some() {
                            if matches!(state.current(), Route::Login {} | Route::Register {}) {
                                return Some(NavigationTarget::Internal(Route::Saves {}));
                            }
                        } else {
                            if !matches!(state.current(), Route::Login {} | Route::Register {}) {
                                return Some(NavigationTarget::Internal(Route::Login {}));
                            }
                        }

                        None
                    })
            },
        }
        Toaster {}
    }
}

mod saves;
mod versions;

#[component]
fn UserDropdown(user: api::UserPreview) -> Element {
    rsx! {
        div { class: "flex items-center group h-10 w-fit px-4 relative",
            span { class: "text-white", "{user.username}" }

            div { class: "hidden group-hover:block absolute right-0 top-full bg-neutral-700 rounded shadow-lg border border-neutral-500",
                button {
                    class: "px-4 py-2 cursor-pointer hover:underline",
                    onclick: move |_| async move {
                        if let Err(e) = api::logout().await {
                            error!("Error logging out: {}", e);
                        }
                        (*USER.write()) = None;
                    },
                    "Logout"
                }
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    let navigator = use_navigator();
    let user_rsx = if let Some(user) = USER() {
        rsx! {
            UserDropdown { user }
        }
    } else {
        navigator.replace(Route::Login {});
        rsx! {}
    };

    rsx! {
        div { class: "flex justify-between items-center h-10
         bg-neutral-900 text-white",
            div { id: "navbar w-fit h-10",
                Link {
                    class: "px-4 h-10 flex items-center hover:underline",
                    to: Route::Saves {},
                    span { "Saves" }
                }
            }

            {user_rsx}
        }

        Outlet::<Route> {}

    }
}

#[component]
pub fn Toaster() -> Element {
    use_resource(move || async move {
        loop {
            let now = std::time::Instant::now();

            TOASTS.with_mut(|t| {
                t.retain(|toast| now.duration_since(toast.at) < std::time::Duration::from_secs(5));
            });

            time_sleep(100).await;
        }
    });

    rsx! {
        div { class: "fixed bottom-4 left-4 flex flex-col gap-2 z-50",
            for toast in TOASTS() {
                div { class: "bg-neutral-700 text-white p-4 rounded shadow-lg border border-neutral-500",
                    h3 { class: "font-bold", "{toast.title}" }
                    {toast.message}
                }
            }
        }
    }
}

#[cfg(feature = "desktop")]
async fn time_sleep(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

#[cfg(feature = "web")]
async fn time_sleep(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(feature = "server")]
async fn time_sleep(ms: u64) {
    // On the server, we don't need to sleep, as the server doesn't have a UI to update
}

#[component]
fn AuthLayout() -> Element {
    let route = use_route::<Route>();

    let is_login = matches!(route, Route::Login {});

    rsx! {
        div { class: "flex flex-col items-center mt-10",
            div { class: "flex flex-row container justify-center items-center text-2xl border rounded w-fit",
                Link { to: Route::Login {},
                    div {
                        class: if is_login { "bg-neutral-600 " },
                        class: "flex items-center justify-center cursor-pointer hover:underline w-60 h-20 rounded-l",
                        span { "Login" }
                    }
                }
                Link { to: Route::Register {},
                    div {
                        class: if !is_login { "bg-neutral-600 " },
                        class: "flex items-center justify-center cursor-pointer hover:underline w-60 h-20 rounded-r",
                        span { "Register" }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);

    let navigator = use_navigator();

    let mut login_user = use_action(move || async move {
        let usr = match api::login(username(), password()).await {
            Ok(usr) => usr,
            Err(e) => match e {
                ServerFnError::ServerError { message, .. } => {
                    return Ok(Some(message));
                }
                _ => {
                    return Ok(Some("An unknown error occurred".to_string()));
                }
            },
        };
        *USER.write() = Some(usr);
        navigator.replace(Route::Saves {});
        Ok::<Option<String>, ServerFnError>(None)
    });

    let failure_message = login_user.value().and_then(|res| {
        res.ok().and_then(|s| s()).map(|msg| {
            rsx! {
                p { class: "col-span-full text-red-500 mt-2", {msg} }
            }
        })
    });

    rsx! {
        document::Title { "Login" }

        form {
            class: "grid grid-cols-[auto_1fr] gap-2 items-center p-4 container w-120 border border-neutral-500/50 rounded mt-8",
            onsubmit: move |e| {
                e.prevent_default();
                login_user
                    .call()
            },
            label { r#for: "username", "Username:" }
            input {
                id: "username",
                class: INPUT_CLASS,
                placeholder: "Username",
                required: true,
                oninput: move |e| username.set(e.value()),
            }
            label { r#for: "password", "Password:" }
            input {
                id: "password",
                class: INPUT_CLASS,
                r#type: "password",
                placeholder: "Password",
                required: true,
                oninput: move |e| password.set(e.value()),
            }
            {failure_message}
            div { class: "flex justify-end col-span-full mt-2",
                input {
                    r#type: "submit",
                    class: "rounded bg-white text-black px-4 py-2 cursor-pointer",
                    "Login"
                }
            }
        }
    }
}

#[component]
fn Register() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);

    let navigator = use_navigator();

    let mut register = use_action(move || async move {
        if password() != confirm_password() {
            return Ok(Some("Passwords do not match".to_string()));
        }
        let usr = match api::register(username(), password()).await {
            Ok(usr) => usr,
            Err(e) => match e {
                ServerFnError::ServerError { message, .. } => {
                    return Ok(Some(message));
                }
                _ => {
                    return Ok(Some("An unknown error occurred".to_string()));
                }
            },
        };
        *USER.write() = Some(usr);
        navigator.replace(Route::Saves {});
        Ok::<Option<String>, ServerFnError>(None)
    });

    let failure_message = register.value().and_then(|res| {
        res.ok().and_then(|s| s()).map(|msg| {
            rsx! {
                p { class: "col-span-full text-red-500 mt-2", {msg} }
            }
        })
    });

    rsx! {
        document::Title { "Register" }

        form {
            class: "grid grid-cols-[auto_1fr] gap-2 items-center p-4 container w-120 border border-neutral-500/50 rounded mt-8",
            onsubmit: move |e| {
                e.prevent_default();
                register.call();
            },
            label { r#for: "username", "Username:" }
            input {
                id: "username",
                class: INPUT_CLASS,
                placeholder: "Username",
                required: true,
                oninput: move |e| username.set(e.value()),
            }
            label { r#for: "password", "Password:" }
            input {
                id: "password",
                class: INPUT_CLASS,
                r#type: "password",
                placeholder: "Password",
                required: true,
                oninput: move |e| password.set(e.value()),
            }
            label { r#for: "confirm_password", "Confirm Password:" }
            input {
                id: "confirm_password",
                class: INPUT_CLASS,
                r#type: "password",
                placeholder: "Confirm Password",
                required: true,
                oninput: move |e| confirm_password.set(e.value()),
            }
            {failure_message}
            div { class: "flex justify-end col-span-full mt-2",
                input {
                    r#type: "submit",
                    class: "rounded bg-white text-black px-4 py-2 cursor-pointer",
                    "Submit"
                }
            }
        }
    }
}

const DIALOG_CLASS: &str =
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

const INPUT_CLASS: &str = "bg-neutral-700 text-white border border-neutral-500 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-500";
