use color::Color;
use board::Board;

mod random;
mod full;
pub use self::random::Random;
pub use self::full::Full;

pub type Solution = Vec<Color>;

pub trait Solver {
    fn solve(self, b: &Board) -> Solution;
}
