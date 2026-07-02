use dioxus::core::SuperInto;

use crate::{
    desktop::{DeployOptions, DeployOptionsStoreExt},
    prelude::*,
};

#[component]
pub fn SavePathFinder(
    save: ReadSignal<api::Save>,
    path: ReadSignal<std::path::PathBuf>,
    set_path: Callback<std::path::PathBuf>,
    error: Callback<String>,
) -> Element {
    let game = use_memo(move || save.read().game);

    let deps = use_store(|| crate::desktop::DeployOptions::from(game()));

    match game() {
        api::Game::IntoTheRadius2 => rsx! {
            IntoTheRadiusNewVersionOptions { deps, set_path }
        },
        api::Game::Satisfactory => rsx! {
            SatisfactoryNewVersionOptions { deps }
        },
    }
}

#[component]
pub fn IntoTheRadiusNewVersionOptions(
    deps: Store<DeployOptions>,
    set_path: Callback<std::path::PathBuf>,
) -> Element {
    use crate::desktop::{self, into_the_radius_2::*};
    if !deps.is_into_the_radius_2() {
        deps.set(DeployOptions::IntoTheRadius2(Default::default()));
    }

    let itr = deps.into_the_radius_2().unwrap();

    let is_coop: ReadSignal<Option<bool>> = use_memo(move || Some(itr.coop()())).super_into();
    let slot: ReadSignal<Option<SaveSlots>> = use_memo(move || Some(itr.slot()())).super_into();

    let mut error_msg = use_signal(|| None::<String>);

    use_effect(move || {
        let coop = itr.coop()();
        let slot = itr.slot()();

        let save_dir = desktop::dirs::get_game_save_dir(api::Game::IntoTheRadius2);
        if !save_dir.exists() {
            error_msg.set(Some(format!(
                "Save directory does not exist: {}",
                save_dir.display()
            )));
        }

        let subfolder = if coop { "Coop" } else { "Single" };
        let slot_path = save_dir.join(subfolder).join(slot.name(coop));

        set_path(slot_path);
    });

    rsx! {
        div { class: "flex flex-row justify-between gap-4 items-center",
            Select::<bool> {

                value: is_coop,
                width: "12rem",
                on_value_change: move |coop: Option<bool>| itr.coop().set(coop.unwrap_or(false)),
                SelectOption::<bool> { index: 1usize, value: false, text_value: "Singleplayer", "Singleplayer" }
                SelectOption::<bool> { index: 1usize, value: true, text_value: "Coop", "Coop" }
            }

            Select::<SaveSlots> {
                class: "flex flex-row justify-end",
                value: slot,
                width: "12rem",
                on_value_change: move |slot: Option<SaveSlots>| itr.slot().set(slot.unwrap_or_default()),
                SelectOption::<SaveSlots> {
                    index: 1usize,
                    value: SaveSlots::Slot1,
                    text_value: "Save 1",
                    "Save 1"
                }
                SelectOption::<SaveSlots> {
                    index: 1usize,
                    value: SaveSlots::Slot2,
                    text_value: "Save 2",
                    "Save 2"
                }
                SelectOption::<SaveSlots> {
                    index: 2usize,
                    value: SaveSlots::Slot3,
                    text_value: "Save 3",
                    "Save 3"
                }
                SelectOption::<SaveSlots> {
                    index: 3usize,
                    value: SaveSlots::AutoSave1,
                    text_value: "Autosave 1",
                    "Autosave 1"
                }
                SelectOption::<SaveSlots> {
                    index: 4usize,
                    value: SaveSlots::AutoSave2,
                    text_value: "Autosave 2",
                    "Autosave 2"
                }
                SelectOption::<SaveSlots> {
                    index: 5usize,
                    value: SaveSlots::AutoSave3,
                    text_value: "Autosave 3",
                    "Autosave 3"
                }
            }
        }

        if let Some(error) = error_msg() {
            p { class: "text-red-500", {error} }
        }
    }
}

#[component]
pub fn SatisfactoryNewVersionOptions(deps: Store<DeployOptions>) -> Element {
    rsx! {}
}
