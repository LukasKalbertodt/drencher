use color::Color;
use std::iter::repeat;
use std::ops;

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


pub struct CellMap<T> {
    size: u8,
    cells: Vec<T>,
}

#[allow(dead_code)]
impl<T> CellMap<T> {
    pub fn new(size: u8, obj: T) -> Self
        where T: Clone
    {
        CellMap {
            size: size,
            cells: repeat(obj).take((size as usize).pow(2)).collect(),
        }
    }

    pub fn default(size: u8) -> Self
        where T: Default
    {
        let cells = repeat(())
            .map(|_| T::default())
            .take((size as usize).pow(2))
            .collect();
        CellMap {
            size: size,
            cells: cells,
        }
    }
}


impl<T> ops::Index<(u8, u8)> for CellMap<T> {
    type Output = T;
    fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
        if x > self.size || y > self.size {
            panic!(
                "x ({}) or y ({}) greater than size ({})",
                x, y, self.size
            );
        }

        &self.cells[
            (y as usize) * (self.size as usize)
                + (x as usize)
        ]
    }
}

impl<T> ops::IndexMut<(u8, u8)> for CellMap<T> {
    fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
        if x > self.size || y > self.size {
            panic!(
                "x ({}) or y ({}) greater than size ({})",
                x, y, self.size
            );
        }

        &mut self.cells[
            (y as usize) * (self.size as usize)
                + (x as usize)
        ]
    }
}
