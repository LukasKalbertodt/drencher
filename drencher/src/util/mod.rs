use color::Color;
use bit_set::BitSet;

pub struct ColorSet {
    data: u8,
}

impl ColorSet {
    pub fn new() -> Self {
        ColorSet {
            data: 0,
        }
    }

    pub fn set(&mut self, c: Color) {
        self.data |= 1 << c.tag;
    }

    pub fn is_set(&self, c: Color) -> bool {
        self.data & (1 << c.tag) != 0
    }

    pub fn clear(&mut self) {
        self.data = 0;
    }
}

impl<'a> IntoIterator for &'a ColorSet {
    type Item = Color;
    type IntoIter = ColorSetIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ColorSetIter {
            set: self,
            pos: 0,
        }
    }
}

pub struct ColorSetIter<'a> {
    set: &'a ColorSet,
    pos: u8,
}

impl<'a> Iterator for ColorSetIter<'a> {
    type Item = Color;
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < 6 && !self.set.is_set(Color::new(self.pos)) {
            self.pos += 1;
        }
        if self.pos < 6 {
            self.pos += 1;
            Some(Color::new(self.pos - 1))
        } else {
            None
        }
    }
}

pub fn union(a: &BitSet, b: &BitSet) -> BitSet {
    let mut out = a.clone();
    out.union_with(b);
    out
}
pub fn intersect(a: &BitSet, b: &BitSet) -> BitSet {
    let mut out = a.clone();
    out.intersect_with(b);
    out
}

macro_rules! bset {
    ($($val:expr),*) => {{
        let mut out = BitSet::new();
        $(
            out.insert($val);
        )*
        out
    }}
}
