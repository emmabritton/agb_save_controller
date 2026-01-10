# AGB Save Controller

Uses two slot saving system, like GBA pokemon games, to reduce the risk of save corruption.

## Usage

```rust
// Define your save struct and version

const SAVE_VERSION: u8 = 1;

struct GameSave {
  money: u16,
  pos: (u8,u8)
}

// Impl SaveEncoding for your struct
impl GbaSave for GameSave {
  //size of save data in bytes
  const SIZE: usize = 4;
  
  //write the object into the slice however you want
  fn encode_into(&self, out: &mut [u8]) {
    out[0] = pos.0;
    out[1] = pos.1;
    //etc
  }
  
  //create the object from the slice however you want
  fn decode_from(bytes: &[u8]) -> Self {
    let pos = (out[0], out[1]);
    //etc
  }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
  //call one of the init methods such as
  gba.save.init_flash_64k();
  //then create the save controller
  let mut save_controller = OwnedSaveController::new(&mut gba.save, SAVE_VERSION).unwrap();
    if save_controller.load().unwrap() == ManagedLoadResult::NoSave {
    //no save found so initialize data
    save_controller.set_data(MyGameSave {
        money: 100,
        pos: (10, 10)
    });
  }
  agb::println!("You have ${}", save_controller.data().unwrap().money);
  save_controller.data_mut().map(|save| save.money += 100);
  save_controller.save().unwrap();
}
```