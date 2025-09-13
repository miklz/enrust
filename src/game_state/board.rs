pub mod piece_list;

use crate::game_state::board::piece_list::PieceList;

#[derive(PartialEq)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

// We derive PartialEq so we can use == and != for color types in our code
#[derive(PartialEq, Copy, Clone)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum Piece {
    EmptySquare = 0,
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
    SentinelSquare = 255,
}

impl Piece {
    fn get_color(self) -> Color {
        match self as u8 {
            1..=6 => Color::White,
            7..=12 => Color::Black,
            _ => panic!("Invalid piece"),
        }
    }

    fn get_type(self) -> PieceType {
        match self {
            Piece::WhitePawn | Piece::BlackPawn => PieceType::Pawn,
            Piece::WhiteKnight | Piece::BlackKnight => PieceType::Knight,
            Piece::WhiteBishop | Piece::BlackBishop => PieceType::Bishop,
            Piece::WhiteRook | Piece::BlackRook => PieceType::Rook,
            Piece::WhiteQueen | Piece::BlackQueen => PieceType::Queen,
            Piece::WhiteKing | Piece::BlackKing => PieceType::King,
            _ => panic!("Invalid piece"),
        }
    }

    fn print_piece(&self) -> &str {
        match self {
            Piece::EmptySquare => ".",
            Piece::SentinelSquare => "X",
            Piece::WhitePawn => "P",
            Piece::WhiteRook => "R",
            Piece::WhiteKnight => "N",
            Piece::WhiteBishop => "B",
            Piece::WhiteQueen => "Q",
            Piece::WhiteKing => "K",
            Piece::BlackPawn => "p",
            Piece::BlackRook => "r",
            Piece::BlackKnight => "n",
            Piece::BlackBishop => "b",
            Piece::BlackQueen => "q",
            Piece::BlackKing => "k",
        }
    }

    fn is_empty(self) -> bool {
        self == Piece::EmptySquare
    }

    fn is_sentinel(self) -> bool {
        self == Piece::SentinelSquare
    }

    fn is_valid_piece(self) -> bool {
        (self as u8) >= 1 && (self as u8) <= 12
    }

    fn is_color(self, color: Color) -> bool {
        if !self.is_valid_piece() {
            return false;
        }
        self.get_color() == color
    }

    fn is_white(self) -> bool {
        self.is_color(Color::White)
    }

    fn is_opponent(self, color: Color) -> bool {
        self.is_valid_piece() && self.get_color() != color
    }

    fn is_friend(self, color: Color) -> bool {
        self.is_valid_piece() && self.get_color() == color
    }
}

#[derive(Clone)]
pub struct Move {
    pub from: i16,
    pub to: i16,
    pub piece: Piece,                     // The moving piece
    pub captured_piece: Piece,            // Piece captured (Empty squares count as pieces)
    pub promotion: Option<Piece>,         // Promotion piece (if any)
    pub castling: Option<CastlingInfo>,   // Castling information
    pub en_passant: bool,                 // Whether this is an en passant capture
    pub en_passant_square: Option<i16>,   // Set when pawn moves two squares
    pub previous_en_passant: Option<i16>, // Previous en passant target
}

#[derive(Clone)]
pub struct CastlingRights {
    pub white_queenside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub black_kingside: bool,
}

#[derive(Clone)]
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
    fn detect_castling(&self, piece: Piece, from: i16, to: i16) -> Option<CastlingInfo> {
        if piece.get_type() == PieceType::King {
            // Kingside castling: e1-g1 or e8-g8
            if (from == 4 && to == 6) || (from == 60 && to == 62) {
                let (rook_from, rook_to) = if from == 4 { (7, 5) } else { (63, 61) };
                let rook_from = self.map_inner_to_outer_board(rook_from);
                let rook_to = self.map_inner_to_outer_board(rook_to);
                let rook_piece = if piece.is_white() {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                };
                return Some(CastlingInfo {
                    rook_from,
                    rook_to,
                    rook_piece,
                });
            }
            // Queenside castling: e1-c1 or e8-c8
            else if (from == 4 && to == 2) || (from == 60 && to == 58) {
                let (rook_from, rook_to) = if from == 4 { (0, 3) } else { (56, 59) };
                let rook_from = self.map_inner_to_outer_board(rook_from);
                let rook_to = self.map_inner_to_outer_board(rook_to);
                let rook_piece = if piece.is_white() {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                };
                return Some(CastlingInfo {
                    rook_from,
                    rook_to,
                    rook_piece,
                });
            }
        }
        None
    }

    fn detect_en_passant(&self, piece: Piece, from: i16, to: i16, captured: Piece) -> bool {
        // En passant: pawn moving diagonally to empty square when en passant target is set
        if piece.get_type() == PieceType::Pawn && captured == Piece::EmptySquare {
            if let Some(ep_target) = self.get_en_passant_target() {
                // Check if this is an en passant capture
                let expected_from = if piece.is_white() {
                    ep_target - self.board_width // White pawn was one rank below
                } else {
                    ep_target + self.board_width // Black pawn was one rank above
                };

                return from == expected_from && to == ep_target;
            }
        }
        false
    }

    /* Convert uci algebraic notation format:
     * <from square><to square>[<promoted to>]
     * to Move struct
     */
    fn parse_algebraic_move(&self, uci_notation: &str) -> Option<Move> {
        /* Convert <rank><file> to 8x8 square */
        fn notation_to_square(square_notation: &str) -> Option<i16> {
            if square_notation.len() != 2 {
                return None;
            }

            let file = square_notation.chars().nth(0).unwrap();
            let rank = square_notation.chars().nth(1).unwrap();

            if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
                return None;
            }

            let file_idx = (file as u8 - b'a') as i16; // a=0, b=1, ...
            let rank_idx = (rank as u8 - b'1') as i16; // 1=0, 2=1, ...

            Some(rank_idx * 8 + file_idx)
        }

        if uci_notation.len() < 4 {
            return None;
        }

        let from = self.map_inner_to_outer_board(notation_to_square(&uci_notation[0..2])?);
        let to = self.map_inner_to_outer_board(notation_to_square(&uci_notation[2..4])?);

        // Get the moving piece from the board
        let moving_piece = self.get_piece_on_square(from);
        if moving_piece == Piece::EmptySquare {
            return None;
        }

        // Get captured piece
        let captured_piece = self.get_piece_on_square(to);

        let promotion = if uci_notation.len() == 5 {
            match &uci_notation[4..5] {
                "q" => Some(if moving_piece.is_white() {
                    Piece::WhiteQueen
                } else {
                    Piece::BlackQueen
                }),
                "r" => Some(if moving_piece.is_white() {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                }),
                "n" => Some(if moving_piece.is_white() {
                    Piece::WhiteKnight
                } else {
                    Piece::BlackKnight
                }),
                "b" => Some(if moving_piece.is_white() {
                    Piece::WhiteBishop
                } else {
                    Piece::BlackBishop
                }),
                _ => None,
            }
        } else {
            None
        };

        let castling = self.detect_castling(moving_piece, from, to);

        let en_passant = self.detect_en_passant(moving_piece, from, to, captured_piece);

        Some(Move {
            from,
            to,
            piece: moving_piece,
            captured_piece,
            promotion,
            castling,
            en_passant,
            en_passant_square: None,
            previous_en_passant: self.get_en_passant_target(),
        })
    }

    pub fn from_uci(&self, uci_notation: &str) -> Option<Move> {
        Self::parse_algebraic_move(&self, uci_notation)
    }

    pub fn move_to_uci(&self, mv: &Move) -> String {
        let from_square = self.square_to_notation(mv.from);
        let to_square = self.square_to_notation(mv.to);

        let promotion_suffix = if let Some(promo_piece) = mv.promotion {
            match promo_piece {
                Piece::WhiteQueen | Piece::BlackQueen => "q",
                Piece::WhiteRook | Piece::BlackRook => "r",
                Piece::WhiteBishop | Piece::BlackBishop => "b",
                Piece::WhiteKnight | Piece::BlackKnight => "n",
                _ => "",
            }
        } else {
            ""
        };

        format!("{}{}{}", from_square, to_square, promotion_suffix)
    }

    fn square_to_notation(&self, square: i16) -> String {
        // Convert from your internal 0-63 representation to algebraic notation
        let chess_square = self.map_to_standard_chess_board(square);
        let file = (chess_square % 8) as u8;
        let rank = (chess_square / 8) as u8;

        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;

        format!("{}{}", file_char, rank_char)
    }

    fn create_pawn_move(
        &self,
        from: i16,
        to: i16,
        piece: Piece,
        captured: Piece,
        promotion: Option<Piece>,
        en_passant: bool,
        en_passant_square: Option<i16>,
    ) -> Move {
        Move {
            from,
            to,
            piece,
            captured_piece: captured,
            promotion,
            castling: None,
            en_passant,
            en_passant_square,
            previous_en_passant: self.get_en_passant_target(),
        }
    }

    fn create_move(&self, from: i16, to: i16, piece: Piece, captured: Piece) -> Move {
        Move {
            from,
            to,
            piece,
            captured_piece: captured,
            promotion: None,
            castling: None,
            en_passant: false,
            en_passant_square: None,
            previous_en_passant: self.get_en_passant_target(),
        }
    }

    fn create_castling_move(
        &self,
        king_from: i16,
        king_to: i16,
        king_piece: Piece,
        rook_from: i16,
        rook_to: i16,
    ) -> Move {
        let color = if king_from == 4 {
            Color::White
        } else {
            Color::Black
        };
        Move {
            from: king_from,
            to: king_to,
            piece: king_piece,
            captured_piece: Piece::EmptySquare,
            promotion: None,
            castling: Some(CastlingInfo {
                rook_from,
                rook_to,
                rook_piece: if color == Color::White {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                },
            }),
            en_passant: false,
            en_passant_square: None,
            previous_en_passant: self.get_en_passant_target(),
        }
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
        let board_offset = 2 * self.board_width + (self.board_width - 8) / 2;
        let chess_square = square - board_offset;

        // rank goes from 1 to 8
        chess_square / self.board_width + 1
    }

    fn square_file(&self, square: i16) -> i16 {
        let board_offset = 2 * self.board_width + (self.board_width - 8) / 2;
        let chess_square = square - board_offset;

        // file goes from 1 to 8
        chess_square % self.board_width + 1
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
        let moves = self
            .piece_list
            .generate_legal_moves(&mut board_copy, side_to_move);

        if moves.is_empty() {
            None
        } else {
            // Return first move for now (random move implementation)
            Some(moves[0].clone())
        }
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
