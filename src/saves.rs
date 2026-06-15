use dioxus::prelude::*;

use crate::{Dialog, Route, USER};

#[component]
pub fn Saves() -> Element {
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
        document::Title { "Save Manager" }

        div { class: "flex flex-col",
            button {
                class: "cursor-pointer active:underline",
                onclick: move |_| async move {
                    login_user.call().await;
                },
                "Login Test User"
            }

            button {
                onclick: move |_| async move {
                    logout_user.call().await;
                },
                "Logout"
            }

            pre { "Logged in: {login_user.value():?}" }
            pre { "Username: {username}" }

            SaveList { saves }

        }

    }
}

#[component]
fn SaveList(saves: ReadSignal<Vec<api::Save>>) -> Element {
    rsx! {
        div { class: "grid grid-cols-[1fr_auto] gap-x-4 border-b border-neutral-500 mb-2",
            div { class: "font-bold grid grid-cols-subgrid col-span-2 px-4 py-2 border-b border-neutral-500",
                span { "Name" }
                span { "Versions" }
            }
            for save in saves() {
                SaveRow { key: "{save.id}", save }
            }
        }
    }
}

#[component]
fn SaveRow(save: api::Save) -> Element {
    rsx! {
        Link {
            to: Route::SaveDetails { id: save.id },
            class: "grid grid-cols-subgrid col-span-2 py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700",

            span { "{save.name}" }
            span { class: "text-center", "{save.version_count}" }
        }
    }
}
