extern crate docopt;
extern crate rand;
extern crate rustc_serialize;
extern crate term_painter;
extern crate time;
extern crate rayon;
extern crate pbr;
#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use] extern crate glium;
extern crate smallvec;
extern crate bit_set;

mod color;
mod board;
mod solver;
mod bench;
mod util;

use solver::Solver;
use board::Board;
use docopt::Docopt;
use term_painter::{ToStyle, Color};
use bench::run_benchmark;


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
                        'deter0', or to use a fairly high repetition count.
                        There is also no output of the board or the solution
                        in this mode.
  --no-progress         Hide progress bar.
  --no-threads          Disable threading
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_player: Option<String>,
    flag_version: bool,
    flag_size: u8, // TODO: nice error message when input is too big
    flag_board: String,
    flag_bench: Option<usize>,
    flag_no_progress: bool,
    flag_no_threads: bool,
}


fn main() {
    // register logger
    env_logger::init().unwrap();

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
            !args.flag_no_progress,
            !args.flag_no_threads,
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
    let board = try!(gen_board(init_algo, size, 0));
    let player = try!(get_player(player));

    // let the player try to solve the board
    let res = player.solve(board.clone());

    // depending on whether the player already prints output
    if !player.prints_output() {
        // go through all the moves and print the board at every state
        let mut board = board;
        println!("Start board:\n{}", board);
        for &c in res.as_ref().unwrap_or_else(|e| e) {
            println!("Drenching: {}", c);
            board.drench(c);
            println!("{}", board);
        }
        println!("");
    }

    match res {
        Ok(res) => println!("Game was solved (in {} steps)! :-)", res.len()),
        Err(_) => println!("Game was NOT solved! :-("),
    }

    Ok(())
}

fn gen_board(init_algo: &str, size: u8, id: u32) -> Result<Board, ()> {
    match init_algo {
        "random" => Ok(Board::random(size)),
        "deter0" => Ok(Board::deterministic_random(size, id)),
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
        "heuristic" => Ok(Box::new(solver::Heuristic)),
        "modcount" => Ok(Box::new(solver::ModCount)),
        other => {
            println!("Player '{}' does not exist!", other);
            Err(())
        }
    }
}
