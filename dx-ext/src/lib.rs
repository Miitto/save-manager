#[cfg(feature = "desktop")]
mod file_data;

#[cfg(feature = "desktop")]
pub use file_data::FileData as DesktopFileData;

mod loader_store;
pub use loader_store::*;

pub mod prelude {
    pub use crate::loader_store::*;
}
