mod color;
mod board;

use color::Color;
use board::Board;

fn main() {
    // for i in 0..6 {
    //     let col = Color::new(i);
    //     print!("{}", col);
    // }

    let b = Board::random();
    println!("{}", b);

    println!("Hello, world!");
}
