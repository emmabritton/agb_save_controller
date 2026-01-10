#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SlotId {
    Slot1,
    Slot2,
}

pub struct Layout {
    pub slot_bytes: usize,
    pub slot1_addr: usize,
    pub slot2_addr: usize,
}