use color::Color;
use std::ops;
use std::fmt;
use rand;
use rand::distributions::{Range, IndependentSample};

pub struct Board {
    size: u8,
    cells: Vec<Color>,
}


impl Board {
    pub fn random(size: u8) -> Board {
        let mut v = Vec::with_capacity(
            (size as usize) * (size as usize)
        );
        let mut rng = rand::thread_rng();
        let range = Range::new(0, 6);

        for _ in 0..size * size {
            let n = range.ind_sample(&mut rng);
            v.push(Color::new(n));
        }

        Board {
            size: size,
            cells: v,
        }
    }

    pub fn drench(&mut self, new: Color) {
        let mut stack = Vec::new();
        let start_color = self[(0, 0)];
        stack.push((0, 0));

        if new == start_color {
            return;
        }

        while let Some((x, y)) = stack.pop() {
            if self[(x, y)] == start_color {
                self[(x, y)] = new;

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
            }
        }
    }

    pub fn is_drenched(&self) -> bool {
        self.cells.iter().all(|&c| c == self[(0, 0)])
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
