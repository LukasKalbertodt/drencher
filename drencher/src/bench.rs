use super::{gen_board, get_player};
use solver::{Solver, Solution};
use board::Board;
use time::Duration;
use term_painter::{ToStyle, Color};
use rayon::prelude::*;
use pbr::ProgressBar;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::f64;

struct RunOutcome {
    board: Board,
    elapsed_time: Duration,
    moves: Solution,
}

pub fn run_benchmark(
    init_algo: &str,
    size: u8,
    player: &str,
    count: usize,
    progress: bool,
    threading: bool,
) -> Result<(), ()> {
    println!("Benchmarking player '{}' ({} iterations)", player, count);

    let player = try!(get_player(player));
    let mut benchmark = Vec::with_capacity(count);
    let pb = Mutex::new(ProgressBar::new(count as u64));
    let no_solution_count = AtomicUsize::new(0);

    let weight = if threading { f64::INFINITY } else { 0f64 };
    (0..count).into_par_iter().weight(weight).map(|i| {

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
            moves: Vec::new(),  // will be overridden later
        };

        // collect solve outcome
        run_outcome.elapsed_time = Duration::span(|| {
            res = Some(player.solve(board));
        });

        // increment progress bar
        if progress {
            pb.lock().unwrap().inc();
        }

        // if the solver didn't find a solution, we will increment the
        // no_solution_count and ignore this run
        run_outcome.moves = match res.unwrap() {
            Ok(res) => res,
            Err(_) => {
                // I don't understand memory orderings...
                no_solution_count.fetch_add(1, Ordering::SeqCst);
                return None;
            }
        };

        Some(run_outcome)
    }).collect_into(&mut benchmark);

    // from now on: no multithreading anymore
    let no_solution_count = no_solution_count.load(Ordering::SeqCst);

    // remove runs where some error occured
    let benchmark: Vec<_> = benchmark.into_iter().filter_map(|e| e).collect();
    if benchmark.len() < count - no_solution_count {
        println!(
            "{} {} runs returned with an error!",
            Color::BrightYellow.paint("!!! Warning:"),
            count - benchmark.len()
        );
    }

    if no_solution_count > 0 {
        println!(
            "{} {} runs did not find a solution!",
            Color::BrightYellow.paint("!!! Warning:"),
            no_solution_count
        );
    }

    // check that all returned results are indeed valid results and only save
    // valid results
    let count_before = benchmark.len();
    let benchmark: Vec<_> = benchmark.into_iter()
        .filter(|run| {
            let mut board = run.board.clone();
            for &c in &run.moves {
                board.drench(c);
            }
            board.is_drenched()
        }).collect();

    if count_before > benchmark.len() {
        println!(
            "{} {} runs returned a wrong result!",
            Color::BrightYellow.paint("!!! Warning:"),
            count_before -  benchmark.len()
        );
    }

    // calc output
    let elapsed_time = benchmark.iter().fold(Duration::zero(), |sum, elem|
        sum + elem.elapsed_time
    );
    let min_run = benchmark.iter().min_by_key(|elem| elem.elapsed_time).unwrap();
    let max_run = benchmark.iter().max_by_key(|elem| elem.elapsed_time).unwrap();
    let num_moves = benchmark.iter().fold(0, |sum, elem|
        sum + elem.moves.len()
    );
    let valid_count = benchmark.len();

    // --- output of the results
    println!(
        "\n{}",
        Color::BrightWhite.bold().paint("----- Benchmark done ------------")
    );
    println!(
        "+++ Time elapsed: {} (avg: {}, min: {}, max: {})",
        Color::BrightYellow.paint(format_duration(elapsed_time)),
        Color::BrightBlue.paint(format_duration(elapsed_time / (valid_count as i32))),
        Color::BrightBlue.paint(format_duration(min_run.elapsed_time)),
        Color::BrightBlue.paint(format_duration(max_run.elapsed_time)),
    );
    println!(
        "+++ Number of moves: {} ({} on average)",
        Color::BrightYellow.paint(num_moves),
        Color::BrightBlue.paint((num_moves as f64) / (valid_count as f64)),
    );

    println!(
        "Initial board that took the most time (solved with {} moves):\n{}",
        Color::BrightBlue.paint(max_run.moves.len()),
        max_run.board,
    );
    println!(
        "Initial board that took the least time (solved with {} moves):\n{}",
        Color::BrightBlue.paint(min_run.moves.len()),
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
