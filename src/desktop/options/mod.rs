pub mod into_the_radius_2;

impl From<&crate::desktop::DeployOptions> for std::path::PathBuf {
    fn from(value: &crate::desktop::DeployOptions) -> Self {
        match value {
            crate::desktop::DeployOptions::IntoTheRadius2(options) => {
                let mut path = crate::desktop::dirs::get_game_save_dir(api::Game::IntoTheRadius2);
                if options.coop {
                    path.push("Coop");
                } else {
                    path.push("Single");
                }
                path.push(options.slot.name(options.coop));
                path
            }
            crate::desktop::DeployOptions::Satisfactory {} => {
                unimplemented!()
            }
        }
    }
}
