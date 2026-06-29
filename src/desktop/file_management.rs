use dioxus::prelude::*;
use futures::StreamExt;

fn get_version_path(save_name: &str, version: &api::Version) -> std::path::PathBuf {
    let cache_dir = super::dirs::get_version_cache_dir();
    cache_dir
        .join(save_name)
        .join(format!("{}.zip", version.version))
}

pub async fn download_version(
    save_name: &str,
    version: &api::Version,
) -> Result<std::path::PathBuf, ()> {
    let mut stream = match api::download_version(version.save_id, version.id).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to download version: {e}");
            return Err(());
        }
    };

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

    Ok(file_path)
}

use dioxus::stores as dioxus_stores;

use crate::desktop::into_the_radius_2;
#[derive(Debug, Clone, dioxus::stores::Store)]
pub enum DeployOptions {
    IntoTheRadius2(into_the_radius_2::ItrOptions),
    Satisfactory {},
}

impl DeployOptions {
    pub fn matches_game(&self, game: api::Game) -> bool {
        matches!(
            (self, game),
            (
                DeployOptions::IntoTheRadius2 { .. },
                api::Game::IntoTheRadius2
            ) | (DeployOptions::Satisfactory {}, api::Game::Satisfactory)
        )
    }
}

impl From<api::Game> for DeployOptions {
    fn from(game: api::Game) -> Self {
        match game {
            api::Game::IntoTheRadius2 => {
                DeployOptions::IntoTheRadius2(into_the_radius_2::ItrOptions {
                    coop: false,
                    slot: into_the_radius_2::SaveSlots::Slot1,
                })
            }
            api::Game::Satisfactory => DeployOptions::Satisfactory {},
        }
    }
}

pub async fn deploy_version(
    save: &api::Save,
    version: &api::Version,
    deploy_options: DeployOptions,
) -> Result<(), ()> {
    if !deploy_options.matches_game(save.game) {
        error!(
            "Deploy options do not match game: {:?} vs {:?}",
            deploy_options, save.game
        );
        return Err(());
    }

    let zip_path = get_version_path(&save.name, version);
    if !zip_path.exists()
        && let Err(_) = download_version(&save.name, version).await
    {
        error!(
            "Failed to download version {} for deployment",
            version.version
        );
        return Err(());
    }

    let save_dir = super::dirs::get_game_save_dir(save.game);
    if !save_dir.exists() {
        error!("Save directory does not exist: {:?}", save_dir);
        return Err(());
    }

    let file = std::fs::File::open(&zip_path).expect("Failed to open zip file");
    let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip file");

    match deploy_options {
        DeployOptions::IntoTheRadius2(into_the_radius_2::ItrOptions { coop, slot }) => {
            let subfolder = if coop { "Coop" } else { "Single" };
            let slot_path = save_dir.join(subfolder).join(slot.name());

            let mut file = archive.by_index(0).expect("Bad save zip file");

            if slot_path.exists() {
                // Backup the existing save file
                let backup_path = slot_path.with_extension("bak");
                std::fs::copy(&slot_path, &backup_path).expect("Failed to backup save file");
            }

            let mut slot_file =
                std::fs::File::create(&slot_path).expect("Failed to create save file");
            std::io::copy(&mut file, &mut slot_file).expect("Failed to write save file");
        }
        DeployOptions::Satisfactory {} => {
            // No additional setup needed for Satisfactory
        }
    };

    Ok(())
}
