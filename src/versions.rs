use crate::prelude::*;
use api::{NamedUserAccessStoreExt, SaveAccessStoreExt, UserAccessExt, UserPreviewStoreExt};

pub mod custom_types;
mod list;
mod new;
pub use list::VersionList;
use new::*;

type VersionProvider = LoaderStore<Vec<api::Version>>;

#[component]
pub fn SaveDetails(id: ReadSignal<i32>) -> Element {
    let save_res = use_server_future(move || api::get_save_details(id()))?;
    let save_r = save_res().ok_or(anyhow::anyhow!("Failed to load save details"))??;
    let save = use_signal(|| save_r);
    let versions = use_loader_store(move || api::get_save_versions(id()))?;

    use_context_provider::<VersionProvider>(|| versions);

    use_context_provider(|| save);

    let modify = use_server_future(move || {
        _ = USER();
        async move { api::get_user_save_access(id()).await.map(|a| a.can_edit()) }
    })?()
    .map(|r| r.unwrap_or(false))
    .unwrap_or(false);

    let count = use_memo(move || versions.read().len());

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

            VersionList { versions, modify }

            if modify {
                Button {
                    size: ButtonSize::IconLg,
                    class: "fixed bottom-4 right-4",
                    onclick: move |_| new_version_open.set(true),
                    icons::CirclePlus {}
                }
            }

            NewVersionDialog { id, new_version_open }

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

#[derive(Clone)]
struct NewVersionContext {
    pub file: Signal<Option<dioxus::html::FileData>>,
    pub error: Signal<Option<String>>,
}

#[component]
fn NewVersionDialog(id: ReadSignal<i32>, new_version_open: Signal<bool>) -> Element {
    let mut label = use_signal(String::new);
    let mut file = use_signal(|| None::<dioxus::html::FileData>);
    let mut error = use_signal(|| None::<String>);

    use_context_provider(move || NewVersionContext { file, error });

    let mut save_versions_res = use_context::<VersionProvider>();

    let make_data = move || {
        let data = custom_types::Data {
            label: label.cloned(),
            file: file.cloned(),
        };

        let form_data = dioxus::html::FormData::new(data);

        FormEvent::new(std::rc::Rc::new(form_data), false)
    };

    let submit = move |e: FormEvent| async move {
        e.prevent_default();

        let data = e.data();

        let multipart: dioxus::fullstack::MultipartFormData = if data.values().len() > 1 {
            e.into()
        } else {
            if file.read().is_none() {
                error.set(Some("No file selected".to_string()));
                return;
            }
            let data = make_data();
            data.into()
        };

        if let Err(e) = api::create_version(id(), multipart).await {
            error!("Failed to create version: {e}");
            error.set(Some(format!("Failed to create version: {e}")));
        } else {
            save_versions_res.restart();
            new_version_open.set(false);
            label.write().clear();
            file.set(None);
        }
    };

    rsx! {
        Dialog {
            open: new_version_open(),
            on_open_change: move |open| new_version_open.set(open),
            class: "min-w-max",
            h2 { class: "text-2xl font-bold", "New Version" }

            Separator {}

            form { class: "flex flex-col gap-4", onsubmit: submit,
                Input {
                    placeholder: "Label",
                    name: "label",
                    required: true,
                    value: label(),
                    oninput: move |e: FormEvent| label.set(e.value()),
                }

                NewVersionFileSelection {}

                if let Some(e) = error() {
                    p { class: "text-red-500", {e} }
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

type SaveAccessProvider = LoaderStore<api::SaveAccess>;

#[component]
fn SaveAccessDialog(
    id: ReadSignal<i32>,
    save_access_open: Signal<bool>,
    owner: ReadSignal<i32>,
) -> Element {
    let mut save_access = use_loader_store(move || api::get_save_access(id()))?;

    let mut add_new_access = use_action(move |username: String| async move {
        if let Err(e) = api::add_user_save_access(id(), username).await {
            error!("Failed to add access: {e}");
            return match e {
                ServerFnError::ServerError { message, .. } => Ok(Some(message)),
                _ => Ok(Some("Failed to add access".to_string())),
            };
        }
        save_access.restart();
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
                save_access,
                save_id: id,
                is_owner: USER().is_some_and(|u| u.id == owner()),
            }

        }
    }
}

#[component]
pub fn SaveAccessList(
    save_access: SaveAccessProvider,
    save_id: ReadSignal<i32>,
    is_owner: bool,
) -> Element {
    let owner = rsx! {
        div { class: "grid grid-cols-subgrid col-span-full p-2 items-center",
            span { "{save_access.read_store().owner().username()}" }
            span { class: "font-bold text-center col-span-2", "Owner" }
        }
        Separator { class: "col-span-full" }
    };

    rsx! {
        div { class: "grid grid-cols-[1fr_auto_auto] gap-x-4 border-b border-border mb-2 items-center max-h-[80dvh] overflow-y-auto",
            {owner}
            for access in save_access.read_store().access_list().iter() {
                SaveAccessRow {
                    key: "{access.user().id()}",
                    access,
                    save_id,
                    is_owner,
                }
            }
        }
    }
}

#[component]
fn SaveAccessRow(
    access: ReadStore<api::NamedUserAccess>,
    save_id: ReadSignal<i32>,
    is_owner: bool,
) -> Element {
    let mut save_access = use_context::<SaveAccessProvider>();
    let mut remove_access = use_action(move || {
        let username = access.user().username().cloned();
        async move {
            if let Err(e) = api::remove_user_save_access(save_id(), username).await {
                error!("Failed to remove access: {e}");
                return match e {
                    ServerFnError::ServerError { message, .. } => Ok(Some(message)),
                    _ => Ok(Some("Failed to remove access".to_string())),
                };
            }
            save_access.restart();
            Ok(None) as Result<Option<String>, ServerFnError>
        }
    });

    rsx! {
        div {
            class: "grid grid-cols-subgrid col-span-full p-2 items-center cursor-pointer hover:bg-white/10 odd:bg-white/5",
            onclick: move |_| {
                async move {
                    if let Err(e) = api::update_user_save_access(
                            save_id(),
                            access.read().user.username.clone(),
                            if matches!(access.read().access, api::UserAccess::View) {
                                api::UserAccess::Edit
                            } else {
                                api::UserAccess::View
                            },
                        )
                        .await
                    {
                        error!("Failed to update access: {e}");
                    }
                    save_access.restart();
                }
            },
            span { "{access.user().username()}" }
            span { class: "flex justify-center items-center",
                if is_owner {
                    Button { title: "Current Access", size: ButtonSize::Icon,
                        if matches!(access.read().access, api::UserAccess::View) {
                            icons::Eye {}
                        } else {
                            icons::Pencil {}
                        }
                    }
                } else {
                    span { class: "font-bold text-center", "{access.access()}" }
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
