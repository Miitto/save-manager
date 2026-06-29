use crate::prelude::*;

trait CanAuto {
    fn can_fetch(&self) -> bool;
    fn can_deploy(&self) -> bool;
}

impl CanAuto for api::Game {
    fn can_fetch(&self) -> bool {
        matches!(self, api::Game::IntoTheRadius2)
    }

    fn can_deploy(&self) -> bool {
        matches!(self, api::Game::IntoTheRadius2)
    }
}

#[cfg(feature = "desktop")]
#[component]
pub fn NewVersionFileSelection() -> Element {
    let save = use_context::<Signal<api::Save>>();
    let game = use_memo(move || save.read().game);
    let can_fetch = use_memo(move || game().can_fetch());
    let mut auto = use_signal(|| can_fetch.cloned());

    let ui = use_memo(move || {
        if auto() {
            use desktop_ui::NewVersionGameOptions;
            rsx! {
                NewVersionGameOptions {}
            }
        } else {
            rsx! {
                Input {
                    placeholder: "File",
                    name: "file",
                    multiple: false,
                    r#type: "file",
                    required: true,
                }
            }
        }
    });

    rsx! {
        div { class: "flex flex-row justify-between items-center gap-4",
            h3 { class: "text-xl", "File Selection" }

            div { class: "flex flex-row rounded",
                Button {
                    class: "rounded-r-none",
                    disabled: !can_fetch(),
                    variant: if auto() { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                    onclick: move |e: MouseEvent| {
                        e.prevent_default();
                        auto.set(true);
                    },
                    "Auto"
                }
                Button {
                    class: "rounded-l-none",
                    variant: if !auto() { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                    onclick: move |e: MouseEvent| {
                        e.prevent_default();
                        auto.set(false);
                    },
                    "Manual"
                }
            }
        }

        Separator {}

        {ui}
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
pub fn NewVersionFileSelection() -> Element {
    rsx! {
        Input {
            placeholder: "File",
            name: "file",
            multiple: false,
            r#type: "file",
            required: true,
        }
    }
}

#[cfg(feature = "desktop")]
mod desktop_ui {
    use dioxus::core::SuperInto;

    use crate::{
        desktop::{DeployOptions, DeployOptionsStoreExt},
        prelude::*,
        versions::NewVersionContext,
    };

    #[component]
    pub fn NewVersionGameOptions() -> Element {
        let save = use_context::<Signal<api::Save>>();

        let NewVersionContext {
            mut file,
            mut error,
        } = use_context::<NewVersionContext>();

        let game = use_memo(move || save.read().game);

        let deps = use_store(|| crate::desktop::DeployOptions::from(game()));

        let mut path = use_signal(std::path::PathBuf::new);

        let set_path = move |p: std::path::PathBuf| {
            path.set(p);
        };

        let go = use_memo(move || match game() {
            api::Game::IntoTheRadius2 => rsx! {
                IntoTheRadiusNewVersionOptions { deps, set_path }
            },
            api::Game::Satisfactory => rsx! {
                SatisfactoryNewVersionOptions { deps }
            },
        });

        let mut locked = use_signal(|| false);

        let set_file_data = move || {
            let path = path.cloned();
            async move {
                if !path.exists() {
                    error.set(Some("File does not exist".to_string()));
                    return;
                }
                locked.set(true);
                let file_data =
                    dioxus::html::FileData::new(crate::versions::custom_types::FileData(path));

                file.set(Some(file_data));
            }
        };

        rsx! {
            if locked() {
                span { class: "text-white/50", "{path().display()}" }
                Button {
                    size: ButtonSize::Lg,
                    onclick: move |e: MouseEvent| {
                        e.prevent_default();
                        file.set(None);
                        path.set(std::path::PathBuf::new());
                        locked.set(false);
                    },
                    "Clear"
                }
            } else {
                {go}

                Button {
                    size: ButtonSize::Lg,
                    onclick: move |e: MouseEvent| async move {
                        e.prevent_default();

                        set_file_data().await;
                    },
                    "Select"
                }
            }
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
                    SelectOption::<bool> {
                        index: 1usize,
                        value: false,
                        text_value: "Singleplayer",
                        "Singleplayer"
                    }
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
}
