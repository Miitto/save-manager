use std::process::Command;

use dioxus::prelude::*;

use api::UserAccessExt;

use futures::StreamExt;

use crate::{ConfirmDialog, Dialog, USER};

type VersionProvider = Resource<Result<Vec<api::Version>, ServerFnError>>;

struct SaveName {
    name: String,
}

#[component]
pub fn SaveDetails(id: i32) -> Element {
    let save = use_server_future(move || api::get_save_details(id))?().unwrap();
    let mut save_versions_res = use_server_future(move || api::get_save_versions(id))?;

    use_context_provider::<VersionProvider>(|| save_versions_res);

    let save_version_list = save_versions_res().unwrap();

    let mut save_name = use_signal(|| SaveName {
        name: String::default(),
    });

    use_context_provider(|| save_name);

    let modify = use_server_future(move || {
        _ = USER();
        async move { api::get_user_save_access(id).await.map(|a| a.can_edit()) }
    })?()
    .map(|r| r.unwrap_or(false))
    .unwrap_or(false);

    let count = match save_version_list.as_ref() {
        Ok(versions) => versions.len(),
        Err(_) => 0,
    };

    let save_versions = match save_version_list {
        Ok(versions) => {
            rsx! {
                VersionList { versions, modify }
            }
        }
        Err(_) => rsx! {
            p { "Failed to load versions" }
        },
    };
    let mut new_version_open = use_signal(|| false);
    let mut delete_save_open = use_signal(|| false);
    let mut save_access_open = use_signal(|| false);

    let mut save_access_res =
        use_server_future(move || async move { api::get_save_access(id).await })?;

    let mut add_new_access = use_action(move |username: String| async move {
        if let Err(e) = api::add_user_save_access(id, username).await {
            error!("Failed to add access: {e}");
            return match e {
                ServerFnError::ServerError { message, .. } => Ok(Some(message)),
                _ => Ok(Some("Failed to add access".to_string())),
            };
        }
        save_access_res.restart();
        Ok(None) as Result<Option<String>, ServerFnError>
    });

    let add_new_access_error = add_new_access
        .value()
        .and_then(|e| e.ok().map(|e| e()))
        .flatten()
        .map(|e| {
            rsx! {
                p { class: "text-red-500", {e} }
            }
        });

    let nav = use_navigator();

    match save {
        Ok(save) => {
            save_name.write().name = save.name.clone();
            rsx! {
                document::Title { "{save.name}" }

                div { class: "flex flex-col",
                    div { class: "flex flex-row justify-between items-center p-4",
                        h1 { class: "text-4xl font-bold", "{save.name}" }
                        div { class: "flex flex-row gap-2 items-center",
                            p {
                                span { class: "font-bold", "{count}" }
                                " version(s)"
                            }
                            button {
                                class: "flex items-center justify-center w-8 h-8 rounded bg-blue-300 hover:bg-blue-400 cursor-pointer",
                                onclick: move |_| {
                                    save_access_open.set(true);
                                },
                                img { src: crate::icons::USER_KEY }
                            }

                            if USER().is_some_and(|u| u.id == save.owner) {
                                button {
                                    class: "flex items-center justify-center w-8 h-8 rounded bg-red-300 hover:bg-red-400 cursor-pointer",
                                    onclick: move |_| {
                                        delete_save_open.set(true);
                                    },
                                    img { src: crate::icons::TRASH }
                                }
                            }
                        }
                    }

                    hr { class: "my-1" }

                    {save_versions}

                    if modify {
                        button {
                            class: "fixed bottom-4 right-4 w-12 h-12 rounded-full bg-emerald-400 hover:bg-green-300 flex items-center justify-center cursor-pointer",
                            onclick: move |_| new_version_open.set(true),
                            img { src: crate::icons::CIRCLE_PLUS }
                        }
                    }

                    Dialog { open: new_version_open,
                        h2 { class: "text-2xl font-bold", "New Version" }

                        hr { class: "my-2" }

                        form {
                            class: "flex flex-col gap-4",
                            onsubmit: move |e: FormEvent| async move {
                                e.prevent_default();

                                let values = e.data().values();

                                let name = match &values[0].1 {
                                    FormValue::Text(s) => s,
                                    _ => unreachable!("Expected text input for label"),
                                };

                                if name.contains('/') || name.contains('\\') {
                                    crate::toast("Invalid Version Label".to_string(), rsx! {
                                        p { "Version label cannot contain '/' or '\\' characters." }
                                    });
                                    return;
                                }

                                if let Err(e) = api::create_version(id, e.into()).await {
                                    error!("Failed to create version: {e}");
                                }
                                save_versions_res.restart();
                                new_version_open.set(false);
                            },

                            input {
                                class: crate::INPUT_CLASS,
                                placeholder: "Label",
                                name: "label",
                                required: true,
                            }

                            input {
                                class: crate::INPUT_CLASS,
                                placeholder: "File",
                                name: "file",
                                multiple: false,
                                r#type: "file",
                                required: true,
                            }

                            div { class: "flex flex-row justify-between",

                                button {
                                    class: "px-4 py-2 bg-gray-400 rounded cursor-pointer hover:bg-gray-500",
                                    onclick: move |e| {
                                        e.prevent_default();
                                        new_version_open.set(false);
                                    },
                                    "Cancel"
                                }

                                button { class: "px-4 py-2 bg-emerald-400 rounded cursor-pointer hover:bg-green-300",
                                    "Create"
                                }
                            }
                        }
                    }

                    Dialog {
                        open: save_access_open,
                        class: "{crate::DIALOG_CLASS} min-w-max",
                        div { class: "flex flex-row justify-between gap-8 items-top min-w-max",
                            h2 { class: "text-2xl font-bold", "Manage Access" }
                            if USER().is_some_and(|u| u.id == save.owner) {
                                div { class: "flex flex-col gap-2",
                                    form {
                                        class: "flex flex-row gap-2 items-center",
                                        onsubmit: move |e: FormEvent| async move {
                                            e.prevent_default();

                                            debug!("Adding access to save {:?}", e.data());
                                            let username = match &e.data().values()[0].1 {
                                                FormValue::Text(s) => s.clone(),
                                                _ => unreachable!("Expected text input for username"),
                                            };

                                            add_new_access.call(username).await
                                        },
                                        input {
                                            class: crate::INPUT_CLASS,
                                            placeholder: "Username",
                                            name: "username",
                                            required: true,
                                        }

                                        button { class: "p-1 bg-emerald-300 rounded cursor-pointer hover:bg-green-200",
                                            img {
                                                class: "w-6 h-6 ",
                                                src: crate::icons::CIRCLE_PLUS,
                                            }
                                        }
                                    }
                                    {add_new_access_error}
                                }
                            }
                        }

                        hr { class: "my-2" }

                        SaveAccessList {
                            save_access_res,
                            save_id: id,
                            is_owner: USER().is_some_and(|u| u.id == save.owner),
                        }
                    }

                    if USER().is_some_and(|u| u.id == save.owner) {
                        ConfirmDialog {
                            open: delete_save_open,
                            title: "Delete Save".to_string(),
                            message: "Are you sure you want to delete this save? This action cannot be undone."
                                .to_string(),
                            on_confirm: move |_| {
                                async move {
                                    if let Err(e) = api::delete_save(id).await {
                                        error!("Failed to delete save: {e}");
                                    }
                                    nav.replace(crate::Route::Saves {});
                                }
                            },
                        }
                    }
                }
            }
        }
        Err(e) => rsx! {
            div { class: "p-4",
                h2 { "Error loading save details" }
                p { "{e}" }
            }
        },
    }
}

#[component]
fn VersionList(versions: ReadSignal<Vec<api::Version>>, modify: ReadSignal<bool>) -> Element {
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
fn VersionRow(version: api::Version, modify: ReadSignal<bool>) -> Element {
    let time_string = chrono::DateTime::from_timestamp(version.timestamp as i64, 0)
        .expect("Failed to convert date from unixepoch")
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    #[cfg(feature = "desktop")]
    let install_btn = rsx! {
        button {
            title: "Deploy",
            class: "bg-yellow-300 hover:bg-yellow-200 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
            img { src: crate::icons::INSTALL }
        }
    };

    #[cfg(not(feature = "desktop"))]
    let install_btn = rsx! {};

    let mut delete_open = use_signal(|| false);

    let mut version_list = use_context::<VersionProvider>();

    let mut delete_version = use_action(move || async move {
        api::delete_version(version.save_id, version.id).await?;
        version_list.restart();
        Ok(()) as Result<(), ServerFnError>
    });

    let save_name = use_context::<Signal<SaveName>>();

    const DOWNLOAD_CLASS: &str = "bg-cyan-400 hover:bg-teal-300 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center";
    #[cfg(not(feature = "desktop"))]
    let download_button = rsx! {
        Link {
            class: DOWNLOAD_CLASS,
            to: format!("/api/save/{}/{}/download", version.save_id, version.id),
            img { src: crate::icons::DOWNLOAD }
        }
    };

    let v2 = version.clone();

    #[cfg(feature = "desktop")]
    let download_button = rsx! {
        button {
            title: "Download",
            class: DOWNLOAD_CLASS,
            onclick: move |_| {
                let version = v2.clone();
                async move {
                    #[cfg(feature = "desktop")]
                    {
                        let name = save_name.peek().name.clone();
                        download_version(&name, &version).await;

                        crate::toast("Download Complete".to_string(), rsx! {
                            p { "Version {version.version} downloaded successfully." }
                            button {
                                class: "underline cursor-pointer",
                                onclick: move |_| {
                                    let path = get_version_path(&save_name.peek().name, &version);
                                    Command::new("explorer")
                                        .arg("/select,")
                                        .arg(path)
                                        .spawn()
                                        .expect("Failed to open file explorer")
                                        .wait()
                                        .expect("Failed to wait for file explorer");
                                },
                                "View in Explorer"
                            }
                        });
                    }
                }
            },
            img { src: crate::icons::DOWNLOAD }
        }

    };

    rsx! {
        div { class: "grid grid-cols-subgrid col-span-full py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700 items-center",

            span { "{version.label}" }
            span { class: "text-center", "{version.version}" }
            span { class: "text-center", {time_string} }
            span { class: "text-center", "{version.by.username}" }
            {download_button}
            {install_btn}
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
                    version.version,
                    version.label,
                ),
                on_confirm: move |_| {
                    delete_version.call();
                },
                open: delete_open,
            }
        }
    }
}

type SaveListProvider = Resource<Result<api::SaveAccess, ServerFnError>>;

#[component]
fn SaveAccessList(save_access_res: SaveListProvider, save_id: i32, is_owner: bool) -> Element {
    let save_access = save_access_res().and_then(|res| res.ok());

    let owner = save_access.as_ref().map(|a| {
        rsx! {
            div { class: "grid grid-cols-subgrid col-span-full p-2 items-center",
                span { "{a.owner.username}" }
                span { class: "font-bold text-center", "Owner" }
            }
            hr { class: "col-span-full" }
        }
    });

    let save_list = save_access.map(|a| {
        rsx! {
            for access in a.access_list {
                SaveAccessRow {
                    key: "{access.user.id}",
                    access,
                    save_id,
                    save_access_res,
                    is_owner,
                }
            }
        }
    });

    rsx! {
        div { class: "grid grid-cols-[1fr_auto_auto] gap-x-4 border-b border-neutral-500 mb-2 items-center max-h-[80dvh] overflow-y-auto",
            {owner}
            {save_list}
        }
    }
}

#[component]
fn SaveAccessRow(
    access: api::NamedUserAccess,
    save_id: i32,
    save_access_res: SaveListProvider,
    is_owner: bool,
) -> Element {
    let username = access.user.username.clone();
    let mut remove_access = use_action(move || {
        let username = username.clone();
        async move {
            if let Err(e) = api::remove_user_save_access(save_id, username).await {
                error!("Failed to remove access: {e}");
                return match e {
                    ServerFnError::ServerError { message, .. } => Ok(Some(message)),
                    _ => Ok(Some("Failed to remove access".to_string())),
                };
            }
            save_access_res.restart();
            Ok(None) as Result<Option<String>, ServerFnError>
        }
    });

    rsx! {
        div {
            class: "grid grid-cols-subgrid col-span-full p-2 items-center cursor-pointer hover:bg-neutral-600 odd:bg-neutral-800",
            onclick: move |_| {
                let username = access.user.username.clone();
                async move {
                    if let Err(e) = api::update_user_save_access(
                            save_id,
                            username,
                            if matches!(access.access, api::UserAccess::View) {
                                api::UserAccess::Edit
                            } else {
                                api::UserAccess::View
                            },
                        )
                        .await
                    {
                        error!("Failed to update access: {e}");
                    }
                    save_access_res.restart();
                }
            },
            span { "{access.user.username}" }
            span { class: "flex justify-center items-center",
                if is_owner {
                    button {
                        title: "Toggle Access",
                        class: "bg-blue-300 hover:bg-blue-400 cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                        img {
                            src: match access.access {
                                api::UserAccess::View => crate::icons::EYE,
                                api::UserAccess::Edit => crate::icons::PENCIL,
                                _ => unreachable!("Invalid access level for user: {:?}", access.access),
                            },
                        }
                    }
                } else {
                    span { class: "font-bold text-center", "{access.access}" }
                }
            }
            if is_owner {
                button {
                    title: "Revoke Access",
                    class: "bg-red-300 hover:bg-red-400 cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                    onclick: move |e| {
                        e.stop_propagation();
                        remove_access.call();
                    },
                    img { src: crate::icons::TRASH }
                }
            }
        }
    }
}

#[cfg(feature = "desktop")]
fn get_version_cache_dir() -> std::path::PathBuf {
    use std::env;

    let mut cache_dir = env::current_exe().expect("Failed to get current exe path");
    cache_dir.pop(); // Remove the executable name
    cache_dir.push("downloads");
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).expect("Failed to create version cache directory");
    }
    cache_dir
}

#[cfg(feature = "desktop")]
fn get_version_path(save_name: &str, version: &api::Version) -> std::path::PathBuf {
    let cache_dir = get_version_cache_dir();
    cache_dir
        .join(save_name)
        .join(format!("{}.zip", version.version))
}

#[cfg(feature = "desktop")]
async fn download_version(save_name: &str, version: &api::Version) {
    let mut stream = match api::download_version(version.save_id, version.id).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to download version: {e}");
            return;
        }
    };
    debug!("Stream: {:?}", stream);
    let mut bytes = Vec::new();
    while let Some(Ok(chunk)) = stream.next().await {
        bytes.extend_from_slice(&chunk);
    }

    use std::fs::File;
    use std::io::Write;

    let file_path = get_version_path(save_name, version);
    if let Some(parent) = file_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).expect("Failed to create directories for zip file");
    }
    debug!("Saving version {} to {:?}", version.version, file_path);
    let mut file = File::create(&file_path).expect("Failed to create zip file");
    file.write_all(&bytes).expect("Failed to write zip file");
}
