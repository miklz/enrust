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
