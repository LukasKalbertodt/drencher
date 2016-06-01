use super::{Solver, Solution};
use board::Board;
use color::Color;

pub struct ModCount;

impl Solver for ModCount {
    fn solve(&self, mut b: Board) -> Result<Solution, Solution> {
        let mut solution = Solution::new();
        let mut index = 0u8;

        while !b.is_drenched() {
            let color = Color::new(index);
            solution.push(color);
            b.drench(color);

            index += 1;
            index %= 6;
        }

        Ok(solution)
    }
}
