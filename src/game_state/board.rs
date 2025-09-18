pub mod moves;
pub mod piece;
pub mod piece_list;
pub mod search;

use moves::Move;
use piece::{Color, Piece, PieceType};
use piece_list::PieceList;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CastlingRights {
    pub white_queenside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub black_kingside: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CastlingInfo {
    pub rook_from: i16,
    pub rook_to: i16,
    pub rook_piece: Piece,
}

#[derive(Clone)]
pub struct ChessBoard {
    board_width: i16,
    board_height: i16,
    board_squares: [Piece; 12 * 10],

    // Which square is el passant valid
    en_passant_target: Option<i16>,

    castling_rights: CastlingRights,

    piece_list: PieceList,
}

impl ChessBoard {
    fn material_score(&self, piece_list: &PieceList) -> i64 {
        // count pieces
        let w_king = piece_list
            .get_number_of_pieces(Piece::WhiteKing)
            .unwrap_or(0);
        let b_king = piece_list
            .get_number_of_pieces(Piece::BlackKing)
            .unwrap_or(0);
        let w_queen = piece_list
            .get_number_of_pieces(Piece::WhiteQueen)
            .unwrap_or(0);
        let b_queen = piece_list
            .get_number_of_pieces(Piece::BlackQueen)
            .unwrap_or(0);
        let w_rook = piece_list
            .get_number_of_pieces(Piece::WhiteRook)
            .unwrap_or(0);
        let b_rook = piece_list
            .get_number_of_pieces(Piece::BlackRook)
            .unwrap_or(0);
        let w_bishop = piece_list
            .get_number_of_pieces(Piece::WhiteBishop)
            .unwrap_or(0);
        let b_bishop = piece_list
            .get_number_of_pieces(Piece::BlackBishop)
            .unwrap_or(0);
        let w_knight = piece_list
            .get_number_of_pieces(Piece::WhiteKnight)
            .unwrap_or(0);
        let b_kinght = piece_list
            .get_number_of_pieces(Piece::BlackKnight)
            .unwrap_or(0);
        let w_pawn = piece_list
            .get_number_of_pieces(Piece::WhitePawn)
            .unwrap_or(0);
        let b_pawn = piece_list
            .get_number_of_pieces(Piece::BlackPawn)
            .unwrap_or(0);

        let material = 20000 * (w_king - b_king)
            + 900 * (w_queen - b_queen)
            + 500 * (w_rook - b_rook)
            + 300 * (w_bishop - b_bishop + w_knight - b_kinght)
            + 100 * (w_pawn - b_pawn);
        material
    }

    fn evaluate(&self) -> i64 {

        self.material_score(&self.piece_list)
    }

    pub fn is_checkmate(&mut self, color: Color) -> bool {
        let moves = self.generate_moves(color);
        moves.is_empty() && self.is_in_check(color)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        self.piece_list.is_king_in_check(&self, color)
    }

    pub fn from_uci(&self, uci_notation: &str) -> Option<Move> {
        Move::parse_algebraic_move(&self, uci_notation)
    }

    pub fn move_to_uci(&self, mv: &Move) -> String {
        mv.to_uci(&self)
    }

    fn algebraic_to_internal(&self, algebraic_notation: &str) -> i16 {
        if let Some(square) = Move::notation_to_square(algebraic_notation) {
            return self.map_inner_to_outer_board(square);
        }
        -1
    }

    fn get_piece_on_square(&self, square: i16) -> Piece {
        self.board_squares[square as usize]
    }

    fn set_piece_on_square(&mut self, piece: Piece, square: i16) {
        self.board_squares[square as usize] = piece;
    }

    fn are_on_the_same_rank(&self, square1: i16, square2: i16) -> bool {
        // Two squares are on the same rank (row) if their indices divided by board_width are equal.
        // This works because each complete row spans board_width elements.
        //
        // Example (with board_width = 10, including sentinels):
        //
        //  Index layout for a 10x10 board (only a portion shown):
        //      20 21 22 23 24 25 26 27 28 29  ← rank 2
        //      30 31 32 33 34 35 36 37 38 39  ← rank 3
        //
        // To check if two squares are on the same rank:
        //   square1 / board_width == square2 / board_width
        //   → 32 / 10 == 35 / 10 → 3 == 3 → same rank
        square1 / self.board_width == square2 / self.board_width
    }

    fn are_on_the_same_file(&self, square1: i16, square2: i16) -> bool {
        // Two squares are on the same file (column) if their indices modulo board_width are equal.
        // This is because squares in the same vertical column have the same remainder when divided by board_width.

        // Example (with board_width = 10, including sentinels):
        //
        //  Index layout for a 10x10 board (only a portion shown):
        //      20 21 22 23 24 25 26 27 28 29  ← rank 2
        //      30 31 32 33 34 35 36 37 38 39  ← rank 3
        //
        // To check if two squares are on the same file:
        //   square1 % board_width == square2 % board_width
        //   → 23 % 10 == 33 % 10 → 3 == 3 → same file
        square1 % self.board_width == square2 % self.board_width
    }

    fn are_on_the_same_diagonal(&self, square1: i16, square2: i16) -> bool {
        let row1 = square1 / self.board_width;
        let col1 = square1 % self.board_width;

        let row2 = square2 / self.board_width;
        let col2 = square2 % self.board_width;

        // In a grid (like a chess board), a diagonal moves one step in row and one step in column at the same time.
        // So, starting from a square (row, col), you reach diagonals by repeatedly moving in these directions:
        //  top-left: row - 1, col - 1
        //  top-right: row - 1, col + 1
        //  bottom-left: row + 1, col - 1
        //  bottom-right: row + 1, col + 1
        // In all of these, the number of steps taken in rows and columns is equal. That means:
        // The absolute difference between the rows must equal the absolute difference between the columns.
        // So:
        row1.abs_diff(row2) == col1.abs_diff(col2)
        // ensures the movement was exactly diagonal.
    }

    fn get_en_passant_target(&self) -> Option<i16> {
        self.en_passant_target
    }

    fn set_en_passant_target(&mut self, square: Option<i16>) {
        self.en_passant_target = square;
    }

    fn square_rank(&self, square: i16) -> i16 {
        square / self.board_width
    }

    fn square_file(&self, square: i16) -> i16 {
        square % self.board_width
    }

    fn map_inner_to_outer_board(&self, square: i16) -> i16 {
        // We have a larger board with sentinel squares around the edges.
        // This function converts a standard 0-63 chess square to its position
        // in our internal board representation.

        // Calculate the starting position of the inner 8×8 board within our larger board
        let vertical_padding = (self.board_height - 8) / 2; // Rows below the chess board
        let horizontal_padding = (self.board_width - 8) / 2; // Columns to the left

        let board_offset = vertical_padding * self.board_width + horizontal_padding;

        // Convert standard chess coordinates to internal board coordinates
        let chess_rank = square / 8;
        let chess_file = square % 8;

        // Internal position = (rows above) + (chess rank) × (board width) + (columns left) + (chess file)
        let internal_square = self.board_width * chess_rank + chess_file + board_offset;

        internal_square as i16
    }

    fn map_to_standard_chess_board(&self, square: i16) -> usize {
        // Reverse of your map_inner_to_outer_board function
        let board_width = self.board_width;
        let rank = square / board_width;
        let file = square % board_width;

        let chess_rank = rank - 2; // Convert from 2-9 to 0-7
        let chess_file = file - 1; // Convert from 1-8 to 0-7

        (chess_rank * 8 + chess_file) as usize
    }

    fn update_castling_rights(&mut self, mv: &Move) {
        let color = mv.piece.get_color();

        // If king moves, lose both castling rights for that color
        if mv.piece.get_type() == PieceType::King {
            if color == Color::White {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }
        }

        let white_rook_queenside = self.algebraic_to_internal("a1");
        let white_rook_kingside = self.algebraic_to_internal("h1");

        let black_rook_queenside = self.algebraic_to_internal("a8");
        let black_rook_kingside = self.algebraic_to_internal("h8");

        // If rook moves from its starting square, lose corresponding castling right
        match (color, mv.from) {
            (Color::White, square) if square == white_rook_queenside => {
                self.castling_rights.white_queenside = false
            }
            (Color::White, square) if square == white_rook_kingside => {
                self.castling_rights.white_kingside = false
            }
            (Color::Black, square) if square == black_rook_queenside => {
                self.castling_rights.black_queenside = false
            }
            (Color::Black, square) if square == black_rook_kingside => {
                self.castling_rights.black_kingside = false
            }
            _ => {}
        }

        // If a rook is captured, lose corresponding castling right
        if (mv.captured_piece != Piece::EmptySquare)
            && (mv.captured_piece.get_type() == PieceType::Rook)
        {
            match (mv.captured_piece.get_color(), mv.to) {
                (Color::White, square) if square == white_rook_queenside => {
                    self.castling_rights.white_queenside = false
                }
                (Color::White, square) if square == white_rook_kingside => {
                    self.castling_rights.white_kingside = false
                }
                (Color::Black, square) if square == black_rook_queenside => {
                    self.castling_rights.black_queenside = false
                }
                (Color::Black, square) if square == black_rook_kingside => {
                    self.castling_rights.black_kingside = false
                }
                _ => {}
            }
        }

        // If castling move is made, lose both castling rights for that color
        if mv.castling.is_some() {
            if color == Color::White {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }
        }
    }

    /* We expect an 8x8 array of pieces*/
    pub fn set_board(&mut self, board_position: &[Piece; 64]) {
        // Set all squares to invalid
        for square in self.board_squares.iter_mut() {
            *square = Piece::SentinelSquare;
        }

        for (square, &piece) in board_position.iter().enumerate() {
            let inner_square = self.map_inner_to_outer_board(square as i16);
            self.set_piece_on_square(piece, inner_square);
        }

        // When the board is set all at once we have to update the piece-lists
        self.piece_list.update_lists(&self.board_squares);
    }

    pub fn set_en_passant_square(&mut self, square: i16) {
        self.en_passant_target = Some(self.map_inner_to_outer_board(square));
    }

    pub fn set_castling_rights(&mut self, castling_rights: &CastlingRights) {
        self.castling_rights.white_queenside = castling_rights.white_queenside;
        self.castling_rights.white_kingside = castling_rights.white_kingside;
        self.castling_rights.black_queenside = castling_rights.black_queenside;
        self.castling_rights.black_kingside = castling_rights.black_kingside;
    }

    // Given a move, update the board
    pub fn make_move(&mut self, mv: &Move) {
        self.update_castling_rights(mv);

        let piece = mv.piece;

        // If this was an en passant capture
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - self.board_width
            } else {
                mv.to + self.board_width
            };
            self.set_piece_on_square(Piece::EmptySquare, capture_square);
        }

        if let Some(castling) = &mv.castling {
            self.set_piece_on_square(Piece::EmptySquare, castling.rook_from);
            self.set_piece_on_square(castling.rook_piece, castling.rook_to);
        }

        if let Some(piece_promotion) = mv.promotion {
            self.set_piece_on_square(piece_promotion, mv.to);
        } else {
            self.set_piece_on_square(piece, mv.to);
        }

        // When a move is made, the previous square of the piece is cleared
        self.set_piece_on_square(Piece::EmptySquare, mv.from);

        // When pawn moves two squares we update the en passant square
        self.set_en_passant_target(mv.en_passant_square);

        // Update piece list
        self.piece_list.make_move(&mv);
    }

    pub fn unmake_move(&mut self, mv: &Move) {
        // Restaure captured piece
        self.set_piece_on_square(mv.captured_piece, mv.to);

        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - self.board_width
            } else {
                mv.to + self.board_width
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.set_piece_on_square(captured_pawn, capture_square);
        }

        if let Some(castling) = &mv.castling {
            self.set_piece_on_square(castling.rook_piece, castling.rook_from);
            self.set_piece_on_square(Piece::EmptySquare, castling.rook_to);
        }

        if let Some(previous_castling_rights) = mv.previous_castling_rights {
            self.castling_rights = previous_castling_rights;
        }

        // Promotion is undone automatically
        self.set_piece_on_square(mv.piece, mv.from);

        // Restore en passant square to previous state
        self.set_en_passant_target(mv.previous_en_passant);

        self.piece_list.unmake_move(&mv);
    }

    pub fn search(&mut self, side_to_move: Color) -> Option<Move> {
        // We clone the board so that the piece-list
        // can do and undo moves to check for legal moves
        let mut board_copy = self.clone();

        let (_, best_move) = search::pure_minimax_search(&mut board_copy, 4, side_to_move);
        Some(best_move)
    }

    pub fn print_board(&self) {
        println!("\n12x10 Chess Board:");
        println!("==============================");
        // Loop over actual board ranks (10 down to 3 in mailbox indexing)
        for rank in (0..12).rev() {
            print!("{:02} │ ", rank - 1);

            for file in 0..10 {
                let idx = (rank * self.board_width + file) as usize;
                let piece = self.board_squares[idx];

                print!("{} ", piece.print_piece());
            }
            println!("│");
        }

        // Print file letters

        println!("   └─────────────────────");
        println!("     z a b c d e f g h i");

        self.piece_list.debug_print();
    }

    pub fn debug_print(&self) {
        for (square, piece) in self.board_squares.iter().enumerate() {
            print!("{}:{}  ", square, piece.print_piece());
            if square % 10 == 0 {
                println!("");
            }
        }
    }

    pub fn generate_moves(&mut self, color: Color) -> Vec<Move> {
        let mut board_copy = self.clone();
        self.piece_list.generate_legal_moves(&mut board_copy, color)
    }
}

impl Default for ChessBoard {
    fn default() -> Self {
        ChessBoard {
            board_width: 10,
            board_height: 12,
            board_squares: [Piece::SentinelSquare; 10 * 12],
            en_passant_target: None,

            castling_rights: CastlingRights {
                white_kingside: false,
                white_queenside: false,
                black_kingside: false,
                black_queenside: false,
            },

            piece_list: PieceList::default(),
        }
    }
}

#[cfg(test)]
mod chess_board_tests {
    use super::*;

    #[test]
    fn algebraic_to_internal_convertion() {
        let board = ChessBoard::default();

        assert_eq!(board.algebraic_to_internal("e4"), 55);
        assert_eq!(board.algebraic_to_internal("a1"), 21);
        assert_eq!(board.algebraic_to_internal("a8"), 91);
        assert_eq!(board.algebraic_to_internal("h1"), 28);
        assert_eq!(board.algebraic_to_internal("h8"), 98);
    }
}

#[cfg(test)]
mod castling_tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_castling_move_execution() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
        game.board.print_board();

        // Execute kingside castling
        game.make_move("e1g1");
        game.board.print_board();

        // Verify board state after castling
        let king_square = game.board.algebraic_to_internal("g1");
        let rook_square = game.board.algebraic_to_internal("f1");

        assert_eq!(
            game.board.get_piece_on_square(king_square),
            Piece::WhiteKing
        );
        assert_eq!(
            game.board.get_piece_on_square(rook_square),
            Piece::WhiteRook
        );

        // Original squares should be empty
        let original_king = game.board.algebraic_to_internal("e1");
        let original_rook = game.board.algebraic_to_internal("h1");

        assert_eq!(
            game.board.get_piece_on_square(original_king),
            Piece::EmptySquare
        );
        assert_eq!(
            game.board.get_piece_on_square(original_rook),
            Piece::EmptySquare
        );
    }

    #[test]
    fn test_castling_unmake() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        let initial_board = game.board.board_squares.clone();
        let initial_castling = game.board.castling_rights;

        // Make and unmake castling move
        let mv = game
            .create_move("e1g1")
            .expect("Castling move should be valid");
        game.board.make_move(&mv);
        game.board.unmake_move(&mv);

        // Board should be back to initial state
        assert_eq!(game.board.board_squares, initial_board);
        assert_eq!(game.board.castling_rights, initial_castling);
    }

    #[test]
    fn test_complete_castling_scenario() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // White castles kingside
        game.make_move("e1g1");

        // Black castles queenside
        game.make_move("e8c8");

        // Verify final position
        let white_king = game.board.algebraic_to_internal("g1");
        let white_rook = game.board.algebraic_to_internal("f1");
        let black_king = game.board.algebraic_to_internal("c8");
        let black_rook = game.board.algebraic_to_internal("d8");

        assert_eq!(game.board.get_piece_on_square(white_king), Piece::WhiteKing);
        assert_eq!(game.board.get_piece_on_square(white_rook), Piece::WhiteRook);
        assert_eq!(game.board.get_piece_on_square(black_king), Piece::BlackKing);
        assert_eq!(game.board.get_piece_on_square(black_rook), Piece::BlackRook);

        // Castling rights should be lost for both sides
        assert!(!game.board.castling_rights.white_kingside);
        assert!(!game.board.castling_rights.white_queenside);
        assert!(!game.board.castling_rights.black_kingside);
        assert!(!game.board.castling_rights.black_queenside);
    }
}
