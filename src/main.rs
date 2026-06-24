use dioxus::prelude::*;

mod auth;
mod components;
mod saves;
mod versions;

pub use components::*;

#[cfg(feature = "desktop")]
mod file_management;

pub mod icons {
    use dioxus::prelude::*;

    pub const TRASH: Asset = asset!("/assets/trash-2.svg");
    pub const DOWNLOAD: Asset = asset!("/assets/download.svg");
    #[cfg(feature = "desktop")]
    pub const INSTALL: Asset = asset!("/assets/hard-drive-download.svg");
    pub const CIRCLE_PLUS: Asset = asset!("/assets/circle-plus.svg");
    pub const CIRCLE_CHECK: Asset = asset!("/assets/circle-check-big.svg");
    pub const CIRCLE_X: Asset = asset!("/assets/circle-x.svg");
    pub const INFO: Asset = asset!("/assets/info.svg");

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

#[cfg(not(debug_assertions))]
const DEFAULT_SERVER_URL: &str = "https://saves.miitto.dev";

#[cfg(not(debug_assertions))]
static SERVER_URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn main() {
    dioxus_cookie::init();

    #[cfg(feature = "web")]
    dioxus::launch(App);

    #[cfg(feature = "desktop")]
    {
        #[cfg(not(debug_assertions))]
        {
            _ = SERVER_URL.set(
                std::env::var("SERVER_URL")
                    .ok()
                    .or_else(|| {
                        let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();

                        let server_url_file = exe_dir.join("server_url.txt");
                        if server_url_file.exists() {
                            std::fs::read_to_string(server_url_file).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| DEFAULT_SERVER_URL.to_string()),
            );
            dioxus::fullstack::set_server_url(SERVER_URL.get().expect("Server URL not set??"));
        }
        dioxus::LaunchBuilder::new()
            .with_cfg(dioxus::desktop::Config::new().with_menu(None))
            .launch(App);
    }

    #[cfg(feature = "server")]
    api::launch_server(App);
}

use auth::{AuthLayout, Login, Register};
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
        Router::<Route> {}
        Toaster {}
    }
}

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
        rsx! {}
    };

    let mut update_user = use_action(move || async move {
        if let Ok(usr) = api::get_user().await {
            (*USER.write()) = Some(usr);
        } else {
            (*USER.write()) = None;
        }

        Ok(()) as Result<(), ServerFnError>
    });

    use_effect(move || {
        update_user.call();
    });

    use_effect(move || {
        if update_user.value().is_some() && USER().is_none() {
            warn!("User is not logged in, redirecting to login page");
            navigator.replace(Route::Login {});
        }
    });

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
