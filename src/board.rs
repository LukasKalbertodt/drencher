use color::Color;
use std::ops;
use std::fmt;

pub struct Board {
    size: u8,
    cells: Vec<Color>,
}


impl Board {
    pub fn random() -> Board {
        Board {
            size: 8,
            cells: vec![Color::new(0); 8*8],
        }
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
