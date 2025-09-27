use smallvec::{SmallVec, smallvec};
use std::collections::HashMap;

use crate::game_state::board::ChessBoard;
use crate::game_state::board::Color;
use crate::game_state::board::Move;
use crate::game_state::board::Piece;
use crate::game_state::board::PieceType;

#[derive(Clone)]
pub struct PieceList {
    white_king_list: Vec<i16>,
    white_queen_list: Vec<i16>,
    white_rook_list: Vec<i16>,
    white_bishop_list: Vec<i16>,
    white_knight_list: Vec<i16>,
    white_pawn_list: Vec<i16>,

    black_king_list: Vec<i16>,
    black_queen_list: Vec<i16>,
    black_rook_list: Vec<i16>,
    black_bishop_list: Vec<i16>,
    black_knight_list: Vec<i16>,
    black_pawn_list: Vec<i16>,
}

impl PieceList {
    pub fn is_king_in_check(&self, chess_board: &ChessBoard, color: Color) -> Vec<(Piece, i16)> {
        let mut attackers = Vec::new();

        if let Some(king) = self.get_king_square(color) {
            attackers.append(&mut self.get_attackers(chess_board, king, color.opposite()));
        }

        attackers
    }

    fn get_attackers(
        &self,
        chess_board: &ChessBoard,
        king_square: i16,
        by_color: Color,
    ) -> Vec<(Piece, i16)> {
        let mut attackers = Vec::new();

        let attacker_pieces = match by_color {
            Color::White => [
                Piece::WhiteQueen,
                Piece::WhiteRook,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
                Piece::WhitePawn,
                Piece::WhiteKing,
            ],
            Color::Black => [
                Piece::BlackQueen,
                Piece::BlackRook,
                Piece::BlackBishop,
                Piece::BlackKnight,
                Piece::BlackPawn,
                Piece::BlackKing,
            ],
        };

        for attack_piece in attacker_pieces {
            if let Some((attacker_piece, attacker_square)) =
                self.is_attacked_by_piece(chess_board, king_square, attack_piece, by_color)
            {
                attackers.push((attacker_piece, attacker_square));
            }
        }

        attackers
    }

    pub fn generate_legal_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        color: Color,
    ) -> Vec<Move> {
        let king_attackers = self.is_king_in_check(chess_board, color);

        if king_attackers.is_empty() {
            return self.generate_moves(chess_board, color);
        } else if king_attackers.len() == 1 {
            return self.generate_attacker_captures(chess_board, king_attackers, color);
        } else {
            // If multiple attackers, only king moves are possible
            return self.generate_king_moves(chess_board, color);
        }
    }

    fn generate_attacker_captures(
        &mut self,
        chess_board: &mut ChessBoard,
        king_attackers: Vec<(Piece, i16)>,
        color: Color,
    ) -> Vec<Move> {
        let mut valid_moves = Vec::new();

        let Some(king_square) = self.get_king_square(color) else {
            // If there's no king than return empty move list
            return valid_moves;
        };

        let (attacker_piece, attacker_square) = &king_attackers[0];

        // Get squares that block attacker if attacker is a sliding piece
        let mut blocking_squares = vec![*attacker_square];
        match attacker_piece.get_type() {
            PieceType::Queen | PieceType::Rook | PieceType::Bishop => {
                let mut squares = chess_board.get_squares_between(*attacker_square, king_square);

                blocking_squares.append(&mut squares);
            }
            _ => {}
        }

        let pinned_pieces = self.detect_pinned_pieces(chess_board, color);

        valid_moves = self.generate_queen_moves(chess_board, &pinned_pieces, color);
        valid_moves.append(&mut self.generate_rook_moves(chess_board, &pinned_pieces, color));
        valid_moves.append(&mut self.generate_bishop_moves(chess_board, &pinned_pieces, color));
        valid_moves.append(&mut self.generate_knight_moves(chess_board, &pinned_pieces, color));
        valid_moves.append(&mut self.generate_pawn_moves(chess_board, &pinned_pieces, color));

        // Only consider moves that block the attacker or capture it
        valid_moves.retain(|mv| blocking_squares.contains(&mv.to));
        valid_moves.append(&mut self.generate_king_moves(chess_board, color));

        return valid_moves;
    }

    fn generate_moves(&mut self, chess_board: &mut ChessBoard, color: Color) -> Vec<Move> {
        let pinned_pieces = self.detect_pinned_pieces(chess_board, color);

        let mut all_moves = self.generate_king_moves(chess_board, color);
        all_moves.append(&mut self.generate_castling_moves(chess_board, color));
        all_moves.append(&mut self.generate_queen_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_rook_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_bishop_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_knight_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_pawn_moves(chess_board, &pinned_pieces, color));

        all_moves
    }

    fn generate_king_moves(&mut self, chess_board: &mut ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let (king, king_list) = match color {
            Color::White => (Piece::WhiteKing, &self.white_king_list),
            Color::Black => (Piece::BlackKing, &self.black_king_list),
        };

        let king_rays: [i16; 8] = [
            -1,
            1,
            chess_board.board_width,
            -chess_board.board_width,
            chess_board.board_width + 1,
            -chess_board.board_width + 1,
            chess_board.board_width - 1,
            -chess_board.board_width - 1,
        ];

        for &square in king_list {
            for ray in king_rays {
                let position = square + ray;

                // Remove the king to not have the king blocking a square that would otherwise being attacked
                chess_board.set_piece_on_square(Piece::EmptySquare, square);
                // If king will be in check in this position, don't add to possible moves
                if self.is_square_attacked(chess_board, position, color.opposite()) {
                    // Restore king on the board
                    chess_board.set_piece_on_square(king, square);
                    continue;
                }
                // Restore king on the board
                chess_board.set_piece_on_square(king, square);

                let target = chess_board.get_piece_on_square(position);
                if target.is_empty() || target.is_opponent(color) {
                    moves.push(Move::create_move(
                        chess_board,
                        square,
                        position,
                        king,
                        target,
                    ));
                }
            }
        }

        moves
    }

    fn generate_queen_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let (queen, queen_list) = match color {
            Color::White => (Piece::WhiteQueen, &self.white_queen_list),
            Color::Black => (Piece::BlackQueen, &self.black_queen_list),
        };

        let queen_possible_rays: SmallVec<[i16; 4]> = smallvec![
            -1,
            1,
            chess_board.board_width,
            -chess_board.board_width,
            chess_board.board_width + 1,
            -chess_board.board_width + 1,
            chess_board.board_width - 1,
            -chess_board.board_width - 1,
        ];

        for &square in queen_list {
            let mut queen_rays = queen_possible_rays.clone();

            if let Some(pin_direction) = pinned_pieces.get(&square) {
                queen_rays.retain(|dir| (*dir == *pin_direction) || (*dir == -*pin_direction));
            }

            for ray in &queen_rays {
                let mut position = square + ray;
                loop {
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            queen,
                            target,
                        ));
                    } else if target.is_opponent(color) {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            queen,
                            target,
                        ));
                        break;
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }

                    position += ray;
                }
            }
        }

        moves
    }

    fn generate_rook_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (rook, rook_list) = match color {
            Color::White => (Piece::WhiteRook, &self.white_rook_list),
            Color::Black => (Piece::BlackRook, &self.black_rook_list),
        };

        let rook_possible_rays: SmallVec<[i16; 4]> =
            smallvec![1, -1, -chess_board.board_width, chess_board.board_width];

        for &square in rook_list {
            let mut rook_rays = rook_possible_rays.clone();

            if let Some(pin_direction) = pinned_pieces.get(&square) {
                rook_rays.retain(|dir| (*dir == *pin_direction) || (*dir == -*pin_direction));
            }

            for ray in &rook_rays {
                let mut position = square + ray;
                loop {
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            rook,
                            target,
                        ));
                    } else if target.is_opponent(color) {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            rook,
                            target,
                        ));
                        // If there is an enemy in this square, the rook can't go further
                        break;
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }

                    position += ray;
                }
            }
        }

        moves
    }

    fn generate_bishop_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (bishop, bishop_list) = match color {
            Color::White => (Piece::WhiteBishop, &self.white_bishop_list),
            Color::Black => (Piece::BlackBishop, &self.black_bishop_list),
        };

        let bishop_possible_rays: SmallVec<[i16; 4]> = smallvec![
            chess_board.board_width + 1,
            chess_board.board_width - 1,
            -chess_board.board_width + 1,
            -chess_board.board_width - 1,
        ];

        for &square in bishop_list {
            let mut bishop_rays = bishop_possible_rays.clone();
            // If piece is pinned it can only move along pin direction
            if let Some(pin_direction) = pinned_pieces.get(&square) {
                bishop_rays.retain(|dir| (*dir == *pin_direction) || (*dir == -*pin_direction));
            }

            for ray in &bishop_rays {
                let mut position = square + ray;

                loop {
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            bishop,
                            target,
                        ));
                    } else if target.is_opponent(color) {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            bishop,
                            target,
                        ));
                        break;
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }

                    position += ray;
                }
            }
        }

        moves
    }

    fn generate_knight_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (knight, knight_list) = match color {
            Color::White => (Piece::WhiteKnight, &self.white_knight_list),
            Color::Black => (Piece::BlackKnight, &self.black_knight_list),
        };

        let knight_rays: [i16; 8] = [
            2 * chess_board.board_width + 1,
            2 * chess_board.board_width - 1,
            -2 * chess_board.board_width + 1,
            -2 * chess_board.board_width - 1,
            chess_board.board_width * 1 + 2,
            chess_board.board_width * 1 - 2,
            -chess_board.board_width * 1 + 2,
            -chess_board.board_width * 1 - 2,
        ];

        for &square in knight_list {
            // Knights can't move if pinned (they jump, so any pin makes all moves illegal)
            if pinned_pieces.contains_key(&square) {
                continue;
            }

            for ray in knight_rays {
                let target = chess_board.get_piece_on_square(square + ray);
                if target.is_empty() || target.is_opponent(color) {
                    moves.push(Move::create_move(
                        chess_board,
                        square,
                        square + ray,
                        knight,
                        target,
                    ));
                }
            }
        }

        moves
    }

    fn generate_pawn_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (pawn, pawn_list) = match color {
            Color::White => (Piece::WhitePawn, &self.white_pawn_list),
            Color::Black => (Piece::BlackPawn, &self.black_pawn_list),
        };

        let direction: i16 = match color {
            Color::White => chess_board.board_width,
            Color::Black => -chess_board.board_width,
        };

        let promotion_pieces = match color {
            Color::White => [
                Piece::WhiteQueen,
                Piece::WhiteRook,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
            ],
            Color::Black => [
                Piece::BlackQueen,
                Piece::BlackRook,
                Piece::BlackBishop,
                Piece::BlackKnight,
            ],
        };

        let promotion_rank = match color {
            Color::White => chess_board.square_rank(chess_board.algebraic_to_internal("e8")),
            Color::Black => chess_board.square_rank(chess_board.algebraic_to_internal("e1")),
        };

        let double_push_rank = match color {
            Color::White => chess_board.square_rank(chess_board.algebraic_to_internal("e2")),
            Color::Black => chess_board.square_rank(chess_board.algebraic_to_internal("e7")),
        };

        for &square in pawn_list {
            let mut move_forward = true;
            let mut capture_left = true;
            let mut capture_right = true;

            if let Some(pin_direction) = pinned_pieces.get(&square) {
                // If the pawn is being pinned on horizontal direction
                // there's no possible move for this pawn
                if *pin_direction == 1 || *pin_direction == -1 {
                    continue;
                }

                // If the pawn is being pinned on the file, than it can move in the forward direction
                if (*pin_direction == chess_board.board_width)
                    || (*pin_direction == -chess_board.board_width)
                {
                    capture_left = false;
                    capture_right = false;
                }

                // If the pawn is being pinned on the right diagonal, than there's a capture
                // possibility on the right diagonal
                if (*pin_direction == chess_board.board_width + 1)
                    || (*pin_direction == -chess_board.board_width + 1)
                {
                    move_forward = false;
                    capture_left = false;
                }

                // If the pawn is being pinned on the left diagonal, than there's a capture
                // possibility on the left diagonal
                if (*pin_direction == chess_board.board_width - 1)
                    || (*pin_direction == -chess_board.board_width - 1)
                {
                    move_forward = false;
                    capture_right = false;
                }
            }

            let first_target = chess_board.get_piece_on_square(square + direction);
            if move_forward && first_target.is_empty() {
                if chess_board.square_rank(square + direction) != promotion_rank {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction,
                        pawn,
                        first_target,
                        None,
                        false,
                        None,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(Move::create_pawn_move(
                            chess_board,
                            square,
                            square + direction,
                            pawn,
                            first_target,
                            Some(promotion),
                            false,
                            None,
                        ));
                    }
                }
            }

            let target = chess_board.get_piece_on_square(square + direction + 1);
            if capture_right && target.is_opponent(color) {
                if chess_board.square_rank(square + direction + 1) != promotion_rank {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction + 1,
                        pawn,
                        target,
                        None,
                        false,
                        None,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(Move::create_pawn_move(
                            chess_board,
                            square,
                            square + direction + 1,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                            None,
                        ));
                    }
                }
            } else if capture_right
                && (Some(square + direction + 1) == chess_board.get_en_passant_target())
            {
                // Remove the pawns to not having blocking a piece that would give put the king in check
                chess_board.set_piece_on_square(Piece::EmptySquare, square); // Attacker pawn

                let enemy_pawn = chess_board.get_piece_on_square(square + 1);
                chess_board.set_piece_on_square(Piece::EmptySquare, square + 1); // Captured pawn
                // If king would be in check, don't add to possible moves
                if self.is_king_in_check(chess_board, color).is_empty() {
                    // Move don't let king in check so we add to the possible moves
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction + 1,
                        pawn,
                        target,
                        None,
                        true,
                        None,
                    ));
                }
                // Restore pawns on the board
                chess_board.set_piece_on_square(pawn, square); // Attacker pawn
                chess_board.set_piece_on_square(enemy_pawn, square + 1); // Captured pawn
            }

            let target = chess_board.get_piece_on_square(square + direction - 1);
            if capture_left && target.is_opponent(color) {
                if chess_board.square_rank(square + direction - 1) != promotion_rank {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction - 1,
                        pawn,
                        target,
                        None,
                        false,
                        None,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(Move::create_pawn_move(
                            chess_board,
                            square,
                            square + direction - 1,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                            None,
                        ));
                    }
                }
            } else if capture_left
                && (Some(square + direction - 1) == chess_board.get_en_passant_target())
            {
                // Remove the pawns to not having blocking a piece that would give put the king in check
                chess_board.set_piece_on_square(Piece::EmptySquare, square); // Attacker pawn

                let enemy_pawn = chess_board.get_piece_on_square(square - 1);
                chess_board.set_piece_on_square(Piece::EmptySquare, square - 1); // Captured pawn
                // If king would be in check, don't add to possible moves
                if self.is_king_in_check(chess_board, color).is_empty() {
                    // Move don't let king in check so we add to the possible moves
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction - 1,
                        pawn,
                        target,
                        None,
                        true,
                        None,
                    ));
                }

                // Restore pawns on the board
                chess_board.set_piece_on_square(pawn, square); // Attacker pawn
                chess_board.set_piece_on_square(enemy_pawn, square - 1); // Captured pawn
            }

            let target = chess_board.get_piece_on_square(square + 2 * direction);
            if move_forward
                && (color == Color::White)
                && (chess_board.square_rank(square) == double_push_rank)
            {
                if first_target.is_empty() && target.is_empty() {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + 2 * direction,
                        pawn,
                        target,
                        None,
                        false,
                        Some(square + direction),
                    ));
                }
            }

            if move_forward
                && (color == Color::Black)
                && (chess_board.square_rank(square) == double_push_rank)
            {
                if first_target.is_empty() && target.is_empty() {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + 2 * direction,
                        pawn,
                        target,
                        None,
                        false,
                        Some(square + direction),
                    ));
                }
            }
        }

        moves
    }

    fn generate_castling_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let (king_square, king_piece, rook_kingside, rook_queenside) = match color {
            Color::White => (
                chess_board.algebraic_to_internal("e1"),
                Piece::WhiteKing,
                chess_board.algebraic_to_internal("h1"),
                chess_board.algebraic_to_internal("a1"),
            ),
            Color::Black => (
                chess_board.algebraic_to_internal("e8"),
                Piece::BlackKing,
                chess_board.algebraic_to_internal("h8"),
                chess_board.algebraic_to_internal("a8"),
            ),
        };

        let castling_rights = &chess_board.castling_rights;

        // Kingside castling
        if (color == Color::White && castling_rights.white_kingside)
            || (color == Color::Black && castling_rights.black_kingside)
        {
            if self.can_castle_kingside(chess_board, color, king_square, rook_kingside) {
                let king_to = king_square + 2; // g1 or g8
                let rook_to = king_square + 1; // f1 or f8

                moves.push(Move::create_castling_move(
                    chess_board,
                    king_square,
                    king_to,
                    king_piece,
                    rook_kingside,
                    rook_to,
                ));
            }
        }

        // Queenside castling
        if (color == Color::White && castling_rights.white_queenside)
            || (color == Color::Black && castling_rights.black_queenside)
        {
            if self.can_castle_queenside(chess_board, color, king_square, rook_queenside) {
                let king_to = king_square - 2; // c1 or c8
                let rook_to = king_square - 1; // d1 or d8

                moves.push(Move::create_castling_move(
                    chess_board,
                    king_square,
                    king_to,
                    king_piece,
                    rook_queenside,
                    rook_to,
                ));
            }
        }

        moves
    }

    pub fn update_lists(&mut self, board_position: &[Piece; 120]) {
        // The board is our reference, so we can clear all of our lists
        // and set the values from the board to the list
        self.white_pawn_list.clear();
        self.white_rook_list.clear();
        self.white_knight_list.clear();
        self.white_bishop_list.clear();
        self.white_queen_list.clear();
        self.white_king_list.clear();

        self.black_pawn_list.clear();
        self.black_rook_list.clear();
        self.black_knight_list.clear();
        self.black_bishop_list.clear();
        self.black_queen_list.clear();
        self.black_king_list.clear();

        for (square, piece) in board_position.iter().enumerate() {
            // Enumerate returns usize but our squares are i16
            let i16_square = square as i16;
            match piece {
                Piece::WhitePawn => self.white_pawn_list.push(i16_square),
                Piece::WhiteRook => self.white_rook_list.push(i16_square),
                Piece::WhiteKnight => self.white_knight_list.push(i16_square),
                Piece::WhiteBishop => self.white_bishop_list.push(i16_square),
                Piece::WhiteQueen => self.white_queen_list.push(i16_square),
                Piece::WhiteKing => self.white_king_list.push(i16_square),
                Piece::BlackPawn => self.black_pawn_list.push(i16_square),
                Piece::BlackRook => self.black_rook_list.push(i16_square),
                Piece::BlackKnight => self.black_knight_list.push(i16_square),
                Piece::BlackBishop => self.black_bishop_list.push(i16_square),
                Piece::BlackQueen => self.black_queen_list.push(i16_square),
                Piece::BlackKing => self.black_king_list.push(i16_square),
                _ => {}
            }
        }
    }

    pub fn make_move(&mut self, mv: &Move) {
        // Remove captured piece first (if any)
        if mv.captured_piece != Piece::EmptySquare && mv.captured_piece != Piece::SentinelSquare {
            self.remove_piece(mv.captured_piece, mv.to);
        }

        // Handle en passant separately (captured pawn is on different square)
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - 10 // Todo: think of a better way to pass the board width
            } else {
                mv.to + 10
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.remove_piece(captured_pawn, capture_square);
        }

        // Move the piece
        self.remove_piece(mv.piece, mv.from);

        // Add the piece to its new location (or promoted piece)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.add_piece(final_piece, mv.to);

        // Handle castling
        if let Some(castling) = &mv.castling {
            self.remove_piece(castling.rook_piece, castling.rook_from);
            self.add_piece(castling.rook_piece, castling.rook_to);
        }
    }

    pub fn unmake_move(&mut self, mv: &Move) {
        // 1. Handle castling first
        if let Some(castling) = &mv.castling {
            if !self.remove_piece(castling.rook_piece, castling.rook_to) {
                println!("ERROR: Could not remove rook from {}", castling.rook_to);
            }
            self.add_piece(castling.rook_piece, castling.rook_from);
        }

        // 2. Handle en passant
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - 10
            } else {
                mv.to + 10
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };

            // Restore en passant capture
            self.add_piece(captured_pawn, capture_square);
        }

        // 3. Remove moved piece (handle promotion)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        if !self.remove_piece(final_piece, mv.to) {
            println!(
                "ERROR: Could not remove moved piece {} from {}",
                final_piece.print_piece(),
                mv.to
            );
        }

        // 4. Add back the original piece
        self.add_piece(mv.piece, mv.from);

        // 5. Restore captured piece (skip for en passant)
        if !mv.en_passant && mv.captured_piece.is_valid_piece() {
            self.add_piece(mv.captured_piece, mv.to);
        }
    }

    pub fn print_board(&self) {
        // Create an empty 8x8 board
        let mut board = vec!['.'; 64];

        // Helper function to place pieces
        fn place_pieces(board: &mut Vec<char>, pieces: &Vec<i16>, symbol: char) {
            for &square in pieces {
                if square < 64 {
                    board[square as usize] = symbol;
                }
            }
        }

        // Place pieces
        place_pieces(&mut board, &self.white_king_list, 'K');
        place_pieces(&mut board, &self.white_queen_list, 'Q');
        place_pieces(&mut board, &self.white_rook_list, 'R');
        place_pieces(&mut board, &self.white_bishop_list, 'B');
        place_pieces(&mut board, &self.white_knight_list, 'N');
        place_pieces(&mut board, &self.white_pawn_list, 'P');

        place_pieces(&mut board, &self.black_king_list, 'k');
        place_pieces(&mut board, &self.black_queen_list, 'q');
        place_pieces(&mut board, &self.black_rook_list, 'r');
        place_pieces(&mut board, &self.black_bishop_list, 'b');
        place_pieces(&mut board, &self.black_knight_list, 'n');
        place_pieces(&mut board, &self.black_pawn_list, 'p');

        // Print the standard chess board
        println!("\nStandard Chess Board (from Piece Lists):");
        println!("========================================");

        for rank in (0..8).rev() {
            print!("{} │ ", rank + 1);
            for file in 0..8 {
                let index = rank * 8 + file;
                print!("{} ", board[index]);
            }
            println!("│");
        }

        println!("  └─────────────────");
        println!("    a b c d e f g h");
    }

    // Debug function to show all piece lists
    pub fn debug_print(&self) {
        println!("\nPiece List Contents:");
        println!("========================================");

        fn print_list(name: &str, list: &Vec<i16>) {
            let squares: Vec<String> = list.iter().map(|&sq| format!("{}", sq)).collect();
            println!("{:20}: {}", name, squares.join(" "));
        }

        print_list("White Kings", &self.white_king_list);
        print_list("White Queens", &self.white_queen_list);
        print_list("White Rooks", &self.white_rook_list);
        print_list("White Bishops", &self.white_bishop_list);
        print_list("White Knights", &self.white_knight_list);
        print_list("White Pawns", &self.white_pawn_list);

        print_list("Black Kings", &self.black_king_list);
        print_list("Black Queens", &self.black_queen_list);
        print_list("Black Rooks", &self.black_rook_list);
        print_list("Black Bishops", &self.black_bishop_list);
        print_list("Black Knights", &self.black_knight_list);
        print_list("Black Pawns", &self.black_pawn_list);
    }

    fn add_piece(&mut self, piece: Piece, square: i16) {
        let list = self.get_list_mut(piece);
        if let Some(list) = list {
            // Insert in sorted order for consistency
            match list.binary_search(&square) {
                Ok(_) => {} // Already exists (shouldn't happen)
                Err(pos) => list.insert(pos, square),
            }
        }
    }

    fn remove_piece(&mut self, piece: Piece, square: i16) -> bool {
        let list = self.get_list_mut(piece);
        if let Some(list) = list {
            match list.binary_search(&square) {
                Ok(pos) => {
                    list.remove(pos);
                    return true; // Piece found and removed
                }
                Err(_) => {
                    return false; // Doesn't exist (shouldn't happen)
                }
            }
        }
        false // Piece list not found
    }

    fn get_list_mut(&mut self, piece: Piece) -> Option<&mut Vec<i16>> {
        match piece {
            Piece::WhitePawn => Some(&mut self.white_pawn_list),
            Piece::WhiteRook => Some(&mut self.white_rook_list),
            Piece::WhiteKnight => Some(&mut self.white_knight_list),
            Piece::WhiteBishop => Some(&mut self.white_bishop_list),
            Piece::WhiteQueen => Some(&mut self.white_queen_list),
            Piece::WhiteKing => Some(&mut self.white_king_list),
            Piece::BlackPawn => Some(&mut self.black_pawn_list),
            Piece::BlackRook => Some(&mut self.black_rook_list),
            Piece::BlackKnight => Some(&mut self.black_knight_list),
            Piece::BlackBishop => Some(&mut self.black_bishop_list),
            Piece::BlackQueen => Some(&mut self.black_queen_list),
            Piece::BlackKing => Some(&mut self.black_king_list),
            _ => None,
        }
    }

    fn get_list(&self, piece: Piece) -> Option<&Vec<i16>> {
        match piece {
            Piece::WhitePawn => Some(&self.white_pawn_list),
            Piece::WhiteRook => Some(&self.white_rook_list),
            Piece::WhiteKnight => Some(&self.white_knight_list),
            Piece::WhiteBishop => Some(&self.white_bishop_list),
            Piece::WhiteQueen => Some(&self.white_queen_list),
            Piece::WhiteKing => Some(&self.white_king_list),
            Piece::BlackPawn => Some(&self.black_pawn_list),
            Piece::BlackRook => Some(&self.black_rook_list),
            Piece::BlackKnight => Some(&self.black_knight_list),
            Piece::BlackBishop => Some(&self.black_bishop_list),
            Piece::BlackQueen => Some(&self.black_queen_list),
            Piece::BlackKing => Some(&self.black_king_list),
            _ => None,
        }
    }

    pub fn get_number_of_pieces(&self, piece: Piece) -> Option<i64> {
        if let Some(piece_list) = self.get_list(piece) {
            return Some(piece_list.len() as i64);
        }
        None
    }

    fn bishop_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        // Sanity check, the squares can't be the same
        if from == to {
            return false;
        }

        // Check if the squares are in the same diagonal.
        let same_diagonal = chess_board.are_on_the_same_diagonal(from, to);
        if !same_diagonal {
            // If they aren't in the same diagonal the bishop can't move there
            return false;
        }

        // The squares are in the same diagonal, now we need to get in which
        // direction the bishop should move.
        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);
        let row_dir: i16 = if row2 > row1 { 1 } else { -1 };

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);
        let col_dir: i16 = if col2 > col1 { 1 } else { -1 };

        let direction = row_dir * chess_board.board_width + col_dir;

        let mut position = from + direction;
        while position != to {
            let piece = chess_board.get_piece_on_square(position);
            if piece.is_empty() {
                position += direction;
                continue;
            } else {
                return false;
            }
        }

        true
    }

    fn rook_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        // Check if the squares are in the same file or in the same rank.
        let same_rank = chess_board.are_on_the_same_rank(from, to);
        let same_file = chess_board.are_on_the_same_file(from, to);
        if !same_file && !same_rank {
            // If they aren't in the same rank or in the same file,
            // the rook can't move there.
            return false;
        }

        // We now know that the squares are in the same file or in the
        // same rank, we need to get in which direction the rook should
        // move.
        let distance = to - from;
        let direction = if same_rank {
            if distance > 0 { 1 } else { -1 }
        } else {
            if distance > 0 {
                chess_board.board_width
            } else {
                -chess_board.board_width
            }
        };

        let mut position = from + direction;
        while position != to {
            let piece = chess_board.get_piece_on_square(position);
            if piece.is_empty() {
                position += direction;
                continue;
            } else {
                // Blocked by a piece before reaching destination
                return false;
            }
        }

        true
    }

    fn queen_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        // Check if the queen can move like a bishop to the 'to' square
        let bishop = PieceList::bishop_attack(chess_board, from, to);
        if bishop {
            return true;
        }

        // Check if the queen can move like a rook to the 'to' square
        let rook = PieceList::rook_attack(chess_board, from, to);
        if rook {
            return true;
        }

        // Queen can't move there
        false
    }

    fn king_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if row_diff > 1 || col_diff > 1 {
            return false;
        }

        true
    }

    fn knight_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2) {
            return true;
        }

        // Movement doesn't follow an L-shape
        false
    }

    fn pawn_attack(chess_board: &ChessBoard, from: i16, to: i16, color: Color) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if col_diff != 1 || row_diff != 1 {
            // Capture can only happen with a square distance,
            return false;
        }

        // Check if pawn direction is right
        if color == Color::White {
            // White pawn always goes up
            if row2 < row1 {
                return false;
            }
        } else {
            // Black pawn always goes down
            if row2 > row1 {
                return false;
            }
        }

        true
    }

    fn get_king_square(&self, color: Color) -> Option<i16> {
        if color == Color::White {
            if let Some(king_list) = self.get_list(Piece::WhiteKing) {
                if let Some(king) = king_list.get(0) {
                    return Some(*king);
                }
            }
            return None;
        } else {
            if let Some(king_list) = self.get_list(Piece::BlackKing) {
                if let Some(king) = king_list.get(0) {
                    return Some(*king);
                }
            }
            return None;
        }
    }

    fn detect_pinned_pieces(&self, chess_board: &ChessBoard, color: Color) -> HashMap<i16, i16> {
        let mut pinned_pieces = HashMap::new();

        let Some(king_square) = self.get_king_square(color) else {
            return pinned_pieces;
        };

        // Check for pins from each direction
        for direction in &[
            -1,
            1, // Horizontal
            -chess_board.board_width,
            chess_board.board_width, // Vertical
            -chess_board.board_width - 1,
            -chess_board.board_width + 1, // Diagonals
            chess_board.board_width - 1,
            chess_board.board_width + 1,
        ] {
            if let Some((pinned_square, pin_direction)) =
                self.find_pinned_piece_in_direction(chess_board, king_square, *direction, color)
            {
                pinned_pieces.insert(pinned_square, pin_direction);
            }
        }

        pinned_pieces
    }

    fn find_pinned_piece_in_direction(
        &self,
        chess_board: &ChessBoard,
        king_square: i16,
        direction: i16,
        color: Color,
    ) -> Option<(i16, i16)> {
        let mut current = king_square + direction;
        let mut pinned_piece: Option<i16> = None;

        // Move away from king until we hit a piece or board edge
        let mut piece = chess_board.get_piece_on_square(current);
        while !piece.is_sentinel() {
            if !piece.is_empty() {
                if piece.get_color() == color {
                    // First piece we encounter of our color - could be pinned
                    if pinned_piece.is_none() {
                        pinned_piece = Some(current);
                    } else {
                        // Second piece of our color - no pin in this direction
                        return None;
                    }
                } else {
                    // Enemy piece - check if it's a slider that can pin
                    if self.can_piece_pin_in_direction(chess_board, piece, direction) {
                        return pinned_piece.map(|pin_sq| (pin_sq, direction));
                    } else {
                        return None; // Enemy piece can't pin in this direction
                    }
                }
            }

            current += direction;
            piece = chess_board.get_piece_on_square(current);
        }

        None
    }

    fn can_piece_pin_in_direction(
        &self,
        chess_board: &ChessBoard,
        piece: Piece,
        direction: i16,
    ) -> bool {
        match piece.get_type() {
            PieceType::Queen => true, // Queens can pin in any direction
            PieceType::Rook => {
                // Rooks can pin horizontally or vertically
                direction.abs() == 1 || direction.abs() == chess_board.board_width
            }
            PieceType::Bishop => {
                // Bishops can pin diagonally
                direction.abs() == chess_board.board_width - 1
                    || direction.abs() == chess_board.board_width + 1
            }
            _ => false, // Other pieces can't pin
        }
    }

    fn can_castle_kingside(
        &self,
        chess_board: &ChessBoard,
        color: Color,
        king_square: i16,
        rook_square: i16,
    ) -> bool {
        // 0. Check if castling privileges are valid
        if (color == Color::White) && (chess_board.castling_rights.white_kingside != true) {
            return false;
        }

        if (color == Color::Black) && (chess_board.castling_rights.black_kingside != true) {
            return false;
        }

        // 1. Check if king and rook are in starting positions
        if chess_board.get_piece_on_square(king_square)
            != if color == Color::White {
                Piece::WhiteKing
            } else {
                Piece::BlackKing
            }
        {
            return false;
        }

        if chess_board.get_piece_on_square(rook_square)
            != if color == Color::White {
                Piece::WhiteRook
            } else {
                Piece::BlackRook
            }
        {
            return false;
        }

        // 2. Check if squares between king and rook are empty
        let squares_between = match color {
            Color::White => vec![26, 27], // f1, g1
            Color::Black => vec![96, 97], // f8, g8
        };

        for square in squares_between {
            if chess_board.get_piece_on_square(square) != Piece::EmptySquare {
                return false;
            }
        }

        // 3. Check if king is not in check and doesn't move through check
        let check_squares = match color {
            Color::White => vec![25, 26, 27], // e1, f1, g1
            Color::Black => vec![95, 96, 97], // e8, f8, g8
        };

        for square in check_squares {
            let opposite_color = if color == Color::White {
                Color::Black
            } else {
                Color::White
            };
            if self.is_square_attacked(chess_board, square, opposite_color) {
                return false;
            }
        }

        true
    }

    fn can_castle_queenside(
        &self,
        chess_board: &ChessBoard,
        color: Color,
        king_square: i16,
        rook_square: i16,
    ) -> bool {
        // 0. Check if castling privileges are valid
        if (color == Color::White) && (chess_board.castling_rights.white_queenside != true) {
            return false;
        }

        if (color == Color::Black) && (chess_board.castling_rights.black_queenside != true) {
            return false;
        }

        // 1. Check if king and rook are in starting positions
        if chess_board.get_piece_on_square(king_square)
            != if color == Color::White {
                Piece::WhiteKing
            } else {
                Piece::BlackKing
            }
        {
            return false;
        }

        if chess_board.get_piece_on_square(rook_square)
            != if color == Color::White {
                Piece::WhiteRook
            } else {
                Piece::BlackRook
            }
        {
            return false;
        }

        // 2. Check if squares between king and rook are empty
        let squares_between = match color {
            Color::White => vec![22, 23, 24], // b1, c1, d1
            Color::Black => vec![92, 93, 94], // b8, c8, d8
        };

        for square in squares_between {
            if chess_board.get_piece_on_square(square) != Piece::EmptySquare {
                return false;
            }
        }

        // 3. Check if king is not in check and doesn't move through check
        let check_squares = match color {
            Color::White => vec![25, 24, 23], // e1, d1, c1
            Color::Black => vec![95, 94, 93], // e8, d8, c8
        };

        for square in check_squares {
            let opposite_color = if color == Color::White {
                Color::Black
            } else {
                Color::White
            };
            if self.is_square_attacked(chess_board, square, opposite_color) {
                return false;
            }
        }

        true
    }

    fn is_square_attacked(&self, chess_board: &ChessBoard, square: i16, by_color: Color) -> bool {
        let attacker_pieces = match by_color {
            Color::White => [
                Piece::WhiteQueen,
                Piece::WhiteRook,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
                Piece::WhitePawn,
                Piece::WhiteKing,
            ],
            Color::Black => [
                Piece::BlackQueen,
                Piece::BlackRook,
                Piece::BlackBishop,
                Piece::BlackKnight,
                Piece::BlackPawn,
                Piece::BlackKing,
            ],
        };

        for attack_piece in attacker_pieces {
            if self
                .is_attacked_by_piece(chess_board, square, attack_piece, by_color)
                .is_some()
            {
                return true;
            }
        }

        false
    }

    fn is_attacked_by_piece(
        &self,
        chess_board: &ChessBoard,
        square: i16,
        attack_piece: Piece,
        by_color: Color,
    ) -> Option<(Piece, i16)> {
        if let Some(piece_list) = self.get_list(attack_piece) {
            for &piece_square in piece_list {
                let attacks = match attack_piece.get_type() {
                    PieceType::Queen => Self::queen_attack(chess_board, piece_square, square),
                    PieceType::Rook => Self::rook_attack(chess_board, piece_square, square),
                    PieceType::Bishop => Self::bishop_attack(chess_board, piece_square, square),
                    PieceType::Knight => Self::knight_attack(chess_board, piece_square, square),
                    PieceType::Pawn => {
                        Self::pawn_attack(chess_board, piece_square, square, by_color)
                    }
                    PieceType::King => Self::king_attack(chess_board, piece_square, square),
                };

                if attacks {
                    return Some((attack_piece, piece_square));
                }
            }
        }
        None
    }
}

impl Default for PieceList {
    fn default() -> Self {
        PieceList {
            white_king_list: Vec::new(),
            white_queen_list: Vec::new(),
            white_rook_list: Vec::new(),
            white_bishop_list: Vec::new(),
            white_knight_list: Vec::new(),
            white_pawn_list: Vec::new(),

            black_king_list: Vec::new(),
            black_queen_list: Vec::new(),
            black_rook_list: Vec::new(),
            black_bishop_list: Vec::new(),
            black_knight_list: Vec::new(),
            black_pawn_list: Vec::new(),
        }
    }
}

#[cfg(test)]
mod is_square_attacked_tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_pawn_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/8/3p4/4P3/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . p . . . . X │
            03 │ X . . . . P . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                z a b c d e f g h i
        */

        // Black pawn at d4 should attack e3 and c3
        assert!(game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("e3"),
            Color::Black
        ));
        assert!(game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("c3"),
            Color::Black
        ));
        // e4 not attacked
        assert!(!game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("e4"),
            Color::Black
        ));
        // diagonals behind can't be attacked
        assert!(!game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("d5"),
            Color::Black
        ));
        assert!(!game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("c5"),
            Color::Black
        ));
    }

    #[test]
    fn test_knight_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3N4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . N . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        // Knight at d5 should attack 8 squares
        let attacked_squares = ["b4", "b6", "c3", "c7", "e3", "e7", "f4", "f6"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by knight",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("a1");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White),
            "Square {} should not be attacked by knight",
            "a1"
        );
    }

    #[test]
    fn test_bishop_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3B4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . B . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */
        // diagonals
        let attacked_squares = ["a8", "h1", "g8", "a2"];

        // Bishop at d5 should attack diagonals
        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by bishop",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("d4");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White),
            "Square {} should not be attacked by bishop",
            "d4"
        );
    }

    #[test]
    fn test_rook_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3R4/8/8/8/8 w - - 0 1");
        /*
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . R . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */
        let attacked_squares = ["b5", "h5", "d6", "d3"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by rook",
                algebraic_square
            );
        }
        let safe_square = game.board.algebraic_to_internal("e4");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White),
            "Square {} should be attacked by rook",
            "e4"
        );
    }
    #[test]
    fn test_queen_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3Q4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . Q . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let attacked_squares = ["a5", "g5", "d6", "b3", "f7", "b3", "c6", "h1"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by queen",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("g4");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White)
        );
    }

    #[test]
    fn test_king_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3K4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . K . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        // King at d5 should attack surrounding squares
        let attacked_squares = ["c6", "d6", "e6", "c5", "e5", "c4", "d4", "e4"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by king",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("e3"); // too far
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White)
        );
    }

    #[test]
    fn test_blocked_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3R4/3P4/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . R . . . . X │
            04 │ X . . . P . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        // Rook at d5 should not attack squares beyond the pawn at d4
        let attacked_square = game.board.algebraic_to_internal("d4");
        assert!(game.board.piece_list.is_square_attacked(
            &game.board,
            attacked_square,
            Color::White
        ));

        let safe_square = game.board.algebraic_to_internal("d3"); // blocked by pawn
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White)
        );
    }
}

#[cfg(test)]
mod can_castle_queenside_tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_can_castle_queenside_normal() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // White should be able to castle queenside
        assert!(game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));

        // Black should be able to castle queenside
        assert!(game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::Black,
            game.board.algebraic_to_internal("e8"),
            game.board.algebraic_to_internal("a8")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_king_moved() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Simulate king moved by removing castling rights
        game.board.castling_rights.white_queenside = false;

        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_rook_moved() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Simulate rook moved by removing castling rights
        game.board.castling_rights.white_queenside = false;

        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_squares_occupied() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R1B1K2R w KQkq - 0 1");

        // Bishop on c1 blocks queenside castling
        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_through_check() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/8/8/2n5/8/R3K3 w - - 0 1");

        // Black knight attacks d1, which king moves through
        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_in_check() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/8/7b/8/8/R3K3 w - - 0 1");

        // Black bishop attacks e1 (king is in check)
        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_pieces_missing() {
        let mut game = GameState::default();
        game.set_fen_position("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1");

        // No rook on a1
        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_wrong_color() {
        let mut game = GameState::default();
        game.set_fen_position("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Black pieces on white squares shouldn't allow white to castle
        assert!(!game.board.piece_list.can_castle_queenside(
            &game.board,
            Color::White,
            game.board.algebraic_to_internal("e1"), // white king
            game.board.algebraic_to_internal("a8")  // black rook - WRONG ROOK!
        ));
    }
}
