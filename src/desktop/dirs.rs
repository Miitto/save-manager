pub fn get_version_cache_dir() -> std::path::PathBuf {
    use std::env;

    let mut cache_dir = env::current_exe().expect("Failed to get current exe path");
    cache_dir.pop(); // Remove the executable name
    cache_dir.push("downloads");
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).expect("Failed to create version cache directory");
    }
    cache_dir
}

pub fn get_my_games_dir() -> std::path::PathBuf {
    use directories::UserDirs;

    let user_dirs = UserDirs::new().expect("Failed to get user directories");
    user_dirs
        .document_dir()
        .map(|d| d.join("My Games"))
        .expect("Failed to get MyGames directory")
}

pub fn get_local_app_data_dir() -> std::path::PathBuf {
    use directories::BaseDirs;

    let base_dirs = BaseDirs::new().expect("Failed to get user directories");
    base_dirs.data_local_dir().to_path_buf()
}

pub fn get_game_save_dir(game: api::Game) -> std::path::PathBuf {
    use api::Game as E;
    match game {
        E::IntoTheRadius2 => get_my_games_dir().join("IntoTheRadius2").join("Profile 1"),
        E::Satisfactory => unimplemented!("Satisfactory save directory not implemented"),
    }
}
