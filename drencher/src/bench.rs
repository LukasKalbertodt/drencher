use super::{gen_board, get_player};
use solver::Solver;
use board::Board;
use time::Duration;
use term_painter::{ToStyle, Color};
use rayon::prelude::*;
use pbr::ProgressBar;
use std::sync::Mutex;

struct RunOutcome {
    board: Board,
    elapsed_time: Duration,
    moves: usize,
}

pub fn run_benchmark(init_algo: &str, size: u8, player: &str, count: usize)
    -> Result<(), ()>
{
    println!("Benchmarking player '{}' ({} iterations)", player, count);

    let player = try!(get_player(player));
    let mut benchmark = Vec::with_capacity(count);
    let pb = Mutex::new(ProgressBar::new(count as u64));

    (0..count).into_par_iter().weight_max().map(|i| {

        // generate board and get player
        let board = match gen_board(init_algo, size, i as u32) {
            Ok(board) => board,
            Err(_) => return None,
        };

        // let the player try to solve the board
        let mut res = None;
        let mut run_outcome = RunOutcome {
            board: board.clone(),
            elapsed_time: Duration::zero(),
            moves: 0,
        };

        // collect solve outcome
        run_outcome.elapsed_time = Duration::span(|| {
            res = Some(player.solve(board));
        });
        let res = res.unwrap().unwrap_or_else(|e| e);
        run_outcome.moves = res.len();

        pb.lock().unwrap().inc();

        Some(run_outcome)
    }).collect_into(&mut benchmark);

    let benchmark: Vec<_> = benchmark.into_iter().filter_map(|e| e).collect();

    // calc output
    let elapsed_time = benchmark.iter().fold(Duration::zero(), |sum, elem|
        sum + elem.elapsed_time
    );
    let min_run = benchmark.iter().min_by_key(|elem| elem.elapsed_time).unwrap();
    let max_run = benchmark.iter().max_by_key(|elem| elem.elapsed_time).unwrap();
    let num_moves = benchmark.iter().fold(0, |sum, elem|
        sum + elem.moves
    );

    // --- output of the results
    println!(
        "\n{}",
        Color::BrightWhite.bold().paint("----- Benchmark done ------------")
    );
    println!(
        "+++ Time elapsed: {} (avg: {}, min: {}, max: {})",
        Color::BrightYellow.paint(format_duration(elapsed_time)),
        Color::BrightBlue.paint(format_duration(elapsed_time / (count as i32))),
        Color::BrightBlue.paint(format_duration(min_run.elapsed_time)),
        Color::BrightBlue.paint(format_duration(max_run.elapsed_time)),
    );
    println!(
        "+++ Number of moves: {} ({} on average)",
        Color::BrightYellow.paint(num_moves),
        Color::BrightBlue.paint((num_moves as f64) / (count as f64)),
    );

    println!(
        "Initial board that took the most time (solved with {} moves):\n{}",
        Color::BrightBlue.paint(max_run.moves),
        max_run.board,
    );
    println!(
        "Initial board that took the least time (solved with {} moves):\n{}",
        Color::BrightBlue.paint(min_run.moves),
        min_run.board,
    );

    Ok(())
}

fn format_duration(dur: Duration) -> String {
    let min = dur.num_minutes();
    let smaller = dur - Duration::minutes(min);
    let secs = (smaller.num_microseconds().unwrap() as f64) / 1_000_000f64;

    format!("{}m{}s", min, secs)
}
