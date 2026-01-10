#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb_save_controller::{GbaSave, ManagedLoadResult, OwnedSaveController, SaveError};

struct GameSave {
    money: u16,
    pos: (u8,u8)
}

impl GbaSave for GameSave {
    const SIZE: usize = 4;

    fn encode_into(&self, out: &mut [u8]) {
        out[0] = self.pos.0;
        out[1] = self.pos.1;
        let bytes = self.money.to_le_bytes();
        out[2] = bytes[0];
        out[3] = bytes[1];
    }

    fn decode_from(bytes: &[u8]) -> Self {
        let pos = (bytes[0], bytes[1]);
        let money = u16::from_le_bytes([bytes[2], bytes[3]]);
        GameSave {
            pos, money
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    gba.save.init_flash_64k();

    let mut controller: OwnedSaveController<GameSave> = OwnedSaveController::new(&mut gba.save, 1).unwrap();

    match controller.load().unwrap() {
        ManagedLoadResult::NoSave => controller.set_data(GameSave {
            money: 100,
            pos: (4, 5),
        }),
        ManagedLoadResult::Success => {
            assert_eq!(controller.data().unwrap().money, 100);
            assert_eq!(controller.data().unwrap().pos, (4, 5));
        }
        ManagedLoadResult::LastSaveCorrupt => {
            panic!()
        }
    }

    controller.save().unwrap();

    loop {}
}
