use crate::prelude::*;

use super::VersionProvider;

#[component]
pub fn VersionList(
    versions: WriteStore<Vec<api::Version>>,
    modify: ReadSignal<bool>,
    deploy_version: Callback<api::Version>,
) -> Element {
    use super::CanAuto;
    let mut cols = 4;

    let game = use_context::<Signal<api::Save>>().read().game;

    #[cfg(feature = "desktop")]
    {
        cols += 1;
        if game.can_deploy() {
            cols += 1;
        }
    }

    if modify() {
        cols += 1;
    }

    let mut remove_version = move |id: api::VersionId| {
        versions.write().retain(|v| v.id != id);
    };

    rsx! {
        div {
            style: "grid-template-columns: 1fr repeat({cols}, auto);",
            class: "grid gap-x-4 border-b border-border mb-2 max-w-full overflow-hidden",
            div { class: "font-bold grid grid-cols-subgrid col-span-full px-4 py-2 border-b border-border max-w-full",
                span { "Label" }
                span { class: "text-center", "Version" }
                span { class: "text-center", "Timestamp" }
                span { class: "text-center", "By" }
            }
            for version in versions.iter() {
                VersionRow {
                    key: "{version.read().id}",
                    version,
                    modify,
                    remove_version: move |_| {
                        remove_version(version.peek().id);
                    },
                    can_deploy: cfg!(feature = "desktop") && game.can_deploy(),
                    deploy_version,
                }
            }
        }
    }
}

#[component]
pub fn VersionRow(
    version: ReadStore<api::Version>,
    modify: ReadSignal<bool>,
    remove_version: Callback<()>,
    #[props(default)] can_deploy: bool,
    deploy_version: Callback<api::Version>,
) -> Element {
    let toast_api = use_toast();

    let save = use_context::<Signal<api::Save>>();

    let time_string = chrono::DateTime::from_timestamp(version().timestamp as i64, 0)
        .expect("Failed to convert date from unixepoch")
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let mut delete_open = use_signal(|| false);

    let mut version_list = use_context::<VersionProvider>();

    let delete_version = move || {
        remove_version.call(());
        async move {
            if let Err(e) = api::delete_version(version().save_id, version().id).await {
                error!("Failed to delete version: {e}");
                toast_api.error(
                    "Failed to delete version".to_string(),
                    ToastOptions::new().description(e.to_string()),
                );
                version_list.restart();
            }
        }
    };

    #[cfg(feature = "desktop")]
    let reveal_button = {
        use crate::desktop::ExplorerView;
        use std::ops::Deref;

        let path = use_memo(move || {
            let version = version.read();
            crate::desktop::get_version_path(&save.read().name, version.deref())
        });
        debug!("Version path: {:?}", path().display());
        rsx! {
            if path.read().exists() {
                Button {
                    size: ButtonSize::Icon,
                    title: "Reveal in File Explorer",
                    onclick: move |_| { path().select_file() },
                    icons::FolderOpen {}
                }
            } else {
                br {}
            }
        }
    };

    #[cfg(not(feature = "desktop"))]
    let reveal_button = rsx! {};

    rsx! {
        div { class: "grid grid-cols-subgrid col-span-full max-w-full py-2 px-4 hover:bg-white/15 odd:bg-white/10 items-center",
            span { class: "overflow-ellipsis overflow-hidden", "{version().label}" }
            span { class: "text-center", "{version().version}" }
            span { class: "text-center", {time_string} }
            span { class: "overflow-ellipsis overflow-hidden text-center", "{version().by.username}" }
            {reveal_button}
            DownloadButton { version }
            if can_deploy {
                InstallButton { version, deploy_version }
            }
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

        AlertDialog {
            open: delete_open(),
            on_open_change: move |open| {
                delete_open.set(open);
            },
            AlertDialogTitle { "Delete Version" }
            AlertDialogDescription { "Are you sure you want to delete version {version().version} (\"{version().label}\")?" }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                AlertDialogAction {
                    on_click: move |_| async move {
                        delete_version().await;
                    },
                    "Delete"
                }
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
fn DownloadButton(version: ReadSignal<api::Version>) -> Element {
    rsx! {
        ButtonLink {
            size: ButtonSize::Icon,
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
                                            )
                                            .duration(std::time::Duration::from_secs(10)),
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
fn InstallButton(
    version: ReadSignal<api::Version>,
    deploy_version: Callback<api::Version>,
) -> Element {
    rsx! {}
}

#[cfg(feature = "desktop")]
#[component]
fn InstallButton(
    version: ReadSignal<api::Version>,
    deploy_version: Callback<api::Version>,
) -> Element {
    rsx! {
        Button {
            title: "Deploy",
            size: ButtonSize::Icon,
            onclick: move |_| {
                deploy_version(version.peek().clone());
            },
            icons::HardDriveDownload {}
        }
    }
}
