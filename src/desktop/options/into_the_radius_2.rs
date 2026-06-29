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
    pub fn name(&self, coop: bool) -> &'static str {
        match (self, coop) {
            (SaveSlots::Slot1, false) => "Save 1.dat.sav",
            (SaveSlots::Slot2, false) => "Save 2.dat.sav",
            (SaveSlots::Slot3, false) => "Save 3.dat.sav",
            (SaveSlots::AutoSave1, false) => "Autosave 1.dat.sav",
            (SaveSlots::AutoSave2, false) => "Autosave 2.dat.sav",
            (SaveSlots::AutoSave3, false) => "Autosave 3.dat.sav",
            (SaveSlots::Slot1, true) => "Save 1.sav",
            (SaveSlots::Slot2, true) => "Save 2.sav",
            (SaveSlots::Slot3, true) => "Save 3.sav",
            (SaveSlots::AutoSave1, true) => "Autosave 1.sav",
            (SaveSlots::AutoSave2, true) => "Autosave 2.sav",
            (SaveSlots::AutoSave3, true) => "Autosave 3.sav",
        }
    }
}
