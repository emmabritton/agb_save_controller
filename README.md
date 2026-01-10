# AGB Save Controller

Uses two slot saving system, like GBA pokemon games, to reduce the risk of save corruption.

## Usage

```rust
// Define your save struct and version

const SAVE_VERSION: u8 = 1;

struct GameSave {
  name: [u8; 8],
  money: u16,
  pos: (u8,u8)
}

impl Default for GameSave {
  fn default() -> Self {
    Self {
      name: [0; 8],
      money: 0,
      pos: (0,0)
    }
  }
}

// Impl SaveEncoding for your struct
impl GbaSave for GameSave {
  //size of save data in bytes
  const SIZE: usize = 12;
  
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
  //then create the save controller (this loads the latest save, or creates a default if none exists)
  let mut save_controller = OwnedSaveController::new(&mut gba.save, SAVE_VERSION).unwrap();
  //read the latest save 
  let save: &GameSave = save_controller.data();
  //update it the active save
  save_controller.data_mut().map(|sav| sav.money = 10);
  //or
  save_controller.set_data(GameSave::default());
  //save it
  save_controller.save().unwrap();
}
```