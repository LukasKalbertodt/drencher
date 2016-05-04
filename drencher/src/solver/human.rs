//! Interactive solver via Terminal.
//!
//! This solver will interactively ask the user to choose a color
//! and adds this color to the solution vector. The user is presented
//! with the new board state.

use color::Color;
use board::Board;
use super::{Solver, Solution};
use std::io::{self, Write};

/// Type definition for the solver.
pub struct Human;

impl Solver for Human {
    // implement this to avoid printing all board states again
    fn prints_output(&self) -> bool { true }

    fn solve(&self, mut b: Board) -> Result<Solution, Solution> {
        let mut out = Solution::new();

        // print initial board state
        println!("+++++ Initial board:");
        println!("{}", b);

        // while the user still inputs a color...
        while let Some(color) = prompt_color() {
            println!("drenching {}", color);

            out.push(color);
            b.drench(color);
            println!("{}", b);

            if b.is_drenched() {
                return Ok(out);
            }
        }

        Err(out)
    }
}

fn prompt_color() -> Option<Color> {
    loop {
        print!("Color to drench with next?");
        print!(" ({}->{}", 1, Color::new(0));
        for n in 1..6 {
            let c = Color::new(n);
            print!(", {}->{}", n + 1, c);
        }
        print!(")");
        let _ = io::stdout().flush();

        let mut buffer = String::new();
        let maybe_num = io::stdin()
            .read_line(&mut buffer)
            .ok()
            .map(|_| buffer.trim())
            .and_then(|line| line.parse().ok());

        match maybe_num {
            None => println!("Not a number!"),
            Some(0) => println!("Colors are 1-indexed!"),
            Some(n) => return Some(Color::new(n - 1)),
        }
    }
}
