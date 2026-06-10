use api::get_greeting;
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    api::launch_server(App);
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    use api::*;

    let mut login = use_action(login);
    let mut logout = use_action(logout);
    let mut get_username = use_action(get_username);
    let mut permissions = use_action(get_permissions);

    let fetch_new = move |_| async move {
        get_username.call().await;
        permissions.call().await;
    };

    rsx! {
        div {
            class: "flex flex-col",
            button {
                onclick: move |_| async move {login.call().await;}, "Login Test User"
            }

            button {
                onclick: move |_| async move {logout.call().await;}, "Logout"
            }

            button {
                onclick: fetch_new, "Refresh User Info"
            }

            pre { "Logged in: {login.value():?}"}
            pre { "Username: {get_username.value():?}"}
            pre { "Permissions: {permissions.value():?}" }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
        }

        Outlet::<Route> {}
    }
}
