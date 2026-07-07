use crate::prelude::*;

#[component]
pub fn Index() -> Element {
    rsx! {
        document::Meta {
            name: "description",
            content: "A tool for sharing and versioning game saves.",
        }
        document::Meta {
            name: "keywords",
            content: "save, saves, game, versioning, sharing",
        }
        document::Meta { property: "og:title", content: "Save Manager" }
        document::Meta {
            property: "og:description",
            content: "A tool for sharing and versioning game saves.",
        }
        document::Meta { property: "og:url", content: "https://saves.miitto.dev" }

        div { class: " flex flex-col items-center",
            main { class: "flex flex-col max-w-7xl max-auto px-4 md:py-6 gap-4",
                h1 { class: "text-6xl font-bold", "Save Manager" }
                p { class: "text-2xl", "A tool for sharing and versioning game saves." }

                Separator {}

                div { class: "flex gap-4 flex-wrap justify-center",
                    div {
                        p { class: "text-lg mb-2", "Easily share save files within your group." }
                        SharePreview {}
                    }
                    div {
                        p { class: "text-lg mb-2", "Keep track of your save history." }
                        VersionPreview {}
                    }
                }

                div { class: "flex flex-col items-center gap-2 mt-8",
                    p { class: "text-lg mb-2",
                        "And with a desktop client to handle the file management for you."
                    }
                    ButtonLink { to: "https://github.com/Miito/save-manager/releases",
                        "Download"
                        img {
                            src: crate::icons::GITHUB_ICON_LIGHT,
                            class: "w-6 h-6",
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SharePreview() -> Element {
    let save_access = use_loader_store(|| async move {
        Ok::<_, ServerFnError>(api::SaveAccess {
            owner: api::UserPreview {
                id: 0,
                username: "John Doe".to_string(),
            },
            access_list: vec![
                api::NamedUserAccess {
                    user: api::UserPreview {
                        id: 1,
                        username: "Steve".to_string(),
                    },
                    access: api::UserAccess::View,
                },
                api::NamedUserAccess {
                    user: api::UserPreview {
                        id: 2,
                        username: "Jane".to_string(),
                    },
                    access: api::UserAccess::Edit,
                },
            ],
        })
    })?;

    use_context_provider(|| save_access);

    rsx! {
        div { class: "pointer-events-none select-none flex flex-col gap-2 border rounded-xl border-border p-4",
            div { class: "flex flex-row justify-between items-center gap-4",
                p { class: "text-lg font-bold", "Manage Access" }
                div { class: "flex flex-row justify-between items-center gap-2",
                    Input {
                        placeholder: "Username",
                        name: "username",
                        r#type: "text",
                        disabled: true,
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Icon,
                        crate::icons::CirclePlus {}
                    }
                }
            }
            Separator {}
            crate::versions::SaveAccessList { save_access, save_id: 0, is_owner: true }
        }
    }
}

#[component]
fn VersionPreview() -> Element {
    let save = use_signal(|| api::Save {
        id: 0,
        name: "My Save".to_string(),
        game: api::Game::Satisfactory,
        most_recent_version: None,
        version_count: 3,
        owner: 0,
    });

    let versions = use_mapped_loader_store(
        || async move {
            Ok::<_, ServerFnError>(vec![
                api::Version {
                    id: 0,
                    save_id: 0,
                    version: 3,
                    label: "Modular Frames".to_string(),
                    timestamp: 1680000000,
                    by: api::UserPreview {
                        id: 0,
                        username: "John Doe".to_string(),
                    },
                },
                api::Version {
                    id: 0,
                    save_id: 0,
                    version: 2,
                    label: "Steel Plant".to_string(),
                    timestamp: 1680000000,
                    by: api::UserPreview {
                        id: 1,
                        username: "Steve".to_string(),
                    },
                },
                api::Version {
                    id: 0,
                    save_id: 0,
                    version: 1,
                    label: "Wire Factory".to_string(),
                    timestamp: 1680000000,
                    by: api::UserPreview {
                        id: 0,
                        username: "John Doe".to_string(),
                    },
                },
            ])
        },
        move |v| {
            v.into_iter()
                .map(|v| {
                    crate::versions::Version::new(dioxus::core::SuperInto::super_into(save), v)
                })
                .collect::<Vec<_>>()
        },
    )?;

    use_context_provider(|| save);

    use_context_provider(|| versions);

    rsx! {
        div { class: "pointer-events-none select-none border rounded-xl border-border p-4",
            crate::versions::VersionList {
                versions: versions.store().transpose().expect("Version list to have value"),
                modify: true,
                deploy_version: |_| {},
            }
        }
    }
}
