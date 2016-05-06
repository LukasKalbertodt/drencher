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
            println!("+++++ drenching {}", color);

            out.push(color);
            b.drench(color);
            println!("{}", b);

            if b.is_drenched() {
                return Ok(out);
            }
        }

        // apparently we didn't solve the board, but we don't have any more
        // inputs
        Err(out)
    }
}

fn prompt_color() -> Option<Color> {
    // While the user gives us invalid input, we simply loop
    loop {
        // show all possible colors
        print!("Color to drench with next?");
        print!(" ({}->{}", 1, Color::new(0));
        for n in 1..6 {
            let c = Color::new(n);
            print!(", {}->{}", n + 1, c);
        }
        print!(")");

        // flush and return `None` if it wasn't sucessful (very unlikely)
        let res = io::stdout().flush();
        if res.is_err() {
            println!("Wasn't able to flush stdout!");
            return None;
        }

        // Read the users input and try to parse it as `u8`. Meanings of the
        // values of `maybe_num`:
        // - Err(true) -> an IO error occured
        // - Err(false) -> a parsing error occured
        // - Ok(_) -> the parsed `u8`
        let mut buffer = String::new();
        let maybe_num = io::stdin()
            .read_line(&mut buffer)
            .map_err(|_| true)
            .map(|_| buffer.trim())
            .and_then(|line| line.parse().map_err(|_| false));

        match maybe_num {
            Err(true) => return None,
            Err(false) => println!("Not a number!"),
            Ok(0) => println!("Colors are 1-indexed!"),
            Ok(n) => return Some(Color::new(n - 1)),
        }
    }
}
