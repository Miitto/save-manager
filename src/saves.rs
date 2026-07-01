use crate::prelude::*;

use crate::{Route, USER};

#[component]
pub fn Saves() -> Element {
    let toast_api = use_toast();
    let mut saves = use_loader(move || {
        let id = {
            let u = USER.read();
            u.as_ref().unwrap().id
        };
        async move { api::get_user_saves(id).await }
    })?;

    let mut new_save_open = use_signal(|| false);

    let mut selected_game = use_signal::<api::Game>(api::Game::default);

    rsx! {
        document::Title { "Save Manager" }

        SaveList { saves }

        Button {
            size: ButtonSize::IconLg,
            class: "fixed bottom-4 right-4",
            onclick: move |_| new_save_open.set(true),
            icons::CirclePlus {}
        }

        Dialog {
            open: new_save_open(),
            on_open_change: move |open| new_save_open.set(open),
            DialogTitle { "New Save" }

            Separator {}

            form {
                class: "grid grid-cols-1 gap-y-2 p-4 items-center",
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

                    if name.contains('/') || name.contains('\\') {
                        toast_api
                            .error(
                                "Invalid Save Name".to_string(),
                                ToastOptions::new()
                                    .description("Save name cannot contain '/' or '\\' characters."),
                            );
                        return;
                    }
                    if let Err(e) = api::create_save(name, selected_game()).await {
                        debug!("Error creating save: {:?}", e);
                    } else {
                        new_save_open.set(false);
                    }
                    saves.restart();
                },
                LabeledInput {
                    id: "save_name",
                    name: "save_name",
                    required: true,
                    placeholder: "Save Name",
                    "Name"
                }
                div { class: "flex flex-col gap-y-1",
                    Label { html_for: "save_game", "Game" }
                    Select::<api::Game> {
                        name: "save_game",
                        id: "save_game",
                        width: "12rem",
                        on_value_change: move |game: Option<api::Game>| selected_game.set(game.unwrap_or_default()),
                        SelectGroup {
                            SelectGroupLabel { "Games" }
                            for game in api::Game::iter() {
                                SelectOption::<api::Game> {
                                    index: game as usize,
                                    value: game,
                                    text_value: "{game}",
                                    "{game}"
                                }
                            }
                        }
                    }
                }

                div { class: "flex flex-row justify-between col-span-full mt-4 w-full",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Lg,
                        onclick: move |e: MouseEvent| {
                            e.prevent_default();
                            new_save_open.set(false);
                        },
                        "Cancel"
                    }

                    Button { size: ButtonSize::Lg, "Create" }
                }
            }
        }
    }
}

#[component]
fn SaveList(saves: Loader<Vec<api::Save>>) -> Element {
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
        SortBy::NameAsc => dioxus_icons::lucide::ChevronDown,
        SortBy::NameDesc => dioxus_icons::lucide::ChevronUp,
        _ => dioxus_icons::lucide::ChevronsUpDown,
    };

    let game_sort_icon = match sorted_by() {
        SortBy::GameAsc => dioxus_icons::lucide::ChevronDown,
        SortBy::GameDesc => dioxus_icons::lucide::ChevronUp,
        _ => dioxus_icons::lucide::ChevronsUpDown,
    };

    let last_updated_sort_icon = match sorted_by() {
        SortBy::LastUpdatedAsc => dioxus_icons::lucide::ChevronUp,
        SortBy::LastUpdatedDesc => dioxus_icons::lucide::ChevronDown,
        _ => dioxus_icons::lucide::ChevronsUpDown,
    };

    rsx! {
        div { class: "flex flex-col gap-y-1 mt-2",
            div { class: "flex flex-row items-center justify-end px-2",
                Input {
                    class: "grow max-w-100",
                    placeholder: "Filter saves...",
                    value: "{filter()}",
                    oninput: move |e: FormEvent| filter.set(e.value()),
                }
            }
            div { class: "grid grid-cols-[1fr_auto_auto_auto] gap-x-4 border-b border-border mb-2 items-center",
                div { class: "font-bold grid grid-cols-subgrid col-span-full px-4 py-2 border-b border-border",
                    div { class: "flex flex-row items-center gap-2",
                        span { "Name" }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            onclick: move |_| {
                                match sorted_by() {
                                    SortBy::NameAsc => sorted_by.set(SortBy::NameDesc),
                                    _ => sorted_by.set(SortBy::NameAsc),
                                }
                            },
                            name_sort_icon { size: 32 }
                        }
                    }
                    div { class: "flex flex-row items-center gap-2",
                        span { "Game" }

                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            onclick: move |_| {
                                match sorted_by() {
                                    SortBy::GameAsc => sorted_by.set(SortBy::GameDesc),
                                    _ => sorted_by.set(SortBy::GameAsc),
                                }
                            },
                            game_sort_icon {}
                        }
                    }
                    div { class: "flex flex-row items-center gap-2",
                        span { "Last Updated" }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            onclick: move |_| {
                                match sorted_by() {
                                    SortBy::LastUpdatedAsc => sorted_by.set(SortBy::LastUpdatedDesc),
                                    _ => sorted_by.set(SortBy::LastUpdatedAsc),
                                }
                            },
                            last_updated_sort_icon {}
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
            class: "grid grid-cols-subgrid col-span-full py-2 px-4 hover:bg-white/15 odd:bg-white/10",

            span { "{save.name}" }
            span { "{save.game}" }
            span { {time} }
            span { class: "text-center", "{save.version_count}" }
        }
    }
}
