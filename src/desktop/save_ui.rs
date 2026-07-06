use dioxus::core::SuperInto;

use crate::{
    desktop::{DeployOptions, DeployOptionsStoreExt},
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Props)]
struct GameSaveProps {
    deps: Store<DeployOptions>,
    allow_new: ReadSignal<bool>,
}

#[component]
pub fn SavePathFinder(
    save: ReadSignal<api::Save>,
    deps: Store<DeployOptions>,
    error: Callback<String>,
    allow_new: ReadSignal<bool>,
) -> Element {
    let game = use_memo(move || save.read().game);
    match game() {
        api::Game::IntoTheRadius2 => rsx! {
            IntoTheRadiusNewVersionOptions { deps, allow_new }
        },
        api::Game::Satisfactory => rsx! {
            SatisfactoryNewVersionOptions { deps, allow_new }
        },
    }
}

#[component]
pub fn IntoTheRadiusNewVersionOptions(
    GameSaveProps {
        mut deps,
        allow_new,
    }: GameSaveProps,
) -> Element {
    use crate::desktop::{self, into_the_radius_2::*};
    if !deps.is_into_the_radius_2() {
        deps.set(DeployOptions::IntoTheRadius2(Default::default()));
    }

    let itr = deps.into_the_radius_2().unwrap();

    let is_coop: ReadSignal<Option<bool>> = use_memo(move || Some(itr.coop()())).super_into();
    let slot: ReadSignal<Option<SaveSlots>> = use_memo(move || Some(itr.slot()())).super_into();

    let mut error_msg = use_signal(|| None::<String>);

    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    struct Saves {
        pub s1: bool,
        pub s2: bool,
        pub s3: bool,
        pub a1: bool,
        pub a2: bool,
        pub a3: bool,
    }

    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    struct Existing {
        pub single: Saves,
        pub coop: Saves,
    }

    let existing = use_memo(move || {
        let save_dir = desktop::dirs::get_game_save_dir(api::Game::IntoTheRadius2);

        let mut existing = Existing::default();

        {
            let single = save_dir.join("Single");
            existing.single.s1 = single.join("Save 1.dat.sav").exists();
            existing.single.s2 = single.join("Save 2.dat.sav").exists();
            existing.single.s3 = single.join("Save 3.dat.sav").exists();
            existing.single.a1 = single.join("Autosave 1.dat.sav").exists();
            existing.single.a2 = single.join("Autosave 2.dat.sav").exists();
            existing.single.a3 = single.join("Autosave 3.dat.sav").exists();
        }
        {
            let coop = save_dir.join("Coop");
            existing.coop.s1 = coop.join("Save 1.sav").exists();
            existing.coop.s2 = coop.join("Save 2.sav").exists();
            existing.coop.s3 = coop.join("Save 3.sav").exists();
            existing.coop.a1 = coop.join("Autosave 1.sav").exists();
            existing.coop.a2 = coop.join("Autosave 2.sav").exists();
            existing.coop.a3 = coop.join("Autosave 3.sav").exists();
        }
        existing
    });

    let curr_exists = use_memo(move || {
        if is_coop().unwrap_or(false) {
            existing.read().coop
        } else {
            existing.read().single
        }
    });

    rsx! {
        div { class: "flex flex-row justify-between gap-4 items-center",
            Select::<bool> {
                value: is_coop,
                width: "12rem",
                on_value_change: move |coop: Option<bool>| itr.coop().set(coop.unwrap_or(false)),
                SelectOption::<bool> { index: 0usize, value: false, text_value: "Singleplayer", "Singleplayer" }
                SelectOption::<bool> { index: 1usize, value: true, text_value: "Coop", "Coop" }
            }

            Select::<SaveSlots> {
                class: "flex flex-row justify-end",
                value: slot,
                width: "12rem",
                on_value_change: move |slot: Option<SaveSlots>| itr.slot().set(slot.unwrap_or_default()),
                if allow_new() || curr_exists().s1 {
                    SelectOption::<SaveSlots> {
                        index: 0usize,
                        value: SaveSlots::Slot1,
                        text_value: "Save 1",
                        "Save 1"
                    }
                }
                if allow_new() || curr_exists().s2 {
                    SelectOption::<SaveSlots> {
                        index: 1usize,
                        value: SaveSlots::Slot2,
                        text_value: "Save 2",
                        "Save 2"
                    }
                }
                if allow_new() || curr_exists().s3 {
                    SelectOption::<SaveSlots> {
                        index: 2usize,
                        value: SaveSlots::Slot3,
                        text_value: "Save 3",
                        "Save 3"
                    }
                }
                if allow_new() || curr_exists().a1 {
                    SelectOption::<SaveSlots> {
                        index: 3usize,
                        value: SaveSlots::AutoSave1,
                        text_value: "Autosave 1",
                        "Autosave 1"
                    }
                }
                if allow_new() || curr_exists().a2 {
                    SelectOption::<SaveSlots> {
                        index: 4usize,
                        value: SaveSlots::AutoSave2,
                        text_value: "Autosave 2",
                        "Autosave 2"
                    }
                }
                if allow_new() || curr_exists().a3 {
                    SelectOption::<SaveSlots> {
                        index: 5usize,
                        value: SaveSlots::AutoSave3,
                        text_value: "Autosave 3",
                        "Autosave 3"
                    }
                }
            }
        }

        if let Some(error) = error_msg() {
            p { class: "text-red-500", {error} }
        }
    }
}

#[component]
pub fn SatisfactoryNewVersionOptions(
    GameSaveProps {
        mut deps,
        allow_new,
    }: GameSaveProps,
) -> Element {
    rsx! {}
}
