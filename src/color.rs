use std::fmt;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub tag: u8,
}

impl Color {
    pub fn new(tag: u8) -> Color {
        Color {
            tag: tag,
        }
    }

    pub fn symbol(&self) -> &str {
        match self.tag {
            0 => "\u{1b}[41m  \u{1b}(B\u{1b}[m",  // red
            1 => "\u{1b}[42m  \u{1b}(B\u{1b}[m",  // green
            2 => "\u{1b}[43m  \u{1b}(B\u{1b}[m",  // yellow
            3 => "\u{1b}[44m  \u{1b}(B\u{1b}[m",  // blue
            4 => "\u{1b}[45m  \u{1b}(B\u{1b}[m",  // magenta
            5 => "\u{1b}[46m  \u{1b}(B\u{1b}[m",  // cyan
            _ => "X",
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.symbol().fmt(f)
    }
}
