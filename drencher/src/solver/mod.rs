use color::Color;
use board::Board;

// define solver-implementations, each in it's own module
mod random;
mod exact;
mod human;
mod heuristic;
mod mod_count;

pub use self::random::Random;
pub use self::exact::Exact;
pub use self::human::Human;
pub use self::heuristic::Heuristic;
pub use self::mod_count::ModCount;

// typedef, thanks to Julian
pub type Solution = Vec<Color>;

/// Something that can solve our game from an initial board.
pub trait Solver: Sync {
    /// Given a board, the solver has to return a list of moves. Those moves
    /// either win the game (`Ok(..)`) or don't (`Err(..)`), in which case the
    /// solver wasn't able to find a winning solution. The `Err` value can
    /// still contain a solution vector.
    fn solve(&self, b: Board) -> Result<Solution, Solution>;

    /// Returns true if the solver already outputs every step of the game. This
    /// is probably only the case for the 'human' solver/player.
    fn prints_output(&self) -> bool { false }
}
