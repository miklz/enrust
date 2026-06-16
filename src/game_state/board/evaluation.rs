//! Composable evaluation framework for chess positions.
//!
//! This module provides the traits and types for a modular, pluggable
//! evaluation system. Individual heuristic components (material, PST,
//! mobility, etc.) implement the [`HeuristicComponent`] trait, and a
//! [`CompositeEvaluator`] aggregates them into a single evaluation score.
//!
//! Uses **tapered evaluation**: scores are interpolated between midgame
//! and endgame values based on the amount of material remaining on the board.

use crate::game_state::ChessBoard;
use crate::game_state::board::Move;

pub mod material;
pub mod piece_square;

/// Maximum possible phase value (all pieces present).
pub const TOTAL_PHASE: i16 = 256;

/// Represents the current game phase as a value between 0 and [`TOTAL_PHASE`].
///
/// 0 = pure endgame, [`TOTAL_PHASE`] = pure midgame.
#[derive(Clone, Copy, Debug)]
pub struct GamePhase {
    pub phase: i16,
}

impl GamePhase {
    /// Creates a new game phase, clamping to the valid range.
    pub fn new(phase: i16) -> Self {
        Self {
            phase: phase.clamp(0, TOTAL_PHASE),
        }
    }

    /// Returns the phase fraction as a value in [0, TOTAL_PHASE].
    pub fn value(&self) -> i16 {
        self.phase
    }
}

/// A pair of midgame and endgame scores that can be interpolated
/// based on the current game phase.
#[derive(Clone, Copy, Debug)]
pub struct TaperedScore {
    pub mg: i16,
    pub eg: i16,
}

impl TaperedScore {
    /// Creates a new tapered score pair.
    pub const fn new(mg: i16, eg: i16) -> Self {
        Self { mg, eg }
    }

    /// Interpolates between midgame and endgame using the given phase.
    ///
    /// # Arguments
    ///
    /// * `phase` - Current game phase (0 = endgame, TOTAL_PHASE = midgame)
    ///
    /// # Returns
    ///
    /// The tapered score: `(mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE`
    pub fn interpolate(&self, phase: &GamePhase) -> i16 {
        let p = phase.value() as i32;
        let mg = self.mg as i32;
        let eg = self.eg as i32;
        let total = TOTAL_PHASE as i32;
        ((mg * p + eg * (total - p)) / total) as i16
    }
}

/// Full board evaluation from white's perspective, in centipawns.
///
/// Implementations aggregate the contributions of one or more
/// [`HeuristicComponent`]s into a final score.
pub trait Evaluator: Send + Sync {
    /// Evaluates the board from white's perspective.
    fn evaluate(&self, board: &ChessBoard) -> i16;

    /// Incremental update after a move has been applied.
    ///
    /// Default implementation falls back to a full re-evaluation.
    ///
    /// # Arguments
    ///
    /// * `board` - Board state after the move
    /// * `mv` - The move that was just applied
    /// * `prev_score` - Evaluation score before the move
    fn evaluate_incremental(&self, board: &ChessBoard, _mv: &Move, _prev_score: i16) -> i16 {
        self.evaluate(board)
    }
}

/// A single heuristic component contributing to the evaluation.
pub trait HeuristicComponent: Send + Sync {
    /// Returns the component's score from white's perspective.
    ///
    /// # Arguments
    ///
    /// * `board` - The current board state
    /// * `phase` - Current game phase for tapered interpolation
    fn score(&self, board: &ChessBoard, phase: &GamePhase) -> i16;

    /// Returns the incremental delta for this component after a move,
    /// or `None` if incremental update is not supported.
    ///
    /// When `None`, the aggregator falls back to `score()`.
    #[allow(unused_variables)]
    fn delta(&self, board: &ChessBoard, mv: &Move) -> Option<i16> {
        None
    }
}

/// Aggregates multiple [`HeuristicComponent`]s into a single evaluation.
///
/// Iterates through components, summing their contributions. The game
/// phase is computed once and shared across all components.
pub struct CompositeEvaluator {
    components: Vec<Box<dyn HeuristicComponent>>,
}

impl CompositeEvaluator {
    /// Creates a new composite evaluator from a list of heuristic components.
    pub fn new(components: Vec<Box<dyn HeuristicComponent>>) -> Self {
        Self { components }
    }

    /// Computes the game phase from the current board position.
    fn compute_phase(&self, board: &ChessBoard) -> GamePhase {
        let piece_list = &board.piece_list;
        let mut phase = 0i16;

        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::WhiteQueen)
            .unwrap_or(0)
            * 40;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::BlackQueen)
            .unwrap_or(0)
            * 40;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::WhiteRook)
            .unwrap_or(0)
            * 20;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::BlackRook)
            .unwrap_or(0)
            * 20;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::WhiteBishop)
            .unwrap_or(0)
            * 12;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::BlackBishop)
            .unwrap_or(0)
            * 12;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::WhiteKnight)
            .unwrap_or(0)
            * 12;
        phase += piece_list
            .get_number_of_pieces(crate::game_state::Piece::BlackKnight)
            .unwrap_or(0)
            * 12;

        GamePhase::new(phase)
    }
}

impl Default for CompositeEvaluator {
    /// Creates the default evaluator with standard heuristics:
    /// material counting and piece-square tables (PesTO).
    fn default() -> Self {
        Self {
            components: vec![
                Box::new(material::MaterialHeuristic),
                Box::new(piece_square::PieceSquareHeuristic),
            ],
        }
    }
}

impl Evaluator for CompositeEvaluator {
    fn evaluate(&self, board: &ChessBoard) -> i16 {
        let phase = self.compute_phase(board);
        let mut total = 0i16;

        for component in &self.components {
            total += component.score(board, &phase);
        }

        total
    }

    fn evaluate_incremental(&self, board: &ChessBoard, mv: &Move, prev_score: i16) -> i16 {
        let mut total = prev_score;

        for component in &self.components {
            if let Some(delta) = component.delta(board, mv) {
                total += delta;
            } else {
                total = self.evaluate(board);
                break;
            }
        }

        total
    }
}
