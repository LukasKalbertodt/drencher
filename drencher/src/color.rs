use std::fmt;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub tag: u8,
}

impl Color {
    pub fn new(tag: u8) -> Color {
        Color {
            tag: tag,
        }
    }

    pub fn as_rgb(&self) -> [f32; 3] {
        let c = match self.tag {
            // 0 => 0xCF000F,
            // 1 => 0x1E824C,
            // 2 => 0xF7CA18,
            // // 3 => 0x446CB3,
            // 4 => 0xB900DB,
            // 5 => 0x52B3D9,
            0 => 0xe74c3c,
            1 => 0x27ae60,
            2 => 0xf1c40f,
            3 => 0x2980b9,
            4 => 0x8e44ad,
            5 => 0x1abc9c,
            _ => 0x000000,
        };

        [
            (((c & 0xFF0000) >> 16) as f32) / 255.0,
            (((c & 0x00FF00) >>  8) as f32) / 255.0,
            (((c & 0x0000FF) >>  0) as f32) / 255.0,
        ]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use term_painter::{ToStyle, Attr};
        use term_painter::Color::*;

        Attr::Plain.bg(match self.tag {
            0 => Red,
            1 => Green,
            2 => Yellow,
            3 => Blue,
            4 => Magenta,
            5 => Cyan,
            _ => White,
        }).paint("  ").fmt(f)
    }
}
