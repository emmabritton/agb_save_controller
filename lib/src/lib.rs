//! It is the users responsibility to manage the expected version
//! If the save contents or size changes without the version changing then loading may result in undefined behavior
//! There is no save migration support
//!
//! For a simple Pokemon style save use OwnedSaveController
//! For a multiple slot system, like Final Fantasy 7, use OwnedFileController
//!
//! # Usage
//!
//! ```rust
//! use game::OwnedSaveController;
//!
//! const SAVE_VERSION: u8 = 1;
//!
//! struct MyGameSave {
//!     money: u16,
//!     pos: (u8, u8)
//! }
//!
//! impl GbaSave for MyGameSave {
//!     const SIZE: usize = 4;
//!
//!     fn encode_into(&self, out: &mut [u8]) {
//!         out[0] = self.pos.0;
//!         out[1] = self.pos.1;
//!         let bytes = self.money.to_le_bytes();
//!         out[2] = bytes[0];
//!         out[3] = bytes[1];
//!     }
//!
//!     fn decode_from(bytes: &[u8]) -> Self {
//!         let pos = (bytes[0], bytes[1]);
//!         let money = u16::from_le_bytes([bytes[2], bytes[3]]);
//!         Self {
//!             pos,
//!             money
//!         }
//!     }
//! }
//!
//! #[agb::entry]
//! fn main(mut gba: agb::Gba) -> ! {
//!#   use game::{OwnedSaveController, ManagedLoadResult};
//!   gba.save.init_flash_64k();
//!   let mut save_controller = OwnedSaveController::new(&mut gba.save, SAVE_VERSION).unwrap();
//!   if save_controller.load().unwrap() == ManagedLoadResult::NoSave {
//!     //no save found so initialize data
//!     save_controller.set_data(MyGameSave {
//!         money: 100,
//!         pos: (10, 10)
//!     });
//!   }
//!   agb::println!("You have ${}", save_controller.data().unwrap().money);
//!   save_controller.data_mut().map(|save| save.money += 100);
//!   save_controller.save().unwrap();
//! }
//! ```

#![no_std]
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_harness")]

mod header;
mod internal;

extern crate alloc;

use crate::SaveError::HardwareError;
use crate::SlotId::Slot1;
use SlotId::Slot2;
use agb::{println, save};
use agb::save::SaveManager;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use thiserror::Error;
use crate::header::{SlotHeader, HEADER_LEN};
use crate::internal::{Layout, SlotId};

#[derive(Debug, Clone, Error)]
pub enum SaveError {
    /// Some hardware error
    #[error("Hardware error: {0:?}")]
    HardwareError(save::Error),
    /// Both saves were corrupt
    #[error("Both saves were corrupt")]
    CorruptedSaves,
    /// Save was called but no data was loaded or set
    #[error("save() was called without calling load() or set_data()")]
    NoSaveData,
    /// Save data is too big for storage
    #[error("Save data requires {}b but storage is {}b", required, actual)]
    SaveTooBig { required: usize, actual: usize },
    #[error("File index {} is higher than available {}", index, max)]
    FileIndexOutsideRange { index: u8, max: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadResult<T: GbaSave> {
    /// No saves found
    NoSave,
    /// Latest save
    Success(T),
    /// Previous save as latest was corrupt
    LastSaveCorrupt(T),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedLoadResult {
    /// No saves found
    NoSave,
    /// Latest save loaded
    Success,
    /// Previous save loaded as latest was corrupt
    LastSaveCorrupt,
}

/// Your save struct must impl
/// You can encode/decode however you want
/// The byte slices in encode and decode will always be the correct size
pub trait GbaSave: Sized {
    /// Save of your save data in bytes
    const SIZE: usize;
    /// Write your save data into `out`
    fn encode_into(&self, out: &mut [u8]);
    /// Construct
    fn decode_from(bytes: &[u8]) -> Self;
}

/// Stores slot and generation info
/// Creates and validates checksums
pub struct SaveSlotController<T: GbaSave> {
    expected_version: u8,
    next_slot: SlotId,
    next_gen: u32,
    layout: Layout,
    _save_format: PhantomData<T>
}

impl<T: GbaSave> SaveSlotController<T> {
    pub fn new(save_manager: &mut SaveManager,
               expected_version: u8) -> Result<Self, SaveError> {
        SaveSlotController::new_with_offset(save_manager, expected_version, 0)
    }

    /// Write saves starting at offset (rounded to next sector size)
    /// If you want to store extra data in additional to these saves, pass it's size in min_offset and save at addr 0
    pub fn new_with_min_offset(save_manager: &mut SaveManager,
                               expected_version: u8,
                               min_offset: usize) -> Result<Self, SaveError> {
        let sector_size =  save_manager.access().map_err(HardwareError)?.sector_size();
        let offset = if sector_size == 0 {
            min_offset
        } else {
            round_up(min_offset, sector_size)
        };
        SaveSlotController::new_with_offset(save_manager, expected_version, offset)
    }

    /// # Params
    /// * `offset`: offset in bytes to write save data, typically `0`
    fn new_with_offset(
        save_manager: &mut SaveManager,
        expected_version: u8,
        offset: usize,
    ) -> Result<Self, SaveError> {
        let (len, sector_size) = {
            let accessor = save_manager.access().map_err(HardwareError)?;
            (accessor.len(), accessor.sector_size())
        };

        println!("data len: {}", T::SIZE);
        println!("sector_size: {sector_size}, len: {len}");

        debug_assert!(sector_size == 0 || offset % sector_size == 0);

        let slot_bytes = if sector_size == 0 {
            T::SIZE + HEADER_LEN
        } else {
            round_up(T::SIZE + HEADER_LEN, sector_size)
        };

        println!("slot bytes: {slot_bytes}");

        let required = offset + (slot_bytes * 2);
        if len < required {
            return Err(SaveError::SaveTooBig {
                required,
                actual: len,
            });
        }

        Ok(Self {
            expected_version,
            next_slot: SlotId::Slot1,
            next_gen: 0,
            layout: Layout {
                slot_bytes,
                slot1_addr: offset,
                slot2_addr: offset + slot_bytes,
            },
            _save_format: Default::default(),
        })
    }
}

impl<T: GbaSave> SaveSlotController<T> {
    /// You must call [SaveSlotController::load] first otherwise this will overwrite anything in storage
    /// Write `data` into storage, automatically handling checksums, slots and generations
    pub fn save(&mut self, save_manager: &mut SaveManager, data: &T) -> Result<(), SaveError> {
        let mut buf = vec![0xFF; self.layout.slot_bytes];
        let payload_range = HEADER_LEN..(HEADER_LEN + T::SIZE);
        data.encode_into(&mut buf[payload_range.clone()]);
        let crc = calc_crc32(&buf[payload_range]);
        let header = SlotHeader::new(self.expected_version, self.next_gen, T::SIZE as u32, crc);

        header.encode_into(&mut buf);

        let addr = match self.next_slot {
            Slot1 => self.layout.slot1_addr,
            Slot2 => self.layout.slot2_addr,
        };

        println!("writing {} bytes at {addr}", buf.len());
        write_atomic(save_manager, addr, &buf)?;

        self.next_gen = self.next_gen.wrapping_add(1);
        self.next_slot = match self.next_slot {
            Slot1 => Slot2,
            Slot2 => Slot1,
        };

        Ok(())
    }

    /// Load save data from storage
    /// Sets slot, gen, etc info
    pub fn load(&mut self, save_manager: &mut SaveManager) -> Result<LoadResult<T>, SaveError> {
        let h1 = read_header::<T>(save_manager, self.layout.slot1_addr, self.expected_version)?;
        let h2 = read_header::<T>(save_manager, self.layout.slot2_addr, self.expected_version)?;

        let winner_slot = match (&h1, &h2) {
            (None, None) => return Ok(LoadResult::NoSave),
            (Some(_), None) => Slot1,
            (None, Some(_)) => Slot2,
            (Some(a), Some(b)) => {
                if b.generation() > a.generation() {
                    Slot2
                } else {
                    Slot1
                }
            }
        };

        let (winner_header, winner_addr, prev_header, prev_addr) = match winner_slot {
            Slot1 => (
                h1.unwrap(),
                self.layout.slot1_addr,
                h2,
                self.layout.slot2_addr,
            ),
            Slot2 => (
                h2.unwrap(),
                self.layout.slot2_addr,
                h1,
                self.layout.slot1_addr,
            ),
        };

        fn try_read_payload<T: GbaSave>(
            save_manager: &mut SaveManager,
            base_addr: usize,
            header: &SlotHeader,
        ) -> Result<Option<T>, SaveError> {
            let mut payload = vec![0u8; T::SIZE];
            let payload_off = base_addr + HEADER_LEN;
            let mut accessor = save_manager.access().map_err(HardwareError)?;
            accessor
                .read(payload_off, &mut payload)
                .map_err(HardwareError)?;
            let crc = calc_crc32(&payload);
            if crc != header.crc32() {
                return Ok(None);
            }
            Ok(Some(T::decode_from(&payload)))
        }

        if let Some(data) = try_read_payload::<T>(save_manager, winner_addr, &winner_header)? {
            self.next_slot = match winner_slot {
                Slot1 => Slot2,
                Slot2 => Slot1,
            };
            self.next_gen = winner_header.generation().wrapping_add(1);
            return Ok(LoadResult::Success(data));
        }

        if let Some(prev_header) = prev_header {
            if let Some(data) = try_read_payload::<T>(save_manager, prev_addr, &prev_header)? {
                let recovered_slot = match winner_slot {
                    Slot1 => Slot2,
                    Slot2 => Slot1,
                };
                self.next_slot = match recovered_slot {
                    Slot1 => Slot2,
                    Slot2 => Slot1,
                };
                self.next_gen = prev_header.generation().wrapping_add(1);
                return Ok(LoadResult::LastSaveCorrupt(data));
            }
        }

        Err(SaveError::CorruptedSaves)
    }

    /// Delete save and reset slot, gen, etc
    pub fn delete(&mut self, save_manager: &mut SaveManager) -> Result<(), SaveError> {
        self.next_gen = 0;
        self.next_slot = Slot1;

        let start = self.layout.slot1_addr;
        let end = self.layout.slot2_addr + self.layout.slot_bytes;

        let mut accessor = save_manager.access().map_err(HardwareError)?;
        let mut writer = accessor.prepare_write(start..end).map_err(HardwareError)?;

        writer.write(start, &[0xFF]).map_err(HardwareError)?;

        let slot2_off = self.layout.slot2_addr + start;
        writer.write(slot2_off, &[0xFF]).map_err(HardwareError)?;
        Ok(())
    }
}

/// Convenience for [SaveSlotController] that includes the GbaSave
pub struct SaveController<T: GbaSave> {
    data: Option<T>,
    controller: SaveSlotController<T>,
}

impl<T: GbaSave> SaveController<T> {
    pub fn new(save_manager: &mut SaveManager, expected_version: u8) -> Result<Self, SaveError> {
        let controller = SaveSlotController::new(save_manager, expected_version)?;
        Ok(Self {
            data: None,
            controller,
        })
    }
}

impl<T: GbaSave> SaveController<T> {
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn data_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }

    pub fn set_data(&mut self, data: T) {
        self.data = Some(data);
    }

    pub fn save(&mut self, save_manager: &mut SaveManager) -> Result<(), SaveError> {
        if let Some(data) = &self.data {
            self.controller.save(save_manager, data)
        } else {
            Err(SaveError::NoSaveData)
        }
    }

    /// Load save data from storage
    ///
    /// Returns true if a save was found
    pub fn load(&mut self, save_manager: &mut SaveManager) -> Result<ManagedLoadResult, SaveError> {
        self.controller
            .load(save_manager)
            .map(|result| match result {
                LoadResult::NoSave => ManagedLoadResult::NoSave,
                LoadResult::Success(data) => {
                    self.data = Some(data);
                    ManagedLoadResult::Success
                }
                LoadResult::LastSaveCorrupt(data) => {
                    self.data = Some(data);
                    ManagedLoadResult::LastSaveCorrupt
                }
            })
    }

    pub fn delete(&mut self, save_manager: &mut SaveManager) -> Result<(), SaveError> {
        self.data = None;
        self.controller.delete(save_manager)
    }
}

/// Convenience for [SaveController] that includes the [SaveManager]
///
/// Takes ownership of [SaveManager]
pub struct OwnedSaveController<'gba, T: GbaSave> {
    manager: &'gba mut SaveManager,
    controller: SaveController<T>,
}

impl<'gba, T: GbaSave> OwnedSaveController<'gba, T> {
    /// Takes ownership of [SaveManager]
    pub fn new(manager: &'gba mut SaveManager, expected_version: u8) -> Result<Self, SaveError> {
        let controller = SaveController::new(manager, expected_version)?;
        Ok(Self {
            manager,
            controller,
        })
    }
}

impl<'gba, T: GbaSave> OwnedSaveController<'gba, T> {
    pub fn data(&self) -> Option<&T> {
        self.controller.data()
    }

    pub fn data_mut(&mut self) -> Option<&mut T> {
        self.controller.data_mut()
    }

    pub fn set_data(&mut self, data: T) {
        self.controller.set_data(data);
    }

    pub fn save(&mut self) -> Result<(), SaveError> {
        self.controller.save(self.manager)
    }

    /// Load save data from storage
    ///
    /// Returns true if a save was found
    pub fn load(&mut self) -> Result<ManagedLoadResult, SaveError> {
        self.controller.load(self.manager)
    }

    pub fn delete(&mut self) -> Result<(), SaveError> {
        self.controller.delete(self.manager)
    }
}

/// Multislot save controller
///
/// If you have a lot of save slots you should put this in a [Box]
///
/// It can't store the save data (like [SaveController] does) due to memory limitations
pub struct FileController<T: GbaSave> {
    file_count: u8,
    save_controllers: Vec<SaveSlotController<T>>,
}

impl<T: GbaSave> FileController<T> {
    pub fn new(
        save_manager: &mut SaveManager,
        file_count: u8,
        expected_version: u8,
    ) -> Result<Self, SaveError> {
        let (len, sector_size) = {
            let accessor = save_manager.access().map_err(HardwareError)?;
            (accessor.len(), accessor.sector_size())
        };

        let slot_bytes = if sector_size == 0 {
            T::SIZE + HEADER_LEN
        } else {
            round_up(T::SIZE + HEADER_LEN, sector_size)
        };

        let save_size = slot_bytes * 2;
        let required = save_size * file_count as usize;
        if len < required {
            return Err(SaveError::SaveTooBig {
                required,
                actual: len,
            });
        }
        let save_controllers = (0..file_count as usize)
            .map(|i| {
                SaveSlotController::new_with_offset(save_manager, expected_version, save_size * i)
                    .expect("storage size already checked")
            })
            .collect();

        Ok(Self {
            file_count,
            save_controllers,
        })
    }
}

impl<T: GbaSave> FileController<T> {
    pub fn save(
        &mut self,
        save_manager: &mut SaveManager,
        file_idx: u8,
        data: &T,
    ) -> Result<(), SaveError> {
        if file_idx >= self.file_count {
            return Err(SaveError::FileIndexOutsideRange {
                index: file_idx,
                max: self.file_count,
            });
        }

        self.save_controllers[file_idx as usize].save(save_manager, data)
    }

    pub fn load(
        &mut self,
        save_manager: &mut SaveManager,
        file_idx: u8,
    ) -> Result<LoadResult<T>, SaveError> {
        if file_idx >= self.file_count {
            return Err(SaveError::FileIndexOutsideRange {
                index: file_idx,
                max: self.file_count,
            });
        }

        self.save_controllers[file_idx as usize].load(save_manager)
    }
}

/// Convenience for [FileController] that includes the [SaveManager]
///
/// Takes ownership of [SaveManager]
pub struct OwnedFileController<'gba, T: GbaSave> {
    file_controller: FileController<T>,
    save_manager: &'gba mut SaveManager,
}

impl<'gba, T: GbaSave> OwnedFileController<'gba, T> {
    /// Takes ownership of [SaveManager]
    pub fn new(
        save_manager: &'gba mut SaveManager,
        file_count: u8,
        expected_version: u8,
    ) -> Result<Self, SaveError> {
        let file_controller = FileController::new(save_manager, file_count, expected_version)?;
        Ok(Self {
            file_controller,
            save_manager,
        })
    }
}

impl<'gba, T: GbaSave> OwnedFileController<'gba, T> {
    pub fn save(&mut self, file_idx: u8, data: &T) -> Result<(), SaveError> {
        self.file_controller.save(self.save_manager, file_idx, data)
    }

    pub fn load(&mut self, file_idx: u8) -> Result<LoadResult<T>, SaveError> {
        self.file_controller.load(self.save_manager, file_idx)
    }
}

fn round_up(n: usize, align: usize) -> usize {
    (n + align - 1) / align * align
}

fn write_atomic(
    save_manager: &mut SaveManager,
    addr: usize,
    bytes: &[u8],
) -> Result<(), SaveError> {
    let mut accessor = save_manager.access().map_err(HardwareError)?;
    let mut writer = accessor
        .prepare_write(addr..addr + bytes.len())
        .map_err(HardwareError)?;
    writer.write(addr, bytes).map_err(HardwareError)?;
    Ok(())
}

fn calc_crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn read_header<T: GbaSave>(
    save_manager: &mut SaveManager,
    addr: usize,
    expected_version: u8,
) -> Result<Option<SlotHeader>, SaveError> {
    let mut buf = [0u8; HEADER_LEN];
    let mut accessor = save_manager.access().map_err(HardwareError)?;
    accessor.read(addr, &mut buf).map_err(HardwareError)?;
    let h = SlotHeader::decode_from(&buf);
    if h.is_valid::<T>(expected_version) {
        Ok(Some(h))
    } else {
        Ok(None)
    }
}
