#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveType {
    SinglePlayer(SaveSlots),
    Coop(SaveSlots),
}

impl SaveType {
    pub fn join_path(&self, path: &std::path::Path) -> std::path::PathBuf {
        match self {
            SaveType::SinglePlayer(slot) => path.join("Single").join(slot.name()),
            SaveType::Coop(slot) => path.join("Coop").join(slot.name()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveSlots {
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
