extern crate rand;

mod color;
mod board;
mod solver;

use color::Color;
use solver::Solver;
use board::Board;
use std::io::{self, Write};

fn read_line() -> Option<String> {
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Err(_) => None,
        Ok(_) => Some(buffer.trim().to_owned()),
    }
}

fn prompt_color() -> Option<String> {
    print!("Give meh color!!1 (");
    for n in 0..6 {
        let c = Color::new(n);
        print!("{}->{}, ", n, c);
    }
    print!(")");
    let _ = io::stdout().flush();
    read_line()
}

fn main() {
    let b = Board::random(
        std::env::args().nth(1).unwrap().parse().unwrap()
    );
    println!("{}", b);

    // ai_solve(b, solver::Random);
    ai_solve(b, solver::Exact);
    // user_solve();
}

#[allow(dead_code)]
fn ai_solve<S: Solver>(mut b: Board, ai: S) {
    let res = ai.solve(&b);
    for c in res {
        println!("Drenching: {}", c);
        b.drench(c);
        println!("{}", b);
    }
    println!("");
}

#[allow(dead_code)]
fn user_solve(mut b: Board) {
    while let Some(line) = prompt_color() {
        match line.parse() {
            Ok(n) => {
                let c = Color::new(n);
                println!("drenching {}", c);

                b.drench(c);
                println!("{}", b);

                if b.is_drenched() {
                    break;
                }
            },
            Err(_) => {
                println!("fuuuuuu");
            }
        }
    }
}
