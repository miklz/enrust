use crate::game_state::GameState;

use std::io::{self, Write};

fn uci_cmd_protocol() {
    println!("id name EnRust");
    println!("id author Mikael Ferraz Aldebrand");
    println!("uciok");
}

pub fn uci_main() {
    let mut game_state = GameState::default();

    loop {
        // Read from stdin
        let mut cli_cmd = String::new();
        io::stdin()
            .read_line(&mut cli_cmd)
            .expect("Failed to read command");

        let cmd = cli_cmd.trim();
        let mut uci_cmd = cmd.split_whitespace();

        // Get command
        if let Some(keyword) = uci_cmd.next() {
            match keyword {
                "uci" => {
                    uci_cmd_protocol();
                }
                "isready" => {
                    println!("readyok");
                }
                "ucinewgame" => {
                    game_state.start_position();
                }
                "quit" => {
                    break;
                }
                "position" => {
                    let args: Vec<&str> = uci_cmd.collect();

                    if args.is_empty() {
                        println!("info string No position args");
                    } else if args[0] == "startpos" {
                        game_state.start_position();
                        // if "moves ..." follow
                        if args.len() > 1 && args[1] == "moves" {
                            for mv in &args[2..] {
                                game_state.make_move(mv);
                            }
                        }
                    } else if args[0] == "fen" {
                        // "position fen <fen-string> moves ..."
                        // collect FEN until "moves"
                        if let Some(idx) = args.iter().position(|&x| x == "moves") {
                            let fen = args[1..idx].join(" ");
                            game_state.set_fen_position(&fen);
                            for mv in &args[idx + 1..] {
                                game_state.make_move(mv);
                            }
                        } else {
                            let fen = args[1..].join(" ");
                            game_state.set_fen_position(&fen);
                        }
                    }
                }
                "go" => {
                    // "go depth 10", "go movetime 1000", etc.
                    let args: Vec<&str> = uci_cmd.collect();
                    // for now, just a placeholder
                    println!("info string Go called with args: {:?}", args);
                    // for every "go" command a "bestmove" command is needed
                    // println!("bestmove e2e4");
                }
                _ => {
                    // For now, ignore unimplemented commands
                    println!("info string Unhandled command: {}", cmd);
                    game_state.print_board();
                }
            }
        }

        // Flush stdout after every response (important for UCI protocol)
        io::stdout().flush().unwrap();
    }
}