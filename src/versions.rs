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

        div { class: "flex flex-col max-w-screen",
            div { class: "flex flex-row justify-between items-center p-4 max-w-full",
                h1 { class: "text-4xl font-bold overflow-ellipsis overflow-hidden",
                    "{save().name}"
                }
                div { class: "flex flex-row gap-2 items-center",
                    p { class: "whitespace-nowrap",
                        span { class: "font-bold ", "{count}" }
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

            VersionList {
                versions: versions.store().transpose().expect("Version list to have value"),
                modify,
            }

            if modify {
                Button {
                    size: ButtonSize::IconLg,
                    class: "fixed bottom-4 right-4",
                    onclick: move |_| new_version_open.set(true),
                    icons::CirclePlus {}
                }

                NewVersionDialog { id, new_version_open }
            }

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
fn NewVersionDialog(id: ReadSignal<i32>, new_version_open: Signal<bool>) -> Element {
    let mut label = use_signal(String::new);

    let mut error = use_signal(|| None::<String>);

    let mut version_list = use_context::<VersionProvider>();
    let (file_select, get_file_data) = NewVersionFileSelection();
    let make_data = move |file: dioxus::html::FileData| {
        let data = custom_types::Data {
            label: label.cloned(),
            file: Some(file),
        };

        let form_data = dioxus::html::FormData::new(data);

        FormEvent::new(std::rc::Rc::new(form_data), false)
    };

    let submit = move |e: FormEvent| {
        let get_file_data = get_file_data();
        async move {
            e.prevent_default();

            let data = e.data();

            let multipart: dioxus::fullstack::MultipartFormData = if data.values().len() > 1 {
                e.into()
            } else {
                let data = match get_file_data {
                    Ok(d) => make_data(d),
                    Err(e) => {
                        error!("Failed to make data: {e}");
                        error.set(Some(e));
                        return;
                    }
                };
                data.into()
            };

            match api::create_version(id(), multipart).await {
                Ok(v) => {
                    version_list.write().insert(0, v);
                    new_version_open.set(false);
                    label.write().clear();
                }
                Err(e) => {
                    error!("Failed to create version: {e}");
                    match e {
                        ServerFnError::ServerError { message, .. } => error.set(Some(message)),
                        _ => error.set(Some("Failed to create version".to_string())),
                    }
                }
            }
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
                    maxlength: 50,
                    oninput: move |e: FormEvent| label.set(e.value()),
                }

                {file_select}

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

    use_context_provider::<SaveAccessProvider>(|| save_access);

    let write = save_access
        .store()
        .transpose()
        .ok_or(anyhow::anyhow!("Save access list to have value"))?;

    let mut error = use_signal(|| None::<String>);

    let add_new_access = move |username: String| {
        let new_row = api::NamedUserAccess {
            user: api::UserPreview {
                id: 0,
                username: username.clone(),
            },
            access: api::UserAccess::View,
        };

        let mut access_list = write.access_list();
        let pos = access_list
            .read()
            .binary_search_by(|a| a.user.username.cmp(&username));

        async move {
            if let Err(pos) = pos {
                access_list.write().insert(pos, new_row);
            } else {
                error!("User already has access");
                error.set(Some("User already has access".into()));
            }

            if let Err(e) = api::add_user_save_access(id(), username).await {
                error!("Failed to add access: {e}");
                save_access.restart();

                match e {
                    ServerFnError::ServerError { message, .. } => error.set(Some(message)),
                    _ => error.set(Some("Failed to add access".to_string())),
                }
            }
        }
    };

    let error_rsx = use_memo(move || {
        error().map(|e| {
            rsx! {
                p { class: "text-red-500", {e} }
            }
        })
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
                                    _ => {
                                        error!("Expected text input for form value");
                                        String::new()
                                    }
                                };

                                if username.trim().is_empty() {
                                    return;
                                }

                                add_new_access(username).await
                            },
                            Input {
                                placeholder: "Username",
                                name: "username",
                                required: true,
                            }

                            Button { size: ButtonSize::Icon, icons::CirclePlus {} }
                        }
                        {error_rsx}
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

    let write = save_access
        .store()
        .transpose()
        .ok_or(anyhow::anyhow!("Save access list to have value"))?;

    let remove_access = move |id: api::UserId| {
        write.access_list().retain(|access| access.user.id != id);
    };

    rsx! {
        div { class: "grid grid-cols-[1fr_auto_auto] gap-x-4 border-b border-border mb-2 items-center max-h-[80dvh] overflow-y-auto",
            {owner}
            for access in save_access
                .store()
                .transpose()
                .expect("Save access list to have value")
                .access_list()
                .iter()
            {
                SaveAccessRow {
                    key: "{access.user().id()}",
                    access,
                    save_id,
                    is_owner,
                    remove_access: move |_| {
                        remove_access(access.read().user.id);
                    },
                }
            }
        }
    }
}

#[component]
fn SaveAccessRow(
    access: WriteStore<api::NamedUserAccess>,
    save_id: ReadSignal<i32>,
    is_owner: bool,
    remove_access: Callback<()>,
) -> Element {
    let toast_api = use_toast();
    let mut save_access = use_context::<SaveAccessProvider>();
    let remove_access = move || {
        let username = access.user().username().cloned();
        remove_access.call(());
        async move {
            if let Err(e) = api::remove_user_save_access(save_id(), username).await {
                error!("Failed to remove access: {e}");
                save_access.restart();
                toast_api.error(
                    "Failed to remove access".to_string(),
                    ToastOptions::new().description(e.to_string()),
                );
            }
        }
    };

    rsx! {
        div { class: "grid grid-cols-subgrid col-span-full p-2 items-center odd:bg-white/10",
            span { "{access.user().username()}" }
            if is_owner {
                span { class: "flex justify-center items-center",
                    Button {
                        title: "Current Access",
                        size: ButtonSize::Icon,
                        onclick: move |_| {
                            let original_access = access.read().access;
                            access.access().toggle();
                            let username = access.user().username().cloned();
                            async move {
                                if let Err(e) = api::update_user_save_access(
                                        save_id(),
                                        username,
                                        if matches!(original_access, api::UserAccess::View) {
                                            api::UserAccess::Edit
                                        } else {
                                            api::UserAccess::View
                                        },
                                    )
                                    .await
                                {
                                    error!("Failed to update access: {e}");
                                    access.access().set(original_access);
                                    toast_api
                                        .error(
                                            "Failed to update access".to_string(),
                                            ToastOptions::new().description(e.to_string()),
                                        );
                                }
                            }
                        },
                        if matches!(access.read().access, api::UserAccess::View) {
                            icons::Eye {}
                        } else {
                            icons::Pencil {}
                        }
                    }
                }
            } else {
                span { class: "font-bold text-right col-span-2", "{access.access()}" }
            }
            if is_owner {
                Button {
                    size: ButtonSize::Icon,
                    title: "Revoke Access",
                    onclick: move |e: MouseEvent| async move {
                        e.stop_propagation();
                        remove_access().await;
                    },
                    icons::Trash2 {}
                }
            }
        }
    }
}
