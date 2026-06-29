use crate::prelude::*;

use super::VersionProvider;

#[component]
pub fn VersionList(versions: Loader<Vec<api::Version>>, modify: ReadSignal<bool>) -> Element {
    #[cfg(feature = "desktop")]
    const INSTALL_COL: &str = " auto";

    #[cfg(not(feature = "desktop"))]
    const INSTALL_COL: &str = "";

    let cols = if modify() { " auto" } else { "" };

    rsx! {
        div {
            style: "grid-template-columns: 1fr auto auto auto auto{cols}{INSTALL_COL};",
            class: "grid gap-x-4 border-b border-border mb-2",
            div { class: "font-bold grid grid-cols-subgrid col-span-full px-4 py-2 border-b border-border",
                span { "Label" }
                span { class: "text-center", "Version" }
                span { class: "text-center", "Timestamp" }
                span { class: "text-center", "By" }
            }
            for version in versions.read().iter() {
                VersionRow {
                    key: "{version.id}",
                    version: version.clone(),
                    modify,
                }
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
        div { class: "grid grid-cols-subgrid col-span-full py-2 px-4 hover:bg-white/15 odd:bg-white/10 items-center",

            span { "{version().label}" }
            span { class: "text-center", "{version().version}" }
            span { class: "text-center", {time_string} }
            span { class: "text-center", "{version().by.username}" }
            DownloadButton { version }
            InstallButton { version }
            if modify() {
                Button {
                    title: "Delete",
                    variant: ButtonVariant::Destructive,
                    size: ButtonSize::Icon,
                    onclick: move |_| {
                        delete_open.set(true);
                    },

                    icons::Trash2 {}
                }
            }
        }

        if delete_open() {
            AlertDialog {
                open: delete_open(),
                on_open_change: move |open| {
                    delete_open.set(open);
                },
                AlertDialogTitle { "Delete Version" }
                AlertDialogDescription {
                    "Are you sure you want to delete version {version().version} (\"{version().label}\")?"
                }
                AlertDialogActions {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: move |_| {
                            delete_version.call();
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
fn DownloadButton(version: ReadSignal<api::Version>) -> Element {
    rsx! {
        Link {
            class: "dx-button",
            to: format!("/api/save/{}/{}/download", version().save_id, version().id),
            icons::Download {}
        }
    }
}

#[cfg(feature = "desktop")]
#[component]
fn DownloadButton(version: ReadSignal<api::Version>) -> Element {
    use dioxus_primitives::toast::ToastOptions;

    let toast_api = use_toast();
    let save = use_context::<Signal<api::Save>>();

    rsx! {
        Button {
            title: "Download",
            size: ButtonSize::Icon,
            onclick: move |_| {
                let version = version.peek().clone();
                async move {
                    #[cfg(feature = "desktop")]
                    {
                        let name = save.peek().name.clone();
                        match crate::desktop::download_version(&name, &version)
                            .await
                        {
                            Ok(path) => {
                                toast_api
                                    .success(
                                        "Download Complete".to_string(),
                                        ToastOptions::new()
                                            .description(
                                                format!("Version downloaded to {}", path.display()),
                                            ),
                                    );
                            }
                            Err(_) => {
                                toast_api
                                    .error(
                                        "Download Failed".to_string(),
                                        ToastOptions::new()
                                            .description(
                                                "Failed to download version. Please try again.".to_string(),
                                            ),
                                    );
                            }
                        }
                    }
                }
            },
            icons::Download {}
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
    let toast_api = use_toast();

    rsx! {
        Button {
            title: "Deploy",
            size: ButtonSize::Icon,
            onclick: move |_| {
                toast_api
                    .error(
                        "WIP".to_string(),
                        ToastOptions::new()
                            .description("Deploying is not yet implemented.".to_string()),
                    );
            },
            icons::HardDriveDownload {}
        }
    }
}
