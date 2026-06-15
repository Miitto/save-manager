use dioxus::prelude::*;

use api::UserAccessExt;

use crate::{ConfirmDialog, Dialog, USER};

type VersionProvider = Resource<Result<Vec<api::Version>, ServerFnError>>;

#[component]
pub fn SaveDetails(id: i32) -> Element {
    let save = use_server_future(move || api::get_save_details(id))?().unwrap();
    let mut save_versions_res = use_server_future(move || api::get_save_versions(id))?;

    use_context_provider::<VersionProvider>(|| save_versions_res);

    let save_version_list = save_versions_res().unwrap();

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

    let nav = use_navigator();

    match save {
        Ok(save) => rsx! {
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
                            class: "flex items-center justify-center w-8 h-8 rounded bg-red-300 hover:bg-red-400 cursor-pointer",
                            onclick: move |_| {
                                delete_save_open.set(true);
                            },
                            img { src: crate::icons::TRASH }
                        }
                    }
                }

                hr { class: "my-1" }

                {save_versions}

                button {
                    class: "fixed bottom-4 right-4 w-12 h-12 rounded-full bg-emerald-400 hover:bg-green-300 flex items-center justify-center cursor-pointer",
                    onclick: move |_| new_version_open.set(true),
                    img { src: crate::icons::CIRCLE_PLUS }
                }

                Dialog { open: new_version_open,
                    h2 { class: "text-2xl font-bold", "New Version" }

                    hr { class: "my-2" }

                    form {
                        class: "flex flex-col gap-4",
                        onsubmit: move |e: FormEvent| async move {
                            e.prevent_default();

                            if let Err(e) = api::create_version(id, e.into()).await {
                                error!("Failed to create version: {e}");
                            }
                            save_versions_res.restart();
                            new_version_open.set(false);
                        },

                        crate::Input { placeholder: "Label", name: "label", req: true }

                        crate::Input {
                            placeholder: "File",
                            name: "file",
                            mul: false,
                            r#type: "file",
                            req: true,
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
        },
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
    let time_string = version
        .timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let datetime = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .expect("Failed to convert date from unixepoch");
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| "Invalid Timestamp".to_string());

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

    rsx! {
        div { class: "grid grid-cols-subgrid col-span-full py-2 px-4 hover:bg-neutral-600 odd:bg-neutral-700 items-center",

            span { "{version.label}" }
            span { class: "text-center", "{version.version}" }
            span { class: "text-center", {time_string} }
            span { class: "text-center", "{version.by.username}" }
            button {
                title: "Download",
                class: "bg-cyan-400 hover:bg-teal-300 hover:cursor-pointer rounded w-8 h-8 flex justify-center items-center",
                img { src: crate::icons::DOWNLOAD }
            }
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
