use dioxus::prelude::*;

use crate::{Route, USER};

#[component]
pub fn Saves() -> Element {
    let mut saves_res = use_server_future(|| {
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
    })?;

    let saves = saves_res().unwrap_or_else(Vec::<api::Save>::new);

    let mut new_save_open = use_signal(|| false);

    rsx! {
        document::Title { "Save Manager" }

        SaveList { saves }

        button {
            class: "fixed bottom-4 right-4 w-12 h-12 rounded-full bg-emerald-400 hover:bg-green-300 flex items-center justify-center cursor-pointer",
            onclick: move |_| new_save_open.set(true),
            img { src: crate::icons::CIRCLE_PLUS }
        }

        crate::Dialog { open: new_save_open,
            h2 { class: "text-2xl font-bold", "New Save" }

            hr { class: "my-2" }
            form {
                class: "grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 p-4 items-center",
                onsubmit: move |e: FormEvent| async move {
                    e.prevent_default();

                    let data = e.data().values();

                    let get_text = |v: &FormValue| {
                        match v {
                            FormValue::Text(t) => t.clone(),
                            _ => unreachable!("Expected text field."),
                        }
                    };
                    let name = get_text(&data[0].1);
                    let game: api::Game = unsafe {
                        std::mem::transmute(get_text(&data[1].1).parse::<i8>().unwrap())
                    };

                    if name.contains('/') || name.contains('\\') {
                        crate::toast("Invalid Save Name".to_string(), rsx! {
                            p { "Save name cannot contain '/' or '\\' characters." }
                        });
                        return;
                    }

                    if let Err(e) = api::create_save(name, game).await {
                        debug!("Error creating save: {:?}", e);
                    } else {
                        new_save_open.set(false);
                    }
                    saves_res.restart();
                },
                label { r#for: "save_name", "Name" }
                input {
                    class: crate::INPUT_CLASS,
                    id: "save_name",
                    name: "save_name",
                    required: true,
                    placeholder: "Save Name",
                }
                label { r#for: "save_game", "Game" }
                select {
                    required: true,
                    name: "save_game",
                    id: "save_game",
                    class: crate::INPUT_CLASS,
                    for game in api::Game::iter() {
                        option { value: game as i32, "{game}" }
                    }
                }

                div { class: "flex flex-row justify-between col-span-full mt-4",

                    button {
                        class: "px-4 py-2 bg-gray-400 rounded cursor-pointer hover:bg-gray-500",
                        onclick: move |e| {
                            e.prevent_default();
                            new_save_open.set(false);
                        },
                        "Cancel"
                    }

                    button { class: "px-4 py-2 bg-emerald-400 rounded cursor-pointer hover:bg-green-300",
                        "Create"
                    }
                }
            }
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

    let mut filter = use_signal(String::new);
    let mut sorted_by = use_signal(|| SortBy::LastUpdatedDesc);

    let filtered_saves = use_memo(move || {
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
        SortBy::LastUpdatedAsc => crate::icons::CHEVRON_UP,
        SortBy::LastUpdatedDesc => crate::icons::CHEVRON_DOWN,
        _ => crate::icons::CHEVRON_UP_DOWN,
    };

    rsx! {
        div { class: "flex flex-col gap-y-1 mt-2",
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
            let datetime = chrono::DateTime::from_timestamp(t as i64, 0)
                .expect("Failed to convert date from unixepoch");
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
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

#[component]
fn SaveAccessList(save_id: i32) -> Element {
    let save_access_res =
        use_server_future(move || async move { api::get_save_access(save_id).await })?;

    let save_access = save_access_res().and_then(|res| res.ok());

    let save_list = save_access.map(|a| {
        rsx! {
            for access in a.access_list {
                SaveAccessRow { key: "{access.user.id}", access }
            }
        }
    });

    rsx! {
        {save_list}
    }
}

#[component]
fn SaveAccessRow(access: api::NamedUserAccess) -> Element {
    rsx! {
        div { class: "grid grid-cols-[1fr_auto] gap-x-4 border-b border-neutral-500 px-4 py-2 items-center",
            span { "{access.user.username}" }
            span { class: "text-center", "{access.access}" }
        }
    }
}
