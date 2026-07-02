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
pub fn NewVersionFileSelection() -> (Element, impl Fn() -> Result<dioxus::html::FileData, String>) {
    let save = use_context::<Signal<api::Save>>();
    let game = use_memo(move || save.read().game);
    let can_fetch = use_memo(move || game().can_fetch());
    let mut auto = use_signal(|| can_fetch.cloned());

    let mut path = use_signal(std::path::PathBuf::new);
    let mut error = use_signal(|| None::<String>);

    let set_file_data = move || {
        let path = path.cloned();
        if !path.exists() {
            return Err("File does not exist".to_string());
        }
        Ok(dioxus::html::FileData::new(dx_ext::DesktopFileData(path)))
    };

    let ui = use_memo(move || {
        if auto() {
            use crate::desktop::SavePathFinder;
            rsx! {
                SavePathFinder {
                    save,
                    path,
                    set_path: move |p| path.set(p),
                    error: move |e| error.set(Some(e)),
                }
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

    (
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
        },
        set_file_data,
    )
}

#[cfg(not(feature = "desktop"))]
#[component]
pub fn NewVersionFileSelection() -> (Element, impl Fn() -> Result<dioxus::html::FileData, String>) {
    let set_file_data = || Err("File selection not available on this platform".to_string());

    (
        rsx! {
            Input {
                placeholder: "File",
                name: "file",
                multiple: false,
                r#type: "file",
                required: true,
            }
        },
        set_file_data,
    )
}
