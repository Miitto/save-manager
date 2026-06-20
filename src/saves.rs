use dioxus::{html::filter, prelude::*};

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
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum SortBy {
        NameAsc,
        NameDesc,
        GameAsc,
        GameDesc,
        LastUpdatedAsc,
        LastUpdatedDesc,
    }

    let mut filter = use_signal(|| String::new());
    let mut sorted_by = use_signal(|| SortBy::LastUpdatedAsc);

    let mut filtered_saves = use_memo(move || {
        let filter_str = filter().to_lowercase();
        saves()
            .clone()
            .into_iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&filter_str)
                    || s.game.to_string().to_lowercase().contains(&filter_str)
            })
            .collect::<Vec<api::Save>>()
    });

    let sorted_saves = use_memo(move || {
        let mut saves = filtered_saves();
        match sorted_by() {
            SortBy::NameAsc => saves.sort_by_key(|a| a.name.to_lowercase()),
            SortBy::NameDesc => saves.sort_by_key(|b| std::cmp::Reverse(b.name.to_lowercase())),
            SortBy::GameAsc => saves.sort_by_key(|a| a.game.to_string()),
            SortBy::GameDesc => saves.sort_by_key(|b| std::cmp::Reverse(b.game.to_string())),
            SortBy::LastUpdatedAsc => saves.sort_by_key(|a| a.most_recent_version),
            SortBy::LastUpdatedDesc => {
                saves.sort_by_key(|b| std::cmp::Reverse(b.most_recent_version))
            }
        }
        saves
    });

    let name_sort_icon = match sorted_by() {
        SortBy::NameAsc => crate::icons::CHEVRON_DOWN,
        SortBy::NameDesc => crate::icons::CHEVRON_UP,
        _ => crate::icons::CHEVRON_UP_DOWN,
    };

    let button_sort_icon = match sorted_by() {
        SortBy::GameAsc => crate::icons::CHEVRON_DOWN,
        SortBy::GameDesc => crate::icons::CHEVRON_UP,
        _ => crate::icons::CHEVRON_UP_DOWN,
    };

    let last_updated_sort_icon = match sorted_by() {
        SortBy::LastUpdatedAsc => crate::icons::CHEVRON_DOWN,
        SortBy::LastUpdatedDesc => crate::icons::CHEVRON_UP,
        _ => crate::icons::CHEVRON_UP_DOWN,
    };

    rsx! {
        div { class: "flex flex-col gap-y-1",
            div { class: "flex flex-row items-center justify-end px-2",
                input {
                    class: "px-2 py-1 rounded grow max-w-100 bg-neutral-800 text-white border border-neutral-600 focus:outline-none focus:ring-2 focus:ring-blue-500",
                    placeholder: "Filter saves...",
                    value: "{filter()}",
                    oninput: move |e| filter.set(e.value()),
                }
            }
            div { class: "grid grid-cols-[1fr_auto_auto_auto] gap-x-4 border-b border-neutral-500 mb-2 items-center",
                div { class: "font-bold grid grid-cols-subgrid col-span-full px-4 py-2 border-b border-neutral-500",
                    div { class: "flex flex-row items-center gap-2",
                        span { "Name" }
                        button {
                            class: "text-white w-8 h-8 flex justify-center items-center cursor-pointer hover:bg-neutral-600 rounded",
                            onclick: move |_| {
                                match sorted_by() {
                                    SortBy::NameAsc => sorted_by.set(SortBy::NameDesc),
                                    _ => sorted_by.set(SortBy::NameAsc),
                                }
                            },
                            img { class: "invert", src: name_sort_icon }
                        }
                    }
                    div { class: "flex flex-row items-center gap-2",
                        span { "Game" }

                        button {
                            class: "text-white w-8 h-8 flex justify-center items-center cursor-pointer hover:bg-neutral-600 rounded",
                            onclick: move |_| {
                                match sorted_by() {
                                    SortBy::GameAsc => sorted_by.set(SortBy::GameDesc),
                                    _ => sorted_by.set(SortBy::GameAsc),
                                }
                            },
                            img { class: "invert", src: button_sort_icon }
                        }
                    }
                    div { class: "flex flex-row items-center gap-2",
                        span { "Last Updated" }
                        button {
                            class: "text-white w-8 h-8 flex justify-center items-center cursor-pointer hover:bg-neutral-600 rounded",
                            onclick: move |_| {
                                match sorted_by() {
                                    SortBy::LastUpdatedAsc => sorted_by.set(SortBy::LastUpdatedDesc),
                                    _ => sorted_by.set(SortBy::LastUpdatedAsc),
                                }
                            },
                            img { class: "invert", src: last_updated_sort_icon }
                        }

                    }
                    span { "Versions" }
                }
                for save in sorted_saves() {
                    SaveRow { key: "{save.id}", save }
                }
            }
        }
    }
}

#[component]
fn SaveRow(save: api::Save) -> Element {
    let time = save
        .most_recent_version
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| {
                    let datetime = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                        .expect("Failed to convert date from unixepoch");
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|_| "Invalid Timestamp".to_string())
        })
        .unwrap_or_else(|| "Never".to_string());

    rsx! {
        Link {
            to: Route::SaveDetails { id: save.id },
            class: "grid grid-cols-subgrid col-span-full py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700",

            span { "{save.name}" }
            span { "{save.game}" }
            span { {time} }
            span { class: "text-center", "{save.version_count}" }
        }
    }
}
