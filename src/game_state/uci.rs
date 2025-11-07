//! UCI (Universal Chess Interface) protocol implementation.
//!
//! This module provides the UCI protocol handler for communicating with
//! chess GUIs and other UCI-compatible interfaces. It handles command
//! parsing, position setup, search initiation, and response formatting.

use std::str::SplitWhitespace;

use crate::game_state::GameState;
use crate::game_state::SearchConfiguration;

/// Handles the `uci` command by identifying the engine.
///
/// Responds with engine name and author information as required by the UCI protocol.
/// This is typically the first command sent by a GUI to initialize communication.
pub fn handle_uci_command() {
    println!("id name EnRust");
    println!("id author Mikael Ferraz Aldebrand");
    println!("option name Threads type spin default 1 min 1 max 1");
    println!("option name Hash type spin default 256 min 1 max 2048");
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
pub fn handle_go_command(game_state: &mut GameState, tokens: &mut SplitWhitespace) {
    let mut sc = SearchConfiguration::new();

    // Parse all search parameters following the "go" command
    while let Some(token) = tokens.next() {
        match token {
            "wtime" => {
                if let Some(wtime_str) = tokens.next()
                    && let Ok(wtime) = wtime_str.parse::<u64>()
                {
                    sc.wtime = Some(wtime);
                }
            }
            "btime" => {
                if let Some(btime_str) = tokens.next()
                    && let Ok(btime) = btime_str.parse::<u64>()
                {
                    sc.btime = Some(btime);
                }
            }
            "winc" => {
                if let Some(winc_str) = tokens.next()
                    && let Ok(winc) = winc_str.parse::<u64>()
                {
                    sc.winc = Some(winc);
                }
            }
            "binc" => {
                if let Some(binc_str) = tokens.next()
                    && let Ok(binc) = binc_str.parse::<u64>()
                {
                    sc.binc = Some(binc);
                }
            }
            "movestogo" => {
                if let Some(movestogo_str) = tokens.next()
                    && let Ok(movestogo) = movestogo_str.parse::<u64>()
                {
                    sc.movestogo = Some(movestogo);
                }
            }
            "depth" => {
                if let Some(depth_str) = tokens.next()
                    && let Ok(depth) = depth_str.parse::<u64>()
                {
                    sc.depth = Some(depth);
                }
            }
            "nodes" => {
                if let Some(nodes_str) = tokens.next()
                    && let Ok(nodes) = nodes_str.parse::<u64>()
                {
                    sc.nodes = Some(nodes);
                }
            }
            "movetime" => {
                if let Some(movetime_str) = tokens.next()
                    && let Ok(movetime) = movetime_str.parse::<u64>()
                {
                    sc.movetime = Some(movetime);
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
                if let Some(depth_str) = tokens.next()
                    && let Ok(depth) = depth_str.parse::<u64>()
                {
                    game_state.perft_debug(depth, true);
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
    //println!("bestmove {}", game_state.search());
    game_state.search();
}

pub fn handle_setoption_command(game_state: &mut GameState, tokens: &mut SplitWhitespace) {
    // Expect "name" token
    if tokens.next() != Some("name") {
        println!("info string Missing 'name' in setoption command");
        return;
    }

    // Collect the option name (could be multiple words)
    let mut option_name = String::new();
    for token in tokens.by_ref() {
        if token == "value" {
            break;
        }
        if !option_name.is_empty() {
            option_name.push(' ');
        }
        option_name.push_str(token);
    }

    // If we have an option name and found "value", process the value
    if !option_name.is_empty() {
        // Collect the value (could be multiple words)
        let value: String = tokens.collect::<Vec<&str>>().join(" ");

        match option_name.as_str() {
            "Hash" => {
                if let Ok(hash_size) = value.parse::<usize>() {
                    if hash_size > 0 && hash_size <= 2048 {
                        // Reasonable limits
                        game_state.resize_hash_table(hash_size);
                    } else {
                        println!(
                            "info string Hash size {} MB out of range (1-1024)",
                            hash_size
                        );
                    }
                } else {
                    println!("info string Invalid Hash value: '{}'", value);
                }
            }
            _ => {
                // Ignore unsupported options
                println!("info string Unsupported option: '{}'", option_name);
            }
        }
    } else {
        println!("info string Missing option name in setoption command");
    }
}
