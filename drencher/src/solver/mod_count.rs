use super::{Solver, Solution};
use board::Board;
use color::Color;

pub struct ModCount;

impl Solver for ModCount {
    fn solve(&self, b: Board) -> Result<Solution, Solution> {

        Ok(
            (0..)
            .map(|i| Color::new(i % 6))
            .scan(b, |b, color| {
                let drenched = b.is_drenched();
                if !drenched {
                    b.drench(color);
                }
                Some((drenched, color))
            })
            .take_while(|&(is_drenched, _)| !is_drenched)
            .map(|(_, color)| color)
            .collect()
        )
    }
}
