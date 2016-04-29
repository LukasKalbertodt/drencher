extern crate rand;

mod color;
mod board;

use color::Color;
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
    // for i in 0..6 {
    //     let col = Color::new(i);
    //     print!("{}", col);
    // }

    let mut b = Board::random(4);
    println!("{}", b);

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
