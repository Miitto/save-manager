use dioxus::stores::{self as dioxus_stores, Store};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Store, Default)]
pub struct ItrOptions {
    pub coop: bool,
    pub slot: SaveSlots,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveSlots {
    #[default]
    Slot1,
    Slot2,
    Slot3,
    AutoSave1,
    AutoSave2,
    AutoSave3,
}

impl SaveSlots {
    pub fn name(&self) -> &'static str {
        match self {
            SaveSlots::Slot1 => "Save 1.dat.sav",
            SaveSlots::Slot2 => "Save 2.dat.sav",
            SaveSlots::Slot3 => "Save 3.dat.sav",
            SaveSlots::AutoSave1 => "Autosave 1.dat.sav",
            SaveSlots::AutoSave2 => "Autosave 2.dat.sav",
            SaveSlots::AutoSave3 => "Autosave 3.dat.sav",
        }
    }
}
