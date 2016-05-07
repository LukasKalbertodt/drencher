extern crate docopt;
extern crate rand;
extern crate rustc_serialize;
extern crate term_painter;
extern crate time;

mod color;
mod board;
mod solver;

use solver::Solver;
use board::Board;
use docopt::Docopt;
use time::Duration;
use term_painter::{ToStyle, Color};


// USAGE-string used by docopt
const USAGE: &'static str = "
Drencher: implementation of the 'drench' game with AI- and human-players.

Usage:
  drencher [options] [<player>]
  drencher (-h | --help)
  drencher --version

Arguments:
  player                The player/solver for the game.

Options:
  -h --help             Show this screen.
  --version             Show version.
  --size=<size>         Side length of the board [default: 14].
  --board=<initial>     Initial configuration of the board [default: random].
  --bench=<count>       In the benchmarking mode the specified player <count>
                        games are played and timing is measured. It's advised
                        to use a deterministic initial board algorithm, like
                        'isaac-0', or to use a fairly high repetition count.
                        There is also no output of the board or the solution
                        in this mode.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_player: Option<String>,
    flag_version: bool,
    flag_size: u8, // TODO: nice error message when input is too big
    flag_board: String,
    flag_bench: Option<usize>,
}


fn main() {
    // read and parse CLI-args, exit if any error occured
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // if the version flag was set, we just print the version and exit
    if args.flag_version {
        println!("drencher {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let player = args.arg_player.unwrap_or("human".into());

    let res = if let Some(count) = args.flag_bench {
        if player == "human" {
            println!(
                "{}: you are benchmarking with a human player...",
                Color::BrightYellow.paint("Warning"),
            );
        }

        run_benchmark(
            &args.flag_board,
            args.flag_size,
            &player,
            count,
        )
    } else {
        play_standard_mode(
            &args.flag_board,
            args.flag_size,
            &player,
        )
    };

    if res.is_err() {
        std::process::exit(1);
    }
}

fn play_standard_mode(init_algo: &str, size: u8, player: &str)
    -> Result<(), ()>
{
    println!("~~~~~~ Playing a standard game ~~~~~~");

    // generate board and get player
    let board = try!(gen_board(init_algo, size));
    let player = try!(get_player(player));

    // let the player try to solve the board
    let res = player.solve(board.clone());

    // depending on whether the player already prints output
    if !player.prints_output() {
        // go through all the moves and print the board at every state
        let mut board = board;
        for &c in res.as_ref().unwrap_or_else(|e| e) {
            println!("Drenching: {}", c);
            board.drench(c);
            println!("{}", board);
        }
        println!("");
    }

    match res {
        Ok(_) => println!("Game was solved! :-)"),
        Err(_) => println!("Game was NOT solved! :-("),
    }

    Ok(())
}

fn run_benchmark(init_algo: &str, size: u8, player: &str, count: usize)
    -> Result<(), ()>
{
    println!("Benchmarking player '{}' ({} iterations)", player, count);

    // time needed for solving the board & number of moves
    let mut elapsed_time = Duration::zero();
    let mut max_time = Duration::zero();
    let mut min_time = Duration::weeks(1000);   // sufficiently large
    let mut max_board = Board::random(size);
    let mut min_board = Board::random(size);
    let mut max_moves = usize::max_value();   // moves with max time
    let mut min_moves = 0;  // moves with min time
    let mut num_moves = 0;

    for _ in 0..count {
        // generate board and get player
        let board = try!(gen_board(init_algo, size));
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

fn gen_board(init_algo: &str, size: u8) -> Result<Board, ()> {
    match init_algo {
        "random" => Ok(Board::random(size)),
        other => {
            println!("Intial board algorithm '{}' doesn't exist!", other);
            Err(())
        }
    }
}

fn get_player(name: &str) -> Result<Box<Solver>, ()> {
    match name {
        "human" => Ok(Box::new(solver::Human)),
        "exact" => Ok(Box::new(solver::Exact)),
        "random" => Ok(Box::new(solver::Random)),
        other => {
            println!("Player '{}' does not exist!", other);
            Err(())
        }
    }
}

fn format_duration(dur: Duration) -> String {
    let min = dur.num_minutes();
    let smaller = dur - Duration::minutes(min);
    let secs = (smaller.num_microseconds().unwrap() as f64) / 1_000_000f64;

    format!("{}m{}s", min, secs)
}
