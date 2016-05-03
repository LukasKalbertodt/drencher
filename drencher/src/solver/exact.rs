use board::Board;
use color::Color;
use std::mem;
use super::{Solver, Solution};

pub struct Exact;

#[derive(Clone)]
struct State {
    pub moves: Vec<Color>,
    pub board: Board,
}

impl Solver for Exact {
    fn solve(&self, b: &Board) -> Solution {
        let initial = State {
            moves: vec![],
            board: b.clone(),
        };
        let mut states = vec![initial];
        let mut count = 0;
        let mut adj_break = 0;

        loop {
            println!(
                "iteration {} (theory: {})",
                count,
                6usize.pow(count)
            );
            count += 1;

            let mut new_states = Vec::new();

            let mut state_count = 0;
            for state in &states {
                state_count += 1;
                if state.board.is_drenched() {
                    return state.moves.clone();
                }

                let adjacent_colors = state.board.adjacent_colors();

                for color in 0..6 {
                    let color = Color::new(color);

                    if !adjacent_colors.contains_key(&color) {
                        adj_break += 1;
                        continue;
                    }

                    let mut next = state.clone();
                    next.moves.push(color);
                    next.board.drench(color);

                    new_states.push(next);
                }
            }
            println!(
                "we broke {}/{} times ",
                adj_break,
                state_count * 6
            );


            mem::swap(&mut new_states, &mut states);
        }
    }
}
