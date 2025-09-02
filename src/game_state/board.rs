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
        // Our board has more than 64 squares, but the rest of our application
        // should not have to deal with our board representation, therefore,
        // we have to map the inner board to our complete board with the
        // sentinal squares.

        // The offset for the inner board is equal to the number of squares
        // up to the a1 square.
        // 
        // For a board with 10 files and 12 ranks:
        // 1  . ♜ x x x x x x x .  1
        // 0  . . . . . . . . . .  0
        //-1  . . . . . . . . . . -1
        //    Z A B C D E F G H I
        //
        // That would add up to 21 = 10 + 10 + 1.
        // The 2 times '10' is easy to see, the '1' commes from the difference of the
        // whole board width (10) minus the inner board width (8)...
        // divided by 2 (since we have two sentinel columns in each side of the board).
        let board_offset = 2 * self.board_width + (self.board_width - 8) / 2;
        let real_square = square + board_offset; 
        
        self.board_squares[real_square as usize]
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

    fn map_inner_to_board(square: i16) -> usize {
        // Here we map each of the squares in the 8x8 board
        // to the indexes in the 12x10
        let map : [usize; 64] = [
            21, 22, 23, 24, 25, 26, 27, 28,
            31, 32, 33, 34, 35, 36, 37, 38,
            41, 42, 43, 44, 45, 46, 47, 48,
            51, 52, 53, 54, 55, 56, 57, 58,
            61, 62, 63, 64, 65, 66, 67, 68,
            71, 72, 73, 74, 75, 76, 77, 78,
            81, 82, 83, 84, 85, 86, 87, 88,
            91, 92, 93, 94, 95, 96, 97, 98
        ];

        map[square as usize]
    }

    /* We expect an 8x8 array of pieces*/
    pub fn set_board(&mut self, board_position : &[Piece; 64]) {
        let width = self.board_width as usize;
        let mut index_8x8 = 0;
        for rank in (2..10).rev() { // rows 2..9 are the 8 playable ranks
            for file in 1..9 { // columns 1 to 8 are playable
                self.board_squares[width*rank + file] = board_position[index_8x8];
                index_8x8 += 1;
            }
        }
    }

    pub fn set_en_passant_square(&mut self, square: i16) {
        let board_offset = 2 * self.board_width + (self.board_width - 8) / 2;
        self.en_passant_square = board_offset + square;
    }

    pub fn make_move(&mut self, play: Move) {
        let from_index = Self::map_inner_to_board(play.from);
        let piece = self.board_squares[from_index];
        self.board_squares[from_index] = Piece::EmptySquare;
        /* Let's think how we will handle promotion
        if let Some(piece_promotion) = play.promotion {
            self.board_squares[play.to] = piece_promotion;
        } else {
            self.board_squares[play.to] = piece;
        }
        */
        let to_index = Self::map_inner_to_board(play.to);
        self.board_squares[to_index] = piece;
    }

    pub fn print_board(&self) {
        // Loop over actual board ranks (10 down to 3 in mailbox indexing)
        for rank in (2..10).rev() { // rows 2..9 are the 8 playable ranks
            print!("{} ", rank - 1); // Print rank number (1–8)

            for file in 1..9 { // columns 1 to 8 are playable
                let idx = (rank * self.board_width + file) as usize;
                let symbol = match self.board_squares[idx] {
                    Piece::EmptySquare => ".",
                    Piece::SentinelSquare => "X", // shouldn’t appear inside board
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
                };
                print!("{} ", symbol);
            }
            println!();
        }

        // Print file letters
        println!("  a b c d e f g h");
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