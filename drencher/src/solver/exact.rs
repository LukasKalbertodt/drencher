//! Exact Solver
//!
//! This solver always finds an optimal solution (with as few moves as
//! possible). Currently, this solver is very slow and can't really handle
//! instances of size 8 and above.
//!
//! ## Algorithm
//!
//! The idea is simple: the solver traverses the whole game tree in order to
//! find a solution. Traversing is done with breadth-first-search which means
//! that the first solution we find is optimal. The theoretical game tree has
//! 'm^c' nodes, where 'm' is the number of moves of one optimal solution and
//! 'c' is the number of colors. One can easily imagine that trees of bigger
//! instances are impossible to search through. Therefore we have to implement
//! some techniques to avoid searching the *whole* tree.
//!
//! Currently only one optimization is used: when branching deeper into the
//! tree, we ensure that the color of the move corresponding to the current
//! branch, is even a neighbor of the field of cells. If not, the move is
//! useless and the subtree can be ignored. Obviously.
//!
//! ## Representing the tree
//!
//! Right now, the tree is not really represented as a tree. Since we're
//! progressing the tree in BFS, we only need to save the last/current layer.
//! Each node in this layer is saved as a `State` which saves a board state and
//! all moves leading to this state.
//!
//!
//!
use board::Board;
use color::Color;
use std::mem;
use super::{Solver, Solution};

/// Type definition of exact solver. See module documentation for more
/// information.
pub struct Exact;

/// Used to represent one node in the game tree. See module documentation for
/// more information.
#[derive(Clone)]
struct State {
    pub moves: Vec<Color>,
    pub board: Board,
}

impl Solver for Exact {
    fn solve(&self, b: Board) -> Result<Solution, Solution> {
        // root of the tree: no moves yes, initial board
        let initial = State {
            moves: vec![],
            board: b,
        };

        // the `states` vector will hold the current layer of the tree
        let mut states = vec![initial];

        // a few counter to output useful information
        let mut count = 0;

        let mut new_states = Vec::new();

        // BFS until a solution is found
        loop {
            trace!(
                "iteration {} (theory: {})",
                count,
                6usize.pow(count)
            );
            count += 1;

            // in this vector, we will collect all nodes of the next layer
            new_states.clear();
            new_states.reserve(states.len() * 6);

            // for each node in the current layer: add children
            for state in &states {
                // calculate the adjacent colors from the current board
                let adjacent_colors = state.board.adjacent_colors();

                for color in adjacent_colors {
                    // create the new state and push as a child
                    let mut next = state.clone();
                    next.moves.push(color);
                    next.board.drench(color);

                    // if we found a solution, just return
                    if next.board.is_drenched() {
                        return Ok(next.moves.clone());
                    }

                    new_states.push(next);
                }
            }

            // proceed with the next layer
            // states = new_states;
            mem::swap(&mut new_states, &mut states);
        }
    }
}
