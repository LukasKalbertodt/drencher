use super::{Solver, Solution};
use board::Board;
use color::Color;
use rand;
use rand::distributions::{Range, IndependentSample};

pub struct Random;

impl Solver for Random {
    fn solve(self, b: &Board) -> Solution {
        let mut rng = rand::thread_rng();
        let range = Range::new(0, 6);
        let mut solution = Solution::new();

        let mut b = b.clone();
        while !b.is_drenched() {
            let color = Color::new(range.ind_sample(&mut rng));
            solution.push(color);
            b.drench(color);
        }

        solution
    }
}
