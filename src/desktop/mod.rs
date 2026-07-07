use std::process::Command;

mod file_management;
mod options;
mod save_ui;

pub use file_management::*;
pub use options::*;
pub use save_ui::*;

pub mod dirs;

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
