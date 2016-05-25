use super::{gen_board, get_player};
use solver::Solver;
use board::Board;
use time::Duration;
use term_painter::{ToStyle, Color};

pub fn run_benchmark(init_algo: &str, size: u8, player: &str, count: usize)
    -> Result<(), ()>
{
    println!("Benchmarking player '{}' ({} iterations)", player, count);

    // time needed for solving the board & number of moves
    let mut elapsed_time = Duration::zero();
    let mut max_time = Duration::zero();
    let mut min_time = Duration::weeks(1000);   // sufficiently large
    let mut max_board = Board::uniform(size);
    let mut min_board = Board::uniform(size);
    let mut max_moves = usize::max_value();   // moves with max time
    let mut min_moves = 0;  // moves with min time
    let mut num_moves = 0;

    for i in 0..count {
        // generate board and get player
        let board = try!(gen_board(init_algo, size, i as u32));
        let board_clone = board.clone();
        let player = try!(get_player(player));

        // let the player try to solve the board
        let mut res = None;
        let iter_time = Duration::span(|| {
            res = Some(player.solve(board));
        });

        let res = res.unwrap().unwrap_or_else(|e| e);

        // update times
        // TODO: remove with += once it's stable
        elapsed_time = elapsed_time + iter_time;

        if iter_time > max_time {
            max_time = iter_time;
            max_board = board_clone.clone();
            max_moves = res.len();
        }

        if iter_time < min_time {
            min_time = iter_time;
            min_board = board_clone;
            min_moves = res.len();
        }

        // we can unwrap here: the above closure is always executed
        num_moves += res.len();
    }


    // --- output of the results
    println!(
        "\n{}",
        Color::BrightWhite.bold().paint("----- Benchmark done ------------")
    );
    println!(
        "+++ Time elapsed: {} (avg: {}, min: {}, max: {})",
        Color::BrightYellow.paint(format_duration(elapsed_time)),
        Color::BrightBlue.paint(format_duration(elapsed_time / (count as i32))),
        Color::BrightBlue.paint(format_duration(min_time)),
        Color::BrightBlue.paint(format_duration(max_time)),
    );
    println!(
        "+++ Number of moves: {} ({} on average)",
        Color::BrightYellow.paint(num_moves),
        Color::BrightBlue.paint((num_moves as f64) / (count as f64)),
    );

    println!(
        "Initial board that took the most time (solved with {} moves):\n{}",
        Color::BrightBlue.paint(max_moves),
        max_board,
    );
    println!(
        "Initial board that took the least time (solved with {} moves):\n{}",
        Color::BrightBlue.paint(min_moves),
        min_board,
    );

    Ok(())
}

fn format_duration(dur: Duration) -> String {
    let min = dur.num_minutes();
    let smaller = dur - Duration::minutes(min);
    let secs = (smaller.num_microseconds().unwrap() as f64) / 1_000_000f64;

    format!("{}m{}s", min, secs)
}
