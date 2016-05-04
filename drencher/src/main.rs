extern crate rand;
extern crate docopt;
extern crate rustc_serialize;

mod color;
mod board;
mod solver;

use solver::Solver;
use board::Board;
use docopt::Docopt;

const USAGE: &'static str = "
Drencher: implementation of the \"drench\" game with AI- and human-players.

Usage:
  drencher [options] [<player>]
  drencher (-h | --help)
  drencher --version

Options:
  -h --help             Show this screen.
  --version             Show version.
  --size=<size>         Side length of the board [default: 14].
  --board=<initial>     Initial configuration of the board [default: random]
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_player: Option<String>,
    flag_version: bool,
    flag_size: u8, // TODO
    flag_board: String,
}



fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("drencher {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let board = match &*args.flag_board {
        "random" => Board::random(args.flag_size),
        other => panic!("Intial board '{}' doesn't exist!", other),
    };

    let exact;
    let random;
    let human;

    let player: &Solver = match &*args.arg_player.unwrap_or("human".into()) {
        "human" => { human = solver::Human; &human },
        "exact" => { exact = solver::Exact; &exact },
        "random" => { random = solver::Random; &random },
        other => panic!("Player '{}' does not exist!", other),
    };

    let res = player.solve(board.clone());
    if !player.prints_output() {
        let mut board = board;
        if res.is_err() {
            println!("!!! no solution found !!!");
        }
        for c in res.unwrap_or_else(|e| e) {
            println!("Drenching: {}", c);
            board.drench(c);
            println!("{}", board);
        }
        println!("");
    }
}
