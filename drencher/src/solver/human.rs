use color::Color;
use board::Board;
use super::{Solver, Solution};
use std::io::{self, Write};

pub struct Human;

impl Solver for Human {
    fn prints_output(&self) -> bool { true }

    fn solve(&self, b: &Board) -> Solution {
        let mut out = Vec::new();
        let mut b = b.clone();

        println!("{}", b);

        while let Some(line) = prompt_color() {
            match line.parse() {
                Ok(n) => {
                    let c = Color::new(n);
                    println!("drenching {}", c);

                    out.push(c);
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

        out
    }
}

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
