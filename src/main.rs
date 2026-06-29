use dioxus::prelude::*;

mod auth;
mod components;
mod saves;
mod versions;

#[cfg(feature = "desktop")]
mod desktop;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static USER: GlobalSignal<Option<api::UserPreview>> = Signal::global(|| None);

#[cfg(not(debug_assertions))]
const DEFAULT_SERVER_URL: &str = "https://saves.miitto.dev";

#[cfg(not(debug_assertions))]
static SERVER_URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub mod icons {
    pub use dioxus_icons::IconSize;
    pub use dioxus_icons::lucide::*;
}

pub mod prelude {
    pub(crate) use crate::components::*;
    pub use crate::{USER, icons, icons::IconSize};
    pub use dioxus::{fullstack::Loader, prelude::*};
    pub use dioxus_primitives::toast::{ToastOptions, use_toast};
}

use prelude::*;

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
    #[layout(Nav)]
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

        ToastProvider { Router::<Route> {} }
    }
}

#[component]
fn UserDropdown(user: api::UserPreview) -> Element {
    rsx! {
        div { class: "flex items-center group h-10 w-fit px-4 relative z-50",
            span { class: "text-white", "{user.username}" }

            div { class: "hidden group-hover:block absolute right-0 top-full bg-neutral-700 rounded shadow-lg border border-neutral-500",
                Button {
                    variant: ButtonVariant::Ghost,
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
fn Nav() -> Element {
    let navigator = use_navigator();
    let user_rsx = USER().map(|u| rsx! { "{u.username}" });

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
        div {
            Navbar { class: "justify-between",
                NavbarItem {
                    index: 0usize,
                    value: "saves".to_string(),
                    to: Route::Saves {},
                    "Saves"
                }
                NavbarNav { index: 1usize,
                    NavbarTrigger { {user_rsx} }
                    NavbarContent { "data-float": "right",
                        NavbarItem {
                            index: 0usize,
                            value: "logout".to_string(),
                            to: Route::Login {},
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

        if USER().is_some() {
            Outlet::<Route> {}
        }
    }
}
