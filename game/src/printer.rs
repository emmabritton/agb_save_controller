use agb::display::GraphicsFrame;
use agb::display::object::{Object, Sprite};
use agb::fixnum::{Vector2D, vec2};
use alloc::vec::Vec;

pub struct VariWidthType {
    sprites: Vec<&'static Sprite>,
    offsets: Vec<i32>,
}

impl VariWidthType {
    pub fn new(text: &str) -> Self {
        let mut sprites = Vec::with_capacity(text.len());
        let mut offsets = Vec::with_capacity(text.len());

        let mut x = 0;

        for c in text.chars() {
            if c == ' ' {
                x += char_width(c);
                continue;
            }

            let sprite = char_to_sprite(c);

            x -= char_offset(c);

            sprites.push(sprite);
            offsets.push(x);

            x += char_width(c) + 1;
        }

        Self { sprites, offsets }
    }

    pub fn show(&self, pos: Vector2D<i32>, frame: &mut GraphicsFrame) {
        for (&sprite, &offset) in self.sprites.iter().zip(self.offsets.iter()) {
            Object::new(sprite)
                .set_pos(pos + vec2(offset, 0))
                .show(frame);
        }
    }
}

pub fn str_to_sprites(text: &str) -> Vec<&'static Sprite> {
    text.chars().map(char_to_sprite).collect()
}

pub const fn char_to_sprite(c: char) -> &'static Sprite {
    match c {
        '0'..='9' => {
            let index = c as usize - '0' as usize;
            crate::sprites::NUMBERS.sprite(index)
        }
        'A'..='Z' => {
            let index = c as usize - 'A' as usize;
            crate::sprites::UPPER.sprite(index)
        }
        'a'..='z' => {
            let index = c as usize - 'a' as usize;
            crate::sprites::LOWER.sprite(index)
        }
        '!' => crate::sprites::SYM_EXCLAIM.sprite(0),
        '?' => crate::sprites::SYM_QUESTION.sprite(0),
        '.' => crate::sprites::SYM_PERIOD.sprite(0),
        ',' => crate::sprites::SYM_COMMA.sprite(0),
        ':' => crate::sprites::SYM_COLON.sprite(0),
        '#' => crate::sprites::SYM_HASH.sprite(0),
        '(' => crate::sprites::SYM_PAREN_L.sprite(0),
        ')' => crate::sprites::SYM_PAREN_R.sprite(0),
        '%' => crate::sprites::SYM_PERCENT.sprite(0),
        ' ' => crate::sprites::SPACE.sprite(0),
        _ => crate::sprites::UNKNOWN.sprite(0),
    }
}

/// Returns the width in pixels of the given character with starting padding
pub const fn char_width(c: char) -> i32 {
    match c {
        '.' | '!' | ':' | 'i' | ',' | 'l' => 2,
        '(' | ')' | 'j' => 3,
        'T' | 'c' | 'e'..='g' | 'r'..='s' | 'v' | 'x'..='z' => 4,
        '0'..='9'
        | 'A'..='H'
        | 'J'..='L'
        | 'I'
        | 'P'
        | 'R'
        | 'S'
        | 'U'..='V'
        | 'Z'
        | '?'
        | 'b'
        | 'd'
        | 'h'
        | 'k'
        | 'n'..='q'
        | 't'..='u' => 5,
        'X'..='Y' | '#' | '%' | 'a' | 'm' | 'w' => 6,
        'Q' | 'M'..='O' | 'W' => 7,
        ' ' => 4,
        _ => 7,
    }
}

/// Returns the start in pixels of the given character with starting padding
pub const fn char_offset(c: char) -> i32 {
    match c {
        'Q'
        | 'M'
        | 'N'
        | 'O'
        | 'W'
        | 'X'
        | 'Y'
        | 'Z'
        | '.'
        | ','
        | '('
        | ')'
        | '?'
        | '!'
        | ':'
        | '#'
        | '%'
        | 'a'..='z'
        | ' ' => 0,
        '1' | '3' | 'I' => 2,
        _ => 1,
    }
}
