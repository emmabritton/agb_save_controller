#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

#[cfg(test)]
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    loop {}
}

use agb::{include_aseprite, include_background_gfx};

include_aseprite!(
    pub mod sprites,
    "gfx/font.aseprite",
);

include_background_gfx!(
    pub mod bg,
    main => deduplicate "gfx/bg.aseprite",
);

pub mod prelude {
    pub use crate::bg;
    pub use crate::bg_idx;
    pub use crate::sprites;
}

pub mod bg_idx {
    pub const BLACK: usize = 0;
    pub const WHITE: usize = 1;
    pub const BROWN_DARK: usize = 8;
    pub const BROWN_LIGHT: usize = 9;
}

//test with `cargo test --package resources`
#[cfg(test)]
mod test {
    use agb::Gba;

    #[test_case]
    fn test_example_lib(_gba: &mut Gba) {
        assert_eq!(2 + 2, 4);
    }
}