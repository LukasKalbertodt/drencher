use board::Board;
use color::Color;
use super::{Solution, Solver};

pub struct Heuristic;

impl Solver for Heuristic {

    fn solve(&self, b: Board) -> Result<Solution, Solution> {

        

        Err(vec![Color::new(0), Color::new(1), Color::new(2)])
    }
}
