//! UCI (Universal Chess Interface) protocol implementation.
//!
//! This module provides the UCI protocol handler for communicating with
//! chess GUIs and other UCI-compatible interfaces. It handles command
//! parsing, position setup, search initiation, and response formatting.

use std::str::SplitWhitespace;

use crate::game_state::GameState;
use crate::game_state::SearchConfiguration;

use std::io::{self, Write};

/// Handles the `uci` command by identifying the engine.
///
/// Responds with engine name and author information as required by the UCI protocol.
/// This is typically the first command sent by a GUI to initialize communication.
fn handle_uci_command() {
    println!("id name EnRust");
    println!("id author Mikael Ferraz Aldebrand");
    println!("uciok");
}

/// Handles the `go` command to start a search with specified parameters.
///
/// Parses UCI search parameters and initiates the search process. Supports
/// all standard UCI time controls, depth limits, and search modes.
///
/// # Arguments
///
/// * `game_state` - Current game state and position
/// * `tokens` - Command tokens following the "go" keyword
///
/// # Supported Parameters
///
/// - `wtime`, `btime`: Time remaining for white/black
/// - `winc`, `binc`: Time increment per move
/// - `movestogo`: Moves until next time control
/// - `depth`, `nodes`: Search depth/node limits
/// - `movetime`: Fixed time for this move
/// - `infinite`: Search until stopped
/// - `searchmoves`: Restrict search to specific moves
/// - `ponder`: Enable pondering mode
/// - `mate`: Search for mate in N moves
/// - `perft`: Debugging tool for move generation testing
fn handle_go_command(game_state: &mut GameState, tokens: &mut SplitWhitespace) {
    let mut sc = SearchConfiguration::new();

    // Parse all search parameters following the "go" command
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
                // Parse moves until encountering a non-move token
                while let Some(mv) = tokens.clone().next() {
                    if let Some(parsed) = game_state.create_move(mv) {
                        moves.push(parsed);
                        tokens.next(); // advance to next token
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
                return; // Early return for debugging command
            }

            _ => {
                // Ignore unrecognized parameters (UCI protocol requires this)
            }
        }
    }

    // Apply the search configuration and start the search
    game_state.set_time_control(&sc);

    // Output the best move found by the search
    println!("bestmove {}", game_state.search());
}

/// Main UCI protocol loop for handling commands from chess GUIs.
///
/// Implements the UCI protocol state machine that processes commands from
/// standard input and sends responses to standard output. The loop continues
/// until receiving the "quit" command.
///
/// # Supported Commands
///
/// - `uci`: Engine identification
/// - `isready`: Engine readiness check
/// - `ucinewgame`: Start new game
/// - `position`: Set up board position
/// - `go`: Start search with parameters
/// - `quit`: Exit the engine
/// - `print`: Debug command to display board state
///
/// # Protocol Flow
///
/// 1. GUI sends `uci` to initialize
/// 2. Engine responds with identification and `uciok`
/// 3. GUI sends `isready` to check engine status
/// 4. Engine responds with `readyok`
/// 5. GUI sends `position` to set up the board
/// 6. GUI sends `go` to start search
/// 7. Engine responds with `bestmove` when search completes
/// 8. Process repeats until `quit` command
pub fn uci_main() {
    let mut game_state = GameState::default();

    // Main UCI protocol loop
    loop {
        // Read from stdin
        let mut cli_cmd = String::new();
        io::stdin()
            .read_line(&mut cli_cmd)
            .expect("Failed to read command");

        let cmd = cli_cmd.trim();
        let mut uci_cmd = cmd.split_whitespace();

        // Get command keyword and dispatch to appropriate handler
        if let Some(keyword) = uci_cmd.next() {
            match keyword {
                "uci" => {
                    handle_uci_command();
                }
                "isready" => {
                    // Confirm engine is ready to receive commands
                    println!("readyok");
                }
                "ucinewgame" => {
                    // Reset to standard starting position
                    game_state.start_position();
                }
                "quit" => {
                    // Exit the UCI protocol loop
                    break;
                }
                "position" => {
                    let args: Vec<&str> = uci_cmd.collect();

                    if args.is_empty() {
                        println!("info string No position args");
                    } else if args[0] == "startpos" {
                        // Set up standard starting position
                        game_state.start_position();
                        // Apply move sequence if provided
                        if args.len() > 1 && args[1] == "moves" {
                            for mv in &args[2..] {
                                game_state.make_move(mv);
                            }
                        }
                    } else if args[0] == "fen" {
                        // Set up custom position from FEN string
                        if let Some(idx) = args.iter().position(|&x| x == "moves") {
                            // FEN followed by move sequence
                            let fen = args[1..idx].join(" ");
                            game_state.set_fen_position(&fen);
                            for mv in &args[idx + 1..] {
                                game_state.make_move(mv);
                            }
                        } else {
                            // FEN without additional moves
                            let fen = args[1..].join(" ");
                            game_state.set_fen_position(&fen);
                        }
                    }
                }
                "go" => {
                    // Start search with parsed parameters
                    handle_go_command(&mut game_state, &mut uci_cmd);
                }
                // This is not a uci command, is my way of printing the board
                "print" => {
                    // Debug command to display current board state
                    game_state.print_board();
                }
                _ => {
                    // Handle unrecognized commands gracefully
                    println!("info string Unhandled command: {}", cmd);
                }
            }
        }

        // Flush stdout after every response (important for UCI protocol)
        io::stdout().flush().unwrap();
    }
}
