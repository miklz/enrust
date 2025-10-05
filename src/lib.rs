//! # EnRust Chess Engine
//!
//! A high-performance, UCI-compatible chess engine written in Rust.
//!
//! ## Features
//!
//! - **UCI Protocol Support**: Full compatibility with chess GUIs like Arena, ChessBase, etc.
//! - **Move Generation**: Uses piece lists for O(1) piece access and fast move generation
//! - **Search Algorithms**: Implements minimax with alpha-beta pruning and quiescence search
//! - **Board Representation**: 12x10 mailbox system with sentinel squares for efficient boundary checking
//! - **Complete Chess Rules**: Supports all standard rules including castling, en passant, and promotion
//!
//! ## Architecture
//!
//! The engine is organized into several key modules:
//!
//! - [`game_state`]: High-level game state management including position setup, move execution, search configuration
//! - [`game_state::board`]: Core chess logic including board representation, move generation, and search
//! - [`game_state::uci`]: UCI protocol implementation for GUI communication
//!
//! ### Key Design Features
//!
//! - **Piece Lists**: Instead of scanning the entire board, the engine maintains separate lists
//!   for each piece type, enabling fast move generation
//! - **Pinned Piece Detection**: Sophisticated algorithm that detects pinned pieces to avoid
//!   expensive make/unmake operations during move validation
//! - **Check Evasion**: Move generation that only considers relevant moves when in check
//! - **Mailbox Board**: 12x10 representation that includes sentinel squares around the edges
//!   to simplify boundary checking in move generation
//!
//! ## Usage
//!
//! ### Programmatic Usage
//!
//! You can use the engine programmatically:
//!
//! ```rust
//! use enrust::game_state::{GameState, Color, SearchConfiguration};
//!
//! let mut game_state = GameState::default();
//! game_state.start_position();
//!
//! // Set up time control
//! let mut config = SearchConfiguration::new();
//! config.wtime = Some(60000); // 1 minute for white
//! config.btime = Some(60000); // 1 minute for black
//! game_state.set_time_control(&config);
//!
//! // Generate moves
//! let moves = game_state.generate_moves();
//! println!("Available moves: {:?}", moves);
//!
//! // Search for best move
//! game_state.search();
//! ```
//!
//! ### As a UCI Engine
//!
//! The primary way to use EnRust is as a UCI-compatible chess engine:
//!
//! ```rust
//! use enrust::start_engine;
//!
//! fn main() {
//!     //start_engine();  // Starts the UCI protocol loop
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Move Generation**: O(k) where k is the number of pieces rather than O(n²) for board scanning
//! - **Check Detection**: Constant time using piece list lookups
//! - **Pin Detection**: Linear in the number of directions from the king
//! - **Search**: Exponential in depth but optimized with alpha-beta pruning
//!
//! ## Example UCI Session
//!
//! ```text
//! // GUI sends:
//! uci
//!
//! // Engine responds:
//! id name EnRust
//! id author Mikael Ferraz Aldebrand
//! uciok
//!
//! // GUI sends:
//! isready
//!
//! // Engine responds:
//! readyok
//!
//! // GUI sets up position and starts search:
//! position startpos
//! go wtime 300000 btime 300000 movestogo 40
//!
//! // Engine thinks and responds:
//! bestmove e2e4
//! ```
//!
//! ## Module Overview
//!
//! ### [`game_state::board`] Module
//!
//! Contains the core chess logic:
//!
//! - [`game_state::board::ChessBoard`]: Main board representation with mailbox system
//! - [`game_state::board::piece_list::PieceList`]: Efficient piece tracking data structure
//! - [`game_state::board::moves::Move`]: Chess move representation with metadata
//! - [`game_state::GameState`]: High-level game state management
//! - Search algorithms (minimax, alpha-beta, quiescence)
//!
//! ### [`game_state::uci`] Module
//!
//! Handles the UCI protocol:
//!
//! - Command parsing and response generation
//! - Time control management
//! - Position setup from FEN strings
//! - Search parameter configuration
//!
//! ## Chess Engine Strength
//!
//! The engine implements several features that contribute to its playing strength:
//!
//! - **Alpha-Beta Pruning**: Significantly reduces the search tree size
//! - **Quiescence Search**: Extends search in tactical positions to avoid horizon effect
//! - **Check Evasion**: Proper handling of check situations
//! - **Pin Detection**: Accurate move generation considering pinned pieces
//! - **Material Evaluation**: Basic material counting evaluation function
//!
//! ## Limitations and Future Improvements
//!
//! Current limitations that could be addressed in future versions:
//!
//! - Evaluation function is primarily material-based
//! - No opening book support
//! - No endgame tablebase support
//! - No advanced search enhancements like null-move pruning or transposition tables
//! - No pawn structure evaluation
//! - No king safety evaluation
//!
//! ## Contributing
//!
//! The engine is designed to be extensible. Key areas for contribution include:
//!
//! - Improved evaluation functions
//! - Additional search enhancements
//! - Optimization of critical paths
//! - Additional UCI commands and features
//!
//! ## License
//!
//! MIT License
//!
//! Copyright (c) 2025 Mikael F. Aldebrand
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//!
//! ## Acknowledgments
//!
//! - Inspired by classic chess engine architectures
//! - Uses the SmallVec crate for efficient small vector storage
//! - UCI protocol specification by Stefan Meyer-Kahlen

pub mod game_state;

/// Starts the chess engine in UCI mode.
///
/// This function enters the main UCI protocol loop, waiting for commands
/// from a chess GUI. The engine will respond to UCI commands until it
/// receives the "quit" command.
///
/// # Example
///
/// ```rust
/// use enrust::start_engine;
///
/// fn main() {
///     // This will start the engine and begin listening for UCI commands
///     //start_engine();
/// }
/// ```
///
/// # How it Works
///
/// 1. Initializes a default game state
/// 2. Enters a loop reading commands from stdin
/// 3. Processes UCI commands like "uci", "isready", "position", "go"
/// 4. Sends responses to stdout
/// 5. Exits when "quit" command is received
///
/// This is the main entry point for using EnRust as a UCI chess engine.
pub fn start_engine() {
    game_state::uci_main();
}
