pub mod piece_list;

use crate::game_state::board::piece_list::PieceList;

enum PieceType { King, Queen, Rook, Bishop, Knight, Pawn }

// We derive PartialEq so we can use == and != for color types in our code
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
pub enum Color { White, Black }

#[repr(u8)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq)]
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
    fn color(self) -> Color {
        match self as u8 {
            1..=6 => Color::White,
            7..=12 => Color::Black,
            _ => panic!("Invalid piece"),
        }
    }

    fn piece_type(self) -> PieceType {
        match self {
            Piece::WhitePawn    | Piece::BlackPawn      => PieceType::Pawn,
            Piece::WhiteKnight  | Piece::BlackKnight    => PieceType::Knight,
            Piece::WhiteBishop  | Piece::BlackBishop    => PieceType::Bishop,
            Piece::WhiteRook    | Piece::BlackRook      => PieceType::Rook,
            Piece::WhiteQueen   | Piece::BlackQueen     => PieceType::Queen,
            Piece::WhiteKing    | Piece::BlackKing      => PieceType::King,
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
        self.color() == color
    }

    fn is_opponent(self, color: Color) -> bool {
        self.is_valid_piece() && self.color() != color
    }

    fn is_friend(self, color: Color) -> bool {
        self.is_valid_piece() && self.color() == color
    }
}

#[derive(Clone)]
pub struct Move {
    pub from        : i16,
    pub to          : i16,
//    promotion   : Option<Piece>,
}

impl Move {
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

    /* Convert uci algebraic notation format:
     * <from square><to square>[<promoted to>]
     * to Move struct 
     */
    fn parse_algebraic_move(uci_notation: &str) -> Option<Move> {

        if uci_notation.len() < 4 {
            return None;
        }

        let from = Self::notation_to_square(&uci_notation[0..2])?;
        let to = Self::notation_to_square(&uci_notation[2..4])?;

        /*
        let promotion = if uci_notation.len() == 5 {
            match &uci_notation[4..5] {
                "q" => Some(Piece::WhiteQueen),  // Or handle based on side to move
                "r" => Some(Piece::WhiteRook),
                "n" => Some(Piece::WhiteKnight),
                "b" => Some(Piece::WhiteBishop),
                _   => None,
            }
        } else {
            None
        };
        
        Some(Move { from, to, promotion })
        */

        Some(Move { from, to })
    }

    pub fn from_uci(uci_notation: &str) -> Option<Move> {
        Self::parse_algebraic_move(uci_notation)
    }
}

pub struct ChessBoard {
    board_width     : i16,
    board_height    : i16,
    board_squares   : [Piece;12*10],
    
    // Which square is el passant valid
    en_passant_square: i16,

    piece_list      : PieceList,
}

impl ChessBoard {
    fn get_piece_on_square(&self, square: i16) -> Piece {
        self.board_squares[self.map_inner_to_board(square)]
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

    fn en_passant_square(&self) -> i16 {
        let board_offset = 2 * self.board_width + (self.board_width - 8) / 2;
    
        if self.en_passant_square > board_offset {
            return self.en_passant_square - board_offset;
        }

        self.en_passant_square
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

    fn is_light_square_8x8(square: i16) -> bool {
        let rank = square / 8;
        let file = square % 8;

        (rank + file) % 2 != 0
    }

    fn is_dark_square_8x8(square: i16) -> bool {
        !Self::is_light_square_8x8(square)
    }

    fn is_light_square(&self, square: i16) -> bool {
        let rank = self.square_rank(square);
        let file = self.square_file(square);

        (rank + file) % 2 != 0
    }

    fn is_dark_square(&self, square: i16) -> bool {
        !self.is_light_square(square)
    }

    fn map_inner_to_board(&self, square: i16) -> usize {
        // We have a larger board with sentinel squares around the edges.
        // This function converts a standard 0-63 chess square to its position
        // in our internal board representation.
        
        // Calculate the starting position of the inner 8×8 board within our larger board
        let vertical_padding = self.board_height - 8 / 2;     // Rows above the chess board
        let horizontal_padding = (self.board_width - 8) / 2;  // Columns to the left
        
        let board_offset = vertical_padding * self.board_width + horizontal_padding;
        
        // Convert standard chess coordinates to internal board coordinates
        let chess_rank = self.square_rank(square);  // 1-8 (a1-h1 is rank 1)
        let chess_file = self.square_file(square);  // 1-8 (a-file is 1, h-file is 8)
        
        // Internal position = (rows above) + (chess rank) × (board width) + (columns left) + (chess file)
        let internal_square = self.board_width * (chess_rank - 1) + (chess_file - 1) + board_offset;
        
        internal_square as usize
    }

    /* We expect an 8x8 array of pieces*/
    pub fn set_board(&mut self, board_position : &[Piece; 64]) {
        let width = self.board_width as usize;
        let mut index_8x8 = 0;
        for rank in 2..10 { // rows 2..9 are the 8 playable ranks
            for file in 1..9 { // columns 1 to 8 are playable
                self.board_squares[width*rank + file] = board_position[index_8x8];
                index_8x8 += 1;
            }
        }

        // When the board is set all at once we have to update the piece-lists
        self.piece_list.update_lists(board_position);
    }

    pub fn set_en_passant_square(&mut self, square: i16) {
        let board_offset = 2 * self.board_width + (self.board_width - 8) / 2;
        self.en_passant_square = board_offset + square;
    }

    pub fn make_move(&mut self, play: Move) {
        let from_index = self.map_inner_to_board(play.from);
        let piece = self.board_squares[from_index];
        self.board_squares[from_index] = Piece::EmptySquare;
        /* Let's think how we will handle promotion
        if let Some(piece_promotion) = play.promotion {
            self.board_squares[play.to] = piece_promotion;
        } else {
            self.board_squares[play.to] = piece;
        }
        */
        let to_index = self.map_inner_to_board(play.to);
        self.board_squares[to_index] = piece;
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

    /*
    fn generate_moves(&self, color: Color) -> u64 {
        
    }
    */
}

impl Default for ChessBoard {
    fn default() -> Self {
        ChessBoard {
            board_width: 10,
            board_height: 12,
            board_squares: [Piece::SentinelSquare; 10*12],
            en_passant_square: 0,

            piece_list: PieceList::default()
        }
    }
}