use std::str::SplitWhitespace;

use crate::game_state::GameState;
use crate::game_state::SearchConfiguration;

use std::io::{self, Write};

fn handle_uci_command() {
    println!("id name EnRust");
    println!("id author Mikael Ferraz Aldebrand");
    println!("uciok");
}

fn handle_go_command(game_state: &mut GameState, tokens: &mut SplitWhitespace) {
    let mut sc = SearchConfiguration::new();
    while let Some(token) = tokens.next() {
        match token {
            "wtime" => {
                if let Some(wtime_str) = tokens.next() {
                    if let Ok(wtime) = wtime_str.parse::<u64>() {
                        sc.wtime = Some(wtime);
                    }
                }
            }
            "btime" => {
                if let Some(btime_str) = tokens.next() {
                    if let Ok(btime) = btime_str.parse::<u64>() {
                        sc.btime = Some(btime);
                    }
                }
            }
            "winc" => {
                if let Some(winc_str) = tokens.next() {
                    if let Ok(winc) = winc_str.parse::<u64>() {
                        sc.winc = Some(winc);
                    }
                }
            }
            "binc" => {
                if let Some(binc_str) = tokens.next() {
                    if let Ok(binc) = binc_str.parse::<u64>() {
                        sc.binc = Some(binc);
                    }
                }
            }
            "movestogo" => {
                if let Some(movestogo_str) = tokens.next() {
                    if let Ok(movestogo) = movestogo_str.parse::<u64>() {
                        sc.movestogo = Some(movestogo);
                    }
                }
            }
            "depth" => {
                if let Some(depth_str) = tokens.next() {
                    if let Ok(depth) = depth_str.parse::<u64>() {
                        sc.depth = Some(depth);
                    }
                }
            }
            "nodes" => {
                if let Some(nodes_str) = tokens.next() {
                    if let Ok(nodes) = nodes_str.parse::<u64>() {
                        sc.nodes = Some(nodes);
                    }
                }
            }
            "movetime" => {
                if let Some(movetime_str) = tokens.next() {
                    if let Ok(movetime) = movetime_str.parse::<u64>() {
                        sc.movetime = Some(movetime);
                    }
                }
            }
            "infinite" => sc.infinite = true,

            "searchmoves" => {
                let mut moves = Vec::new();
                while let Some(mv) = tokens.clone().next() {
                    if let Some(parsed) = game_state.create_move(mv) {
                        moves.push(parsed);
                        tokens.next(); // advance
                    } else {
                        break; // stop at non-move token
                    }
                }
                sc.searchmoves = Some(moves);
            }

            "ponder" => {
                sc.ponder = true;
            }

            "mate" => {
                if let Some(n) = tokens.next().and_then(|t| t.parse::<u32>().ok()) {
                    sc.mate = Some(n);
                }
            }

            // Not a standard UCI command, but a some debugging tools need this to test the engine
            "perft" => {
                if let Some(depth_str) = tokens.next() {
                    if let Ok(depth) = depth_str.parse::<u64>() {
                        game_state.perft_debug(depth, true);
                    }
                }
            }

            _ => {}
        }
    }

    game_state.set_time_control(&sc);

    println!("bestmove {}", game_state.search());
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
                    handle_uci_command();
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
                    handle_go_command(&mut game_state, &mut uci_cmd);
                }
                // This is not a uci command, is my way of printing the board
                "print" => {
                    game_state.print_board();
                }
                _ => {
                    // For now, ignore unimplemented commands
                    println!("info string Unhandled command: {}", cmd);
                }
            }
        }

        // Flush stdout after every response (important for UCI protocol)
        io::stdout().flush().unwrap();
    }
}
