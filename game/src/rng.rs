#[inline]
pub fn next_u8() -> u8 {
    agb::rng::next_i32() as u8
}

#[inline]
pub fn next_u8_in(min: u8, max: u8) -> u8 {
    let num = next_u8();
    (num % max) + min
}

#[inline]
pub fn next_u16() -> u16 {
    agb::rng::next_i32() as u16
}

#[inline]
pub fn next_u16_in(min: u16, max: u16) -> u16 {
    let num = next_u16();
    (num % max) + min
}

#[inline]
pub fn next_i32() -> i32 {
    agb::rng::next_i32()
}

#[inline]
pub fn next_i32_in(min: i32, max: i32) -> i32 {
    let num = next_i32();
    (num % max) + min
}

pub fn rand<T>(list: &[T]) -> usize {
    let idx = next_u16_in(0, list.len() as u16);
    idx as usize
}
