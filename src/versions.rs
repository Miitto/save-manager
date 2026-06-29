use crate::prelude::*;
use api::UserAccessExt;

mod list;
use list::VersionList;

type VersionProvider = Loader<Vec<api::Version>>;

#[component]
pub fn SaveDetails(id: ReadSignal<i32>) -> Element {
    let save_res = use_server_future(move || api::get_save_details(id()))?;
    let save_r = save_res().ok_or(anyhow::anyhow!("Failed to load save details"))??;
    let save = use_signal(|| save_r);
    let versions = use_loader(move || api::get_save_versions(id()))?;

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
}

#[component]
fn NewVersionDialog(id: ReadSignal<i32>, new_version_open: Signal<bool>) -> Element {
    let mut label = use_signal(String::new);
    let file = use_signal(|| None::<dioxus::html::FileData>);

    use_context_provider(move || NewVersionContext { file });

    let mut save_versions_res = use_context::<VersionProvider>();

    let make_data = move || {
        struct Data {
            pub label: String,
            pub file: Option<dioxus::html::FileData>,
        }

        impl dioxus::html::HasFileData for Data {
            fn files(&self) -> Vec<dioxus::html::FileData> {
                self.file.clone().into_iter().collect()
            }
        }

        impl HasFormData for Data {
            fn valid(&self) -> bool {
                true
            }
            fn value(&self) -> String {
                panic!("This should never be called, as we are using a custom form data handler");
            }
            fn values(&self) -> Vec<(String, FormValue)> {
                vec![
                    ("label".to_string(), FormValue::Text(self.label.clone())),
                    ("file".to_string(), FormValue::File(self.file.clone())),
                ]
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        let data = Data {
            label: label.cloned(),
            file: file.cloned(),
        };

        let form_data = dioxus::html::FormData::new(data);

        FormEvent::new(std::rc::Rc::new(form_data), false)
    };

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

                    let data = e.data();

                    if !data.files().is_empty() {
                        if let Err(e) = api::create_version(id(), e.into()).await {
                            error!("Failed to create version: {e}");
                        }
                    } else {
                        let data = make_data();

                        if let Err(e) = api::create_version(id(), data.into()).await {
                            error!("Failed to create version: {e}");
                        }
                    }
                    save_versions_res.restart();
                    new_version_open.set(false);
                },

                Input {
                    placeholder: "Label",
                    name: "label",
                    required: true,
                    value: label(),
                    oninput: move |e: FormEvent| label.set(e.value()),
                }

                NewVersionFileSelection {}

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
            class: "grid grid-cols-subgrid col-span-full p-2 items-center cursor-pointer hover:bg-white/10 odd:bg-white/5",
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
fn NewVersionFileSelection() -> Element {
    let save = use_context::<Signal<api::Save>>();
    let game = use_memo(move || save.read().game);
    let can_fetch = use_memo(move || game().can_fetch());
    let mut auto = use_signal(|| can_fetch.cloned());

    let ui = use_memo(move || {
        if auto() {
            use crate::versions::desktop_ui::NewVersionGameOptions;
            rsx! {
                NewVersionGameOptions {}
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
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
fn NewVersionFileSelection() -> Element {
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

#[cfg(feature = "desktop")]
mod desktop_ui {
    use dioxus::core::SuperInto;

    use crate::{
        desktop::{DeployOptions, DeployOptionsStoreExt},
        prelude::*,
        versions::NewVersionContext,
    };

    #[component]
    pub fn NewVersionGameOptions() -> Element {
        let save = use_context::<Signal<api::Save>>();

        let NewVersionContext { mut file, .. } = use_context::<NewVersionContext>();

        let game = use_memo(move || save.read().game);

        let deps = use_store(|| crate::desktop::DeployOptions::from(game()));

        let mut path = use_signal(std::path::PathBuf::new);

        let set_path = move |p: std::path::PathBuf| {
            path.set(p);
        };

        let go = use_memo(move || match game() {
            api::Game::IntoTheRadius2 => rsx! {
                IntoTheRadiusNewVersionOptions { deps, set_path }
            },
            api::Game::Satisfactory => rsx! {
                SatisfactoryNewVersionOptions { deps }
            },
        });

        let mut locked = use_signal(|| false);

        let mut set_file_data = move || {
            let path = path.cloned();
            locked.set(true);
            async move {
                if !path.exists() {
                    return None;
                }

                struct FileData(pub std::path::PathBuf);

                impl dioxus::html::NativeFileData for FileData {
                    fn name(&self) -> String {
                        self.0.file_name().unwrap().to_string_lossy().into_owned()
                    }

                    fn size(&self) -> u64 {
                        std::fs::metadata(&self.0).map(|m| m.len()).unwrap_or(0)
                    }

                    fn last_modified(&self) -> u64 {
                        std::fs::metadata(&self.0)
                            .and_then(|m| m.modified())
                            .ok()
                            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|duration| duration.as_secs())
                            .unwrap_or(0)
                    }

                    fn read_bytes(
                        &self,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<bytes::Bytes, dioxus_core::CapturedError>,
                                > + 'static,
                        >,
                    > {
                        let path = self.0.clone();
                        Box::pin(async move { Ok(bytes::Bytes::from(std::fs::read(&path)?)) })
                    }

                    fn read_string(
                        &self,
                    ) -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<String, dioxus_core::CapturedError>,
                                > + 'static,
                        >,
                    > {
                        let path = self.0.clone();
                        Box::pin(async move { Ok(std::fs::read_to_string(&path)?) })
                    }

                    fn inner(&self) -> &dyn std::any::Any {
                        &self.0
                    }

                    fn path(&self) -> std::path::PathBuf {
                        self.0.clone()
                    }

                    fn byte_stream(
                        &self,
                    ) -> std::pin::Pin<
                        Box<
                            dyn futures_util::Stream<
                                    Item = Result<bytes::Bytes, dioxus_core::CapturedError>,
                                >
                                + 'static
                                + Send,
                        >,
                    > {
                        let path = self.0.clone();
                        Box::pin(futures_util::stream::once(async move {
                            Ok(bytes::Bytes::from(std::fs::read(&path)?))
                        }))
                    }

                    fn content_type(&self) -> Option<String> {
                        Some(
                            dioxus::asset_resolver::native::get_mime_from_ext(
                                self.0.extension().and_then(|ext| ext.to_str()),
                            )
                            .to_string(),
                        )
                    }
                }

                let file_data = dioxus::html::FileData::new(FileData(path));

                file.set(Some(file_data));

                None as Option<()>
            }
        };

        rsx! {
            if locked() {
                span { class: "text-white/50", "{path().display()}" }
                Button {
                    size: ButtonSize::Lg,
                    onclick: move |e: MouseEvent| {
                        e.prevent_default();
                        file.set(None);
                        path.set(std::path::PathBuf::new());
                        locked.set(false);
                    },
                    "Clear"
                }
            } else {
                {go}

                Button {
                    size: ButtonSize::Lg,
                    onclick: move |e: MouseEvent| async move {
                        e.prevent_default();

                        set_file_data().await;
                    },
                    "Select"
                }
            }
        }
    }

    #[component]
    pub fn IntoTheRadiusNewVersionOptions(
        deps: Store<DeployOptions>,
        set_path: Callback<std::path::PathBuf>,
    ) -> Element {
        use crate::desktop::{self, into_the_radius_2::*};
        if !deps.is_into_the_radius_2() {
            deps.set(DeployOptions::IntoTheRadius2(Default::default()));
        }

        let itr = deps.into_the_radius_2().unwrap();

        let is_coop: ReadSignal<Option<bool>> = use_memo(move || Some(itr.coop()())).super_into();
        let slot: ReadSignal<Option<SaveSlots>> = use_memo(move || Some(itr.slot()())).super_into();

        let mut error_msg = use_signal(|| None::<String>);

        use_effect(move || {
            let coop = itr.coop()();
            let slot = itr.slot()();

            let save_dir = desktop::dirs::get_game_save_dir(api::Game::IntoTheRadius2);
            if !save_dir.exists() {
                error_msg.set(Some(format!(
                    "Save directory does not exist: {}",
                    save_dir.display()
                )));
            }

            let subfolder = if coop { "Coop" } else { "Single" };
            let slot_path = save_dir.join(subfolder).join(slot.name());

            set_path(slot_path);
        });

        rsx! {
            div { class: "flex flex-row justify-between gap-4 items-center",
                Select::<bool> {

                    value: is_coop,
                    width: "12rem",
                    on_value_change: move |coop: Option<bool>| itr.coop().set(coop.unwrap_or(false)),
                    SelectOption::<bool> {
                        index: 1usize,
                        value: false,
                        text_value: "Singleplayer",
                        "Singleplayer"
                    }
                    SelectOption::<bool> { index: 1usize, value: true, text_value: "Coop", "Coop" }
                }

                Select::<SaveSlots> {
                    class: "flex flex-row justify-end",
                    value: slot,
                    width: "12rem",
                    on_value_change: move |slot: Option<SaveSlots>| itr.slot().set(slot.unwrap_or_default()),
                    SelectOption::<SaveSlots> {
                        index: 1usize,
                        value: SaveSlots::Slot1,
                        text_value: "Save 1",
                        "Save 1"
                    }
                    SelectOption::<SaveSlots> {
                        index: 1usize,
                        value: SaveSlots::Slot2,
                        text_value: "Save 2",
                        "Save 2"
                    }
                    SelectOption::<SaveSlots> {
                        index: 2usize,
                        value: SaveSlots::Slot3,
                        text_value: "Save 3",
                        "Save 3"
                    }
                    SelectOption::<SaveSlots> {
                        index: 3usize,
                        value: SaveSlots::AutoSave1,
                        text_value: "Autosave 1",
                        "Autosave 1"
                    }
                    SelectOption::<SaveSlots> {
                        index: 4usize,
                        value: SaveSlots::AutoSave2,
                        text_value: "Autosave 2",
                        "Autosave 2"
                    }
                    SelectOption::<SaveSlots> {
                        index: 5usize,
                        value: SaveSlots::AutoSave3,
                        text_value: "Autosave 3",
                        "Autosave 3"
                    }
                }
            }

            if let Some(error) = error_msg() {
                p { class: "text-red-500", {error} }
            }
        }
    }

    #[component]
    pub fn SatisfactoryNewVersionOptions(deps: Store<DeployOptions>) -> Element {
        rsx! {}
    }
}
