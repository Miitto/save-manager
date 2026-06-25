use crate::prelude::*;
use api::UserAccessExt;

mod list;
use list::VersionList;

type VersionProvider = Resource<Result<Vec<api::Version>, ServerFnError>>;

#[component]
pub fn SaveDetails(id: ReadSignal<i32>) -> Element {
    let save_res = use_server_future(move || api::get_save_details(id()))?;
    let save_r = save_res().ok_or(anyhow::anyhow!("Failed to load save details"))??;
    let save = use_signal(|| save_r);
    let save_versions_res = use_server_future(move || api::get_save_versions(id()))?;

    use_context_provider::<VersionProvider>(|| save_versions_res);

    let save_version_list = save_versions_res().unwrap().map(|l| use_store(|| l));

    use_context_provider(|| save);

    let modify = use_server_future(move || {
        _ = USER();
        async move { api::get_user_save_access(id()).await.map(|a| a.can_edit()) }
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

    let nav = use_navigator();

    rsx! {
        document::Title { "{save().name}" }

        div { class: "flex flex-col",
            div { class: "flex flex-row justify-between items-center p-4",
                h1 { class: "text-4xl font-bold", "{save().name}" }
                div { class: "flex flex-row gap-2 items-center",
                    p {
                        span { class: "font-bold", "{count}" }
                        " version(s)"
                    }
                    Button {
                        size: ButtonSize::Icon,
                        onclick: move |_| {
                            save_access_open.set(true);
                        },
                        icons::UserKey {}
                    }

                    if USER().is_some_and(|u| u.id == save().owner) {
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Icon,
                            onclick: move |_| {
                                delete_save_open.set(true);
                            },
                            icons::Trash2 {}
                        }
                    }
                }
            }

            Separator {}

            {save_versions}

            if modify {
                Button {
                    size: ButtonSize::IconLg,
                    class: "fixed bottom-4 right-4",
                    onclick: move |_| new_version_open.set(true),
                    icons::CirclePlus {}
                }
            }

            NewVersionDialog { id, new_version_open, save_versions_res }

            SaveAccessDialog { id, save_access_open, owner: save().owner }

            if USER().is_some_and(|u| u.id == save().owner) {
                AlertDialog {
                    open: delete_save_open(),
                    on_open_change: move |open| delete_save_open.set(open),
                    AlertDialogTitle { "Delete Save" }
                    AlertDialogDescription { "Are you sure you want to delete this save? This action cannot be undone." }
                    AlertDialogActions {
                        AlertDialogCancel { "Cancel" }
                        AlertDialogAction {
                            on_click: move |_| {
                                async move {
                                    if let Err(e) = api::delete_save(id()).await {
                                        error!("Failed to delete save: {e}");
                                    }
                                    nav.replace(crate::Route::Saves {});
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NewVersionDialog(
    id: ReadSignal<i32>,
    new_version_open: Signal<bool>,
    save_versions_res: VersionProvider,
) -> Element {
    let toast_api = use_toast();

    rsx! {
        Dialog {
            open: new_version_open(),
            on_open_change: move |open| new_version_open.set(open),
            class: "min-w-max",
            h2 { class: "text-2xl font-bold", "New Version" }

            Separator {}

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
                        toast_api
                            .error(
                                "Invalid Version Label".to_string(),
                                ToastOptions::new()
                                    .description(
                                        "Version label cannot contain '/' or '\\' characters.",
                                    ),
                            );
                        return;
                    }
                    if let Err(e) = api::create_version(id(), e.into()).await {
                        error!("Failed to create version: {e}");
                    }
                    save_versions_res.restart();
                    new_version_open.set(false);
                },

                Input { placeholder: "Label", name: "label", required: true }

                Input {
                    placeholder: "File",
                    name: "file",
                    multiple: false,
                    r#type: "file",
                    required: true,
                }

                div { class: "flex flex-row justify-between",

                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Lg,
                        onclick: move |e: MouseEvent| {
                            e.prevent_default();
                            new_version_open.set(false);
                        },
                        "Cancel"
                    }

                    Button { size: ButtonSize::Lg, "Create" }
                }
            }
        }
    }
}

type SaveListProvider = Resource<Result<api::SaveAccess, ServerFnError>>;

#[component]
fn SaveAccessDialog(
    id: ReadSignal<i32>,
    save_access_open: Signal<bool>,
    owner: ReadSignal<i32>,
) -> Element {
    let mut save_access_res =
        use_server_future(move || async move { api::get_save_access(id()).await })?;

    let mut add_new_access = use_action(move |username: String| async move {
        if let Err(e) = api::add_user_save_access(id(), username).await {
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

    rsx! {
        Dialog {
            open: save_access_open(),
            on_open_change: move |open| save_access_open.set(open),
            class: "min-w-max",
            div { class: "flex flex-row justify-between gap-8 items-top min-w-max",
                h2 { class: "text-2xl font-bold", "Manage Access" }
                if USER().is_some_and(|u| u.id == owner()) {
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
                            Input {
                                placeholder: "Username",
                                name: "username",
                                required: true,
                            }

                            Button { size: ButtonSize::Icon, icons::CirclePlus {} }
                        }
                        {add_new_access_error}
                    }
                }
            }

            Separator {}

            SaveAccessList {
                save_access_res,
                save_id: id,
                is_owner: USER().is_some_and(|u| u.id == owner()),
            }

        }
    }
}

#[component]
fn SaveAccessList(
    save_access_res: SaveListProvider,
    save_id: ReadSignal<i32>,
    is_owner: bool,
) -> Element {
    let save_access = save_access_res().and_then(|res| res.ok());

    let owner = save_access.as_ref().map(|a| {
        rsx! {
            div { class: "grid grid-cols-subgrid col-span-full p-2 items-center",
                span { "{a.owner.username}" }
                span { class: "font-bold text-center col-span-2", "Owner" }
            }
            Separator { class: "col-span-full" }
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
    save_id: ReadSignal<i32>,
    save_access_res: SaveListProvider,
    is_owner: bool,
) -> Element {
    let username = access.user.username.clone();
    let mut remove_access = use_action(move || {
        let username = username.clone();
        async move {
            if let Err(e) = api::remove_user_save_access(save_id(), username).await {
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
                            save_id(),
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
                    Button { title: "Current Access", size: ButtonSize::Icon,
                        if matches!(access.access, api::UserAccess::View) {
                            icons::Eye {}
                        } else {
                            icons::Pencil {}
                        }
                    }
                } else {
                    span { class: "font-bold text-center", "{access.access}" }
                }
            }
            if is_owner {
                Button {
                    size: ButtonSize::Icon,
                    title: "Revoke Access",
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        remove_access.call();
                    },
                    icons::Trash2 {}
                }
            }
        }
    }
}
