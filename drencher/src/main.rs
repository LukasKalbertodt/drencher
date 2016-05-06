extern crate rand;
extern crate docopt;
extern crate rustc_serialize;

mod color;
mod board;
mod solver;

use solver::Solver;
use board::Board;
use docopt::Docopt;

// USAGE-string used by docopt
const USAGE: &'static str = "
Drencher: implementation of the 'drench' game with AI- and human-players.

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
    flag_size: u8, // TODO: nice error message when input is too big
    flag_board: String,
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

    let board = match &args.flag_board[..] {
        "random" => Board::random(args.flag_size),
        other => {
            println!("Intial board algorithm '{}' doesn't exist!", other);
            return;
        }
    };

    // we can't have unsized types on the stack, the following declarations
    // do the trick here...
    let exact;
    let random;
    let human;

    let arg_player = args.arg_player
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("human");
    let player: &Solver = match arg_player {
        "human" => { human = solver::Human; &human },
        "exact" => { exact = solver::Exact; &exact },
        "random" => { random = solver::Random; &random },
        other => {
            println!("Player '{}' does not exist!", other);
            return;
        }
    };

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
        if res.is_err() {
            println!("Game was NOT solved! :-(");
        }
    } else {
        // just print whether or not the game was solved
        match res {
            Ok(_) => println!("Game was solved! :-)"),
            Err(_) => println!("Game was NOT solved! :-("),
        }
    }
}
