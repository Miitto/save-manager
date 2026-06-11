use dioxus::prelude::*;

const TRASH_ICO: Asset = asset!("/assets/trash-2.svg");
const DOWNLOAD_ICO: Asset = asset!("/assets/download.svg");
#[cfg(feature = "desktop")]
const INSTALL_ICO: Asset = asset!("/assets/hard-drive-download.svg");
const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

static USER: GlobalSignal<Option<api::UserPreview>> = Signal::global(|| None);

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

#[component]
fn Home() -> Element {
    let mut login_user = use_action(move || async move {
        let usr = api::login().await?;
        *USER.write() = Some(usr);
        Ok::<(), ServerFnError>(())
    });
    let mut logout_user = use_action(move || async move {
        api::logout().await?;
        *USER.write() = None;
        Ok::<(), ServerFnError>(())
    });

    let username = use_memo(move || {
        USER()
            .map(|u| u.username.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    });

    let saves = use_server_future(|| {
        _ = USER(); // Subscribe to the signal
        async move {
            debug!("Getting saves for user: {:?}", USER());
            if let Some(user) = USER() {
                api::get_user_saves(user.id).await.unwrap_or_else(|e| {
                    debug!("Error fetching saves: {:?}", e);
                    Vec::<api::Save>::new()
                })
            } else {
                std::future::ready(Vec::<api::Save>::new()).await
            }
        }
    })?()
    .unwrap_or_else(Vec::<api::Save>::new);

    rsx! {
        document::Title {"Save Manager"}

        div {
            class: "flex flex-col",
            button {
                onclick: move |_| async move {login_user.call().await;}, "Login Test User"
            }

            button {
                onclick: move |_| async move {logout_user.call().await;}, "Logout"
            }

            pre { "Logged in: {login_user.value():?}"}
            pre { "Username: {username}"}

            SaveList { saves }
        }

    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    let user_rsx = if let Some(user) = USER() {
        rsx! { "{user.username}" }
    } else {
        rsx! {
            Link {
                to: Route::Home {}, // TODO: route to login page
                "Login"
            }
        }
    };

    rsx! {
        div {
            class: "flex justify-between items-center px-4 py-2 bg-neutral-900 text-white",
            div {
                id: "navbar",
                Link {
                    to: Route::Home {},
                    "Home"
                }
            }

            {user_rsx}
        }

        Outlet::<Route> {}
    }
}

#[component]
fn SaveList(saves: ReadSignal<Vec<api::Save>>) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-[1fr_auto] gap-x-4 border-b border-neutral-500 mb-2",
            div {
                class: "font-bold grid grid-cols-subgrid col-span-2 px-4 py-2 border-b border-neutral-500",
                span { "Name" }
                span { "Versions" }
            }
            {saves().into_iter().map(|save| rsx!(SaveRow { save }))}
        }
    }
}

#[component]
fn SaveRow(save: api::Save) -> Element {
    rsx! {
        Link {
            to: Route::SaveDetails {id: save.id},
            class: "grid grid-cols-subgrid col-span-2 py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700",

            span { "{save.name}" }
            span { class: "text-center", "{save.version_count}" }
        }
    }
}

#[component]
fn SaveDetails(id: i32) -> Element {
    let save = use_server_future(move || api::get_save_details(id))?().unwrap();
    let save_versions = use_server_future(move || api::get_save_versions(id))?().unwrap();

    let modify = use_server_future(move || {
        _ = USER();
        async move { api::can_modify_save(id).await }
    })?()
    .and_then(|r| r.ok())
    .unwrap_or(false);

    let save_versions = match save_versions {
        Ok(versions) => {
            rsx! {
                VersionList { versions, modify }
            }
        }
        Err(_) => {
            rsx! {
                p {
                    "Failed to load versions"
                }
            }
        }
    };

    match save {
        Ok(save) => rsx! {
            document::Title {"{save.name}"}

            div {
                class: "flex flex-col",
                div {
                    class: "flex flex-row justify-between items-center p-4",
                    h1 { class: "text-4xl font-bold", "{save.name}" }
                    div {
                        class: "flex flex-row gap-2 items-center",
                    p { span { class: "font-bold", "{save.version_count}"} " version(s)" }
                    button {
                        class: "px-4 py-2 text-black bg-emerald-400 rounded hover:bg-green-300 hover:cursor-pointer",
                        "New"
                    }
                    }
                }

                hr {class: "my-1"}

                {save_versions}
            }
        },
        Err(e) => rsx! {
            div {
                class: "p-4",
                h2 { "Error loading save details" }
                p { "{e}" }
            }
        },
    }
}

#[component]
fn VersionList(versions: ReadSignal<Vec<api::Version>>, modify: ReadSignal<bool>) -> Element {
    #[cfg(feature = "desktop")]
    const INSTALL_COL: &str = " auto";

    #[cfg(not(feature = "desktop"))]
    const INSTALL_COL: &str = "";

    let cols = if modify() { " auto" } else { "" };

    rsx! {
        div {
            style: "grid-template-columns: 1fr auto auto auto auto{cols}{INSTALL_COL};",
            class: "grid gap-x-4 border-b border-neutral-500 mb-2",
            div {
                class: "font-bold grid grid-cols-subgrid col-span-full px-4 py-2 border-b border-neutral-500",
                span { "Label" }
                span { class: "text-center", "Version" }
                span { class: "text-center", "Timestamp" }
                span { class: "text-center", "By" }
            }
            for version in versions.read().iter() {
                VersionRow { version: version.clone(), modify }
            }
        }
    }
}

#[component]
fn VersionRow(version: api::Version, modify: ReadSignal<bool>) -> Element {
    let time_string = version
        .timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let datetime = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .expect("Failed to convert date from unixepoch");
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| "Invalid Timestamp".to_string());

    #[cfg(feature = "desktop")]
    let install_btn = rsx! {
        button {
                title: "Deploy",
                class: "bg-yellow-300 hover:bg-yellow-200 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                img {
                    src: INSTALL_ICO
                }
            }
    };

    #[cfg(not(feature = "desktop"))]
    let install_btn = rsx! {};

    rsx! {
        div {
            class: "grid grid-cols-subgrid col-1 col-span-full py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700 items-center",

            span { "{version.label}" }
            span { class: "text-center", "{version.version}" }
            span { class: "text-center", {time_string} }
            span { class: "text-center", "{version.by.username}" }
            button {
                title: "Download",
                class: "bg-cyan-400 hover:bg-teal-300 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                img {
                    src: DOWNLOAD_ICO
                }
            }
            {install_btn}
            if modify() {
            button {
                title: "Delete",
                class: "bg-red-300 hover:bg-red-400 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                img {
                    src: TRASH_ICO
                }
            }
            }
        }
    }
}
