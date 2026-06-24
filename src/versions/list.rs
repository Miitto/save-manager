use dioxus::prelude::*;

use crate::ConfirmDialog;

use super::{SaveName, VersionProvider};

#[component]
pub fn VersionList(versions: Store<Vec<api::Version>>, modify: ReadSignal<bool>) -> Element {
    #[cfg(feature = "desktop")]
    const INSTALL_COL: &str = " auto";

    #[cfg(not(feature = "desktop"))]
    const INSTALL_COL: &str = "";

    let cols = if modify() { " auto" } else { "" };

    rsx! {
        div {
            style: "grid-template-columns: 1fr auto auto auto auto{cols}{INSTALL_COL};",
            class: "grid gap-x-4 border-b border-neutral-500 mb-2",
            div { class: "font-bold grid grid-cols-subgrid col-span-full px-4 py-2 border-b border-neutral-500",
                span { "Label" }
                span { class: "text-center", "Version" }
                span { class: "text-center", "Timestamp" }
                span { class: "text-center", "By" }
            }
            for version in versions() {
                VersionRow { key: "{version.id}", version, modify }
            }
        }
    }
}

#[component]
pub fn VersionRow(version: ReadSignal<api::Version>, modify: ReadSignal<bool>) -> Element {
    let time_string = chrono::DateTime::from_timestamp(version().timestamp as i64, 0)
        .expect("Failed to convert date from unixepoch")
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let mut delete_open = use_signal(|| false);

    let mut version_list = use_context::<VersionProvider>();

    let mut delete_version = use_action(move || async move {
        api::delete_version(version().save_id, version().id).await?;
        version_list.restart();
        Ok(()) as Result<(), ServerFnError>
    });

    rsx! {
        div { class: "grid grid-cols-subgrid col-span-full py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700 items-center",

            span { "{version().label}" }
            span { class: "text-center", "{version().version}" }
            span { class: "text-center", {time_string} }
            span { class: "text-center", "{version().by.username}" }
            DownloadButton { version }
            InstallButton { version }
            if modify() {
                button {
                    title: "Delete",
                    class: "bg-red-300 hover:bg-red-400 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                    onclick: move |_| {
                        delete_open.set(true);
                    },
                    img { src: crate::icons::TRASH }
                }
            }
        }

        if delete_open() {
            ConfirmDialog {
                title: "Delete Version".to_string(),
                message: format!(
                    "Are you sure you want to delete version {} (\"{}\")?",
                    version().version,
                    version().label,
                ),
                on_confirm: move |_| {
                    delete_version.call();
                },
                open: delete_open,
            }
        }
    }
}

const DOWNLOAD_CLASS: &str = "bg-cyan-400 hover:bg-teal-300 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center";

#[cfg(not(feature = "desktop"))]
#[component]
fn DownloadButton(version: ReadSignal<api::Version>) -> Element {
    rsx! {
        Link {
            class: DOWNLOAD_CLASS,
            to: format!("/api/save/{}/{}/download", version().save_id, version().id),
            img { src: crate::icons::DOWNLOAD }
        }
    }
}

#[cfg(feature = "desktop")]
#[component]
fn DownloadButton(version: ReadSignal<api::Version>) -> Element {
    let save_name = use_context::<Signal<SaveName>>();

    rsx! {
        button {
            title: "Download",
            class: DOWNLOAD_CLASS,
            onclick: move |_| {
                let version = version.peek().clone();
                async move {
                    #[cfg(feature = "desktop")]
                    {
                        let name = save_name.peek().name.clone();
                        match crate::file_management::download_version(&name, &version)
                            .await
                        {
                            Ok(path) => {
                                crate::toast_success(
                                    "Download Complete".to_string(),
                                    rsx! {
                                        p { "Version {version.version} downloaded successfully." }
                                        button {
                                            class: "underline cursor-pointer",
                                            onclick: move |_| {
                                                use crate::file_management::ExplorerView;
                                                path.select_file();
                                            },
                                            "View in Explorer"
                                        }
                                    },
                                );
                            }
                            Err(_) => {
                                crate::toast_error(
                                    "Download Failed".to_string(),
                                    format!("Failed to download version {}.", version.version),
                                );
                            }
                        }
                    }
                }
            },
            img { src: crate::icons::DOWNLOAD }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
fn InstallButton(version: ReadSignal<api::Version>) -> Element {
    rsx! {}
}

#[cfg(feature = "desktop")]
#[component]
fn InstallButton(version: ReadSignal<api::Version>) -> Element {
    rsx! {
        button {
            title: "Deploy",
            class: "bg-yellow-300 hover:bg-yellow-200 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
            onclick: move |_| {
                crate::toast_error("WIP", rsx! {
                    p { "Deploying versions is not yet implemented." }
                });
            },
            img { src: crate::icons::INSTALL }
        }
    }
}
