#![allow(dead_code)]

use color::Color;
use std::ops;
use std::fmt;
use std::collections::HashMap;
use rand;
use rand::distributions::{Range, IndependentSample};
use rand::{IsaacRng, SeedableRng, Rng};
use std::iter::repeat;


#[derive(Clone)]
pub struct Board {
    size: u8,
    cells: Vec<Color>,
}


impl Board {
    pub fn uniform(size: u8) -> Board {
        Board {
            size: size,
            cells: repeat(Color::new(0)).take((size as usize).pow(2)).collect()
        }

    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn random(size: u8) -> Board {
        let mut rng = rand::thread_rng();
        Self::with_rng(size, &mut rng)
    }

    pub fn deterministic_random(size: u8, id: u32) -> Board {
        let mut rng = IsaacRng::from_seed(&[id, id + 42, id + 27, id + 1337]);
        Self::with_rng(size, &mut rng)
    }

    fn with_rng(size: u8, mut rng: &mut Rng) -> Board {
        let mut v = Vec::with_capacity(
            (size as usize).pow(2)
        );
        let range = Range::new(0, 6);

        for _ in 0..(size as u16) * (size as u16) {
            let n = range.ind_sample(&mut rng);
            v.push(Color::new(n));
        }

        Board {
            size: size,
            cells: v,
        }
    }

    pub fn drench(&mut self, new: Color) {
        if new != self[(0, 0)] {
            for (x, y) in self.field_coords().0 {
                self[(x, y)] = new;
            }
        }
    }

    pub fn field_coords(&self) -> (Vec<(u8, u8)>, Vec<(u8, u8)>) {
        let mut stack = Vec::new();
        let start_color = self[(0, 0)];
        stack.push((0, 0));

        let mut visited = Vec::new();
        let mut border = Vec::new();

        while let Some((x, y)) = stack.pop() {
            let already_visited = visited
                .iter()
                .find(|&&pos| pos == (x, y))
                .is_none();
            if already_visited {
                if self[(x, y)] == start_color {
                    visited.push((x, y));

                    if x > 0 {
                        stack.push((x - 1, y));
                    }
                    if y > 0 {
                        stack.push((x, y - 1));
                    }
                    if x < self.size - 1 {
                        stack.push((x + 1, y));
                    }
                    if y < self.size - 1 {
                        stack.push((x, y + 1));
                    }
                } else {
                    border.push((x, y));
                }
            }
        }

        (visited, border)
    }

    pub fn is_drenched(&self) -> bool {
        self.cells.iter().all(|&c| c == self[(0, 0)])
    }

    pub fn adjacent_colors(&self) -> Vec<Color> {
        let mut colors = [0; 6];

        for (x, y) in self.field_coords().1 {
            let color = self[(x, y)];
            colors[color.tag as usize] += 1;
        }

        colors.iter().enumerate().filter_map(|(i, &count)| {
            if count > 0 {
                Some(Color::new(i as u8))
            } else {
                None
            }
        }).collect()
    }
}

impl ops::Index<(u8, u8)> for Board {
    type Output = Color;
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

impl ops::IndexMut<(u8, u8)> for Board {
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

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.size {
            for x in 0..self.size {
                try!(self[(x, y)].fmt(f));
            }
            try!("\n".fmt(f));
        }
        Ok(())
    }
}
