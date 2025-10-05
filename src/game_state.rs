//! Game state management and UCI protocol support.
//!
//! This module provides the high-level game state management, including
//! position setup, move execution, search configuration, and UCI protocol
//! integration for chess engine communication.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub mod board;
pub mod uci;
pub use board::CastlingRights;
pub use board::ChessBoard;
pub use board::moves::Move;
pub use board::piece::{Color, Piece};

/// Configuration for search parameters and time control.
///
/// Used to configure the engine's search behavior according to UCI protocol
/// parameters. Supports time controls, depth limits, and various search modes.
#[derive(Clone)]
pub struct SearchConfiguration {
    /// White time remaining in milliseconds
    pub wtime: Option<u64>,
    /// Black time remaining in milliseconds
    pub btime: Option<u64>,
    /// White time increment per move in milliseconds
    pub winc: Option<u64>,
    /// Black time increment per move in milliseconds
    pub binc: Option<u64>,
    /// Moves remaining until next time control
    pub movestogo: Option<u64>,
    /// Time limit for this move in milliseconds
    pub movetime: Option<u64>,
    /// Maximum search depth
    pub depth: Option<u64>,
    /// Maximum number of nodes to search
    pub nodes: Option<u64>,
    /// Search indefinitely until "stop" command
    pub infinite: bool,
    /// Restrict search to specific moves
    pub searchmoves: Option<Vec<Move>>,
    /// Enable pondering (thinking during opponent's time)
    pub ponder: bool,
    /// Search for a mate in specified number of moves
    pub mate: Option<u32>,
}

impl SearchConfiguration {
    /// Creates a new search configuration with default values.
    ///
    /// All time controls and limits are initially unset.
    pub fn new() -> Self {
        SearchConfiguration {
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            movetime: None,
            depth: None,
            nodes: None,
            infinite: false,
            searchmoves: None,
            ponder: false,
            mate: None,
        }
    }

    /// Calculates the recommended time allocation for the current move.
    ///
    /// Implements basic time management strategy:
    /// - Uses `movetime` if specified directly
    /// - Otherwise divides remaining time by moves to go
    /// - Adds increment if available
    /// - Returns `None` for infinite search
    ///
    /// # Arguments
    ///
    /// * `side_to_move` - Color to calculate time for
    ///
    /// # Returns
    ///
    /// Recommended time in milliseconds, or `None` for infinite search
    fn time_for_move(&self, side_to_move: Color) -> Option<Duration> {
        if self.infinite {
            return None;
        }

        if let Some(movetime) = self.movetime {
            return Some(Duration::from_millis(movetime));
        }

        let (time_left, increment) = match side_to_move {
            Color::White => (self.wtime?, self.winc.unwrap_or(0)),
            Color::Black => (self.btime?, self.binc.unwrap_or(0)),
        };

        // Simple time management: use time_left / movestogo or a fraction
        let moves_to_go = self.movestogo.unwrap_or(20) as f64;
        let allocated_time = (time_left as f64 / moves_to_go).min(time_left as f64 * 0.9) as u64;

        Some(Duration::from_millis(allocated_time + increment))
    }
}

/// Main game state container managing the chess position and search configuration.
///
/// Handles position setup, move execution, move generation, and search operations.
/// Integrates with the UCI protocol for engine communication.
pub struct GameState {
    /// Total ply moves made in the game
    ply_moves: u64,
    /// Current side to move
    side_to_move: Color,
    /// Search configuration and time control settings
    search_control: Option<SearchConfiguration>,
    /// Search interrupt
    stop_flag: Arc<AtomicBool>,
    /// The chess board with current position
    board: ChessBoard,
}

impl GameState {
    /// Sets up the standard chess starting position.
    ///
    /// Equivalent to FEN: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    pub fn start_position(&mut self) {
        let fen_start_pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        self.set_fen_position(fen_start_pos);
    }

    /// Sets the time control and search parameters.
    ///
    /// # Arguments
    ///
    /// * `sc` - Search configuration to apply
    pub fn set_time_control(&mut self, sc: &SearchConfiguration) {
        self.search_control = Some(sc.clone());
    }

    /// Sets up the board position from a FEN string.
    ///
    /// FEN format: `<position> <side> <castling> <en passant> <halfmove> <fullmove>`
    ///
    /// # Arguments
    ///
    /// * `fen_str` - FEN string representing the position
    ///
    /// # Returns
    ///
    /// `true` if FEN was parsed successfully, `false` otherwise
    pub fn set_fen_position(&mut self, fen_str: &str) -> bool {
        // FEN: <position> <side to move> <castling rights> <en passant square> <half move number> <full move number>
        let mut fen = fen_str.split_whitespace();

        let mut board_8x8: [Piece; 64] = [Piece::EmptySquare; 64];

        // The first word is the FEN position
        if let Some(fen_position) = fen.next() {
            let rank_strings: Vec<&str> = fen_position.split('/').collect();
            // FEN has 8 ranks, from rank 8 (black side) to rank 1 (white side)
            for (rank_index, rank_str) in rank_strings.iter().enumerate() {
                let mut file_index = 0;

                for c in rank_str.chars() {
                    if file_index >= 8 {
                        break;
                    }

                    if let Some(num_of_empty_squares) = c.to_digit(10) {
                        for _i in 1..=num_of_empty_squares {
                            file_index += 1;
                        }

                        continue;
                    }

                    let piece = match c {
                        'P' => Piece::WhitePawn,
                        'R' => Piece::WhiteRook,
                        'N' => Piece::WhiteKnight,
                        'B' => Piece::WhiteBishop,
                        'Q' => Piece::WhiteQueen,
                        'K' => Piece::WhiteKing,
                        'p' => Piece::BlackPawn,
                        'r' => Piece::BlackRook,
                        'n' => Piece::BlackKnight,
                        'b' => Piece::BlackBishop,
                        'q' => Piece::BlackQueen,
                        'k' => Piece::BlackKing,
                        _ => {
                            println!("Invalid FEN character {}\n", c);
                            return false;
                        }
                    };

                    let board_index = (7 - rank_index) * 8 + file_index;
                    board_8x8[board_index] = piece;
                    file_index += 1;
                }
            }
        } else {
            return false;
        }

        self.board.set_board(&board_8x8);

        // Side to move
        if let Some(side_to_move) = fen.next() {
            match side_to_move {
                "w" => self.side_to_move = Color::White,
                "b" => self.side_to_move = Color::Black,
                _ => return false,
            }
        } else {
            return false;
        }

        let mut white_queenside = false;
        let mut white_kingside = false;
        let mut black_queenside = false;
        let mut black_kingside = false;

        // Castling rights
        if let Some(castling_rights) = fen.next() {
            for c in castling_rights.chars() {
                match c {
                    '-' => break,
                    'K' => white_kingside = true,
                    'Q' => white_queenside = true,
                    'k' => black_kingside = true,
                    'q' => black_queenside = true,
                    _ => return false,
                }
            }
        }

        let castling_rights = CastlingRights {
            white_queenside,
            white_kingside,
            black_queenside,
            black_kingside,
        };
        self.board.set_castling_rights(&castling_rights);

        if let Some(en_passant) = fen.next() {
            if en_passant != "-" {
                if en_passant.len() == 2 {
                    // 0 for 'a', 1 for 'b', …, 7 for 'h'
                    let file = (en_passant.as_bytes()[0] - b'a') as i16;

                    // 0 for rank 1, …, 7 for rank 8
                    let rank = (en_passant.as_bytes()[1] - b'1') as i16;
                    if file < 8 && rank < 8 {
                        let en_passant_square = rank * 8 + file;
                        self.board.set_en_passant_square(en_passant_square);
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        } else {
            return false;
        }

        let mut total_moves;
        // Half move
        if let Some(half_moves_str) = fen.next() {
            if let Ok(half_moves) = half_moves_str.parse::<u64>() {
                total_moves = half_moves;
            } else {
                return false;
            }
        } else {
            return false;
        }

        // Full move
        if let Some(full_moves_str) = fen.next() {
            if let Ok(full_moves) = full_moves_str.parse::<u64>() {
                total_moves += full_moves;
            } else {
                return false;
            }
        } else {
            return false;
        }

        self.ply_moves = total_moves;

        true
    }

    /// Creates a move object from algebraic notation.
    ///
    /// # Arguments
    ///
    /// * `algebraic_notation` - Move in UCI format (e.g., "e2e4")
    ///
    /// # Returns
    ///
    /// `Some(Move)` if the notation is valid, `None` otherwise
    pub fn create_move(&self, algebraic_notation: &str) -> Option<Move> {
        self.board.from_uci(algebraic_notation)
    }

    /// Executes a move on the board.
    ///
    /// # Arguments
    ///
    /// * `algebraic_notation` - Move in UCI format to execute
    pub fn make_move(&mut self, algebraic_notation: &str) {
        match self.create_move(algebraic_notation) {
            Some(mv) => {
                self.board.make_move(&mv);
                self.side_to_move = self.side_to_move.opposite();
            }
            None => (),
        }
    }

    /// Reverts a move on the board.
    ///
    /// # Arguments
    ///
    /// * `algebraic_notation` - Move in UCI format to undo
    pub fn unmake_move(&mut self, algebraic_notation: &str) {
        match self.create_move(algebraic_notation) {
            Some(mv) => {
                self.board.unmake_move(&mv);
                self.side_to_move = self.side_to_move.opposite();
            }
            None => (),
        }
    }

    /// Generates all legal moves for the current position.
    ///
    /// # Returns
    ///
    /// Vector of moves in UCI string format
    pub fn generate_moves(&mut self) -> Vec<String> {
        let moves = self.board.generate_moves(self.side_to_move);

        let move_ucis: Vec<String> = moves.iter().map(|mv| self.board.move_to_uci(mv)).collect();
        move_ucis
    }

    /// Performs a search to find the best move for the current position.
    ///
    /// Uses the configured time control and search parameters.
    ///
    /// # Returns
    ///
    /// Best move in UCI format, or "0000" if no move found
    pub fn search(&mut self) {
        // The time parameters were set with the time requirements from the go command.
        // This method will then, spawn a thread that will interrupt the search after a calculated time
        self.time_manager();

        let mut board_copy = self.board.clone();
        let side_to_move = self.side_to_move;
        self.stop_flag.store(false, Ordering::Release);
        let stop_flag_clone = Arc::clone(&self.stop_flag);

        thread::spawn(
            move || match board_copy.search(side_to_move, stop_flag_clone) {
                Some(mv) => {
                    println!("bestmove {}", board_copy.move_to_uci(&mv));
                }
                None => {
                    println!("bestmove 0000");
                }
            },
        );
    }

    pub fn stop_search(&self) {
        // Force the search thread to stop and return the best move found up to this point
        self.stop_flag.store(true, Ordering::Release);
    }

    /// Manages search time by spawning a timer thread that will interrupt the search
    /// after the allocated time period has elapsed.
    ///
    /// This function calculates the appropriate time allocation for the current move
    /// based on the game state and search configuration, then spawns a background
    /// thread that will call the `stop_search` method when the time expires.
    ///
    /// # Arguments
    ///
    /// * `game_state` - Reference to the current game state containing side to move.
    ///
    /// # Behavior
    ///
    /// - Calculates time allocation using `time_for_move()` based on the current
    ///   player's time remaining, increment, and moves until next time control
    /// - If time allocation is determined (`Some(Duration)`), spawns a timer thread
    /// - The timer thread sleeps for the allocated duration, then calls the
    ///  `game_state.stop_search()`
    /// - If no time allocation is calculated (`None`), no timer is started,
    ///   allowing for infinite search (when `infinite` flag is set in configuration)
    fn time_manager(&self) {
        if let Some(search_control) = &self.search_control {
            if let Some(time_to_think) = search_control.time_for_move(self.side_to_move) {
                // Here we spawn a new thread that will interrupt the search
                // after the calculated time period.
                let stop_flag = self.stop_flag.clone();
                thread::spawn(move || {
                    thread::sleep(time_to_think);
                    stop_flag.store(true, Ordering::Release);
                });
            }
        }
    }

    /// Performs a perft (performance test) for debugging move generation.
    ///
    /// Counts the number of leaf nodes at a given depth for testing move generation correctness.
    ///
    /// # Arguments
    ///
    /// * `depth` - Depth to search (0 returns immediate count)
    /// * `print` - Whether to print move counts for each branch
    ///
    /// # Returns
    ///
    /// Total number of leaf nodes at the specified depth
    pub fn perft_debug(&mut self, depth: u64, print: bool) -> u64 {
        if depth == 0 {
            return 1;
        }

        let color = self.side_to_move;
        let moves = self.board.generate_moves(color);

        if print {
            println!("Depth {}: {} moves", depth, moves.len());
        }

        let mut total_nodes = 0;

        for mv in moves {
            self.board.make_move(&mv);
            self.side_to_move = self.side_to_move.opposite();

            let nodes = if depth == 1 {
                1
            } else {
                self.perft_debug(depth - 1, false)
            };

            if print {
                println!("{}: {}", self.board.move_to_uci(&mv), nodes);
            }

            total_nodes += nodes;

            self.side_to_move = self.side_to_move.opposite();
            self.board.unmake_move(&mv);
        }

        if print {
            println!("Nodes searched: {}", total_nodes);
        }

        total_nodes
    }

    /// Prints the current board state to stdout.
    pub fn print_board(&self) {
        self.board.print_board();
    }

    /// Gets a reference to the underlying chess board.
    ///
    /// # Returns
    ///
    /// Reference to the ChessBoard instance
    pub fn get_chess_board(&self) -> &ChessBoard {
        &self.board
    }
}

impl Default for GameState {
    /// Creates a default game state with standard starting position.
    fn default() -> Self {
        GameState {
            ply_moves: 0,
            side_to_move: Color::White,
            search_control: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            board: ChessBoard::default(),
        }
    }
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
                    uci::handle_uci_command();
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
                    uci::handle_go_command(&mut game_state, &mut uci_cmd);
                }

                "stop" => {
                    game_state.stop_search();
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
