use std::process::Command;

use dioxus::prelude::*;
use futures::StreamExt;

pub trait ExplorerView {
    fn open_folder(&self);
    fn select_file(&self);
}

impl ExplorerView for std::path::Path {
    fn open_folder(&self) {
        Command::new("explorer")
            .arg(self)
            .spawn()
            .expect("Failed to open folder")
            .wait()
            .expect("Failed to wait for explorer");
    }

    fn select_file(&self) {
        Command::new("explorer")
            .arg("/select,")
            .arg(self)
            .spawn()
            .expect("Failed to open file explorer")
            .wait()
            .expect("Failed to wait for file explorer");
    }
}

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

fn get_version_path(save_name: &str, version: &api::Version) -> std::path::PathBuf {
    let cache_dir = get_version_cache_dir();
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

    Ok(file_path)
}
