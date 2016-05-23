use board::Board;
use std::collections::HashMap;
use super::{Solution, Solver};

pub struct Heuristic;

impl Solver for Heuristic {

    fn solve(&self, mut b: Board) -> Result<Solution, Solution> {

        let mut solution = Solution::new();
        while !b.is_drenched() {

            // get border around "player"
            let (_, mut border) = b.field_coords();
            border.sort();
            border.dedup();

            // count color occurrence in border
            let mut color_count = HashMap::new();
            for pos in border {
                *color_count.entry(b[pos]).or_insert(0) += 1;
            }

            // get most occurred color
            let (color, _) = color_count.iter().max_by_key(|&(_, count)| count).unwrap();

            // drench with this color
            b.drench(*color);
            solution.push(*color);
        }

        Ok(solution)
    }
}
