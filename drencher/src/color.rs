extern crate term_painter;

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
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::term_painter::{ToStyle, Attr};
        use self::term_painter::Color::*;

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
