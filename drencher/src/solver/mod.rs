use color::Color;
use board::Board;

// define solver-implementations, each in it's own module
mod random;
mod exact;
mod human;
pub use self::random::Random;
pub use self::exact::Exact;
pub use self::human::Human;

// typedef, thanks to Julian
pub type Solution = Vec<Color>;

/// Something that can solve our game from an initial board.
pub trait Solver {
    /// Given a board, the solver has to return a list of moves that will
    /// win the game.
    fn solve(&self, b: Board) -> Result<Solution, Solution>;

    /// Returns true if the solver already outputs every step of the game. This
    /// is probably only the case for the 'human' solver/player.
    fn prints_output(&self) -> bool { false }
}
