use color::Color;
use board::Board;

mod random;
mod exact;
mod human;
pub use self::random::Random;
pub use self::exact::Exact;
pub use self::human::Human;

pub type Solution = Vec<Color>;

pub trait Solver {
    fn solve(&self, b: &Board) -> Solution;

    fn prints_output(&self) -> bool { false }
}
