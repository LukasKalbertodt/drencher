use color::Color;
use board::Board;

mod random;
mod exact;
pub use self::random::Random;
pub use self::exact::Exact;

pub type Solution = Vec<Color>;

pub trait Solver {
    fn solve(self, b: &Board) -> Solution;
}
