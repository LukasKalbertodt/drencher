//! Random solver.
//!
//! A solver that just outputs a random valid move. If the solution wasn't
//! found after `MAX_MOVES` moves, an error is returned.
use super::{Solver, Solution};
use board::Board;
use color::Color;
use rand;
use rand::distributions::{Range, IndependentSample};

/// Type definition for the solver.
pub struct Random;

const MAX_MOVES: usize = 1_000;

impl Solver for Random {
    fn solve(&self, mut b: Board) -> Result<Solution, Solution> {
        // Initialize RNG, range and solution vector
        let mut rng = rand::thread_rng();
        let range = Range::new(0, 6);
        let mut solution = Solution::new();

        // just add more random moves until we actually solved game
        while !b.is_drenched() && solution.len() < MAX_MOVES {
            let color = Color::new(range.ind_sample(&mut rng));
            solution.push(color);
            b.drench(color);
        }

        if b.is_drenched() {
            Ok(solution)
        } else {
            Err(solution)
        }
    }
}
