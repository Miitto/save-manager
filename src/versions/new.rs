use crate::prelude::*;

use crate::versions::SaveProvider;

pub trait CanAuto {
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
    let save = use_context::<SaveProvider>();
    let game = use_memo(move || save.read().game);
    let can_fetch = use_memo(move || game().can_fetch());
    let mut auto = use_signal(|| can_fetch.cloned());

    let deps = use_store(|| crate::desktop::DeployOptions::from(game()));

    let path = use_memo(move || {
        let read = deps.read();
        let deps = std::ops::Deref::deref(&read);
        let path: std::path::PathBuf = deps.into();
        path
    });

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
                    deps,
                    error: move |e| error.set(Some(e)),
                    allow_new: false,
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

#[cfg(feature = "desktop")]
#[component]
pub fn DeployVersionFileSelection(deps: Store<crate::desktop::DeployOptions>) -> Element {
    let save = use_context::<SaveProvider>();

    let mut error = use_signal(|| None::<String>);

    rsx! {
        h3 { class: "text-xl", "File Selection" }

        Separator {}

        crate::desktop::SavePathFinder {
            save,
            deps,
            error: move |e| error.set(Some(e)),
            allow_new: true,
        }
    }
}
