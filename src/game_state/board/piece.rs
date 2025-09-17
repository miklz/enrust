#[derive(PartialEq)]
pub enum PieceType {
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
#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub fn get_color(self) -> Color {
        match self as u8 {
            1..=6 => Color::White,
            7..=12 => Color::Black,
            _ => panic!("Invalid piece"),
        }
    }

    pub fn get_type(self) -> PieceType {
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

    pub fn print_piece(&self) -> &str {
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

    pub fn is_empty(self) -> bool {
        self == Piece::EmptySquare
    }

    pub fn is_sentinel(self) -> bool {
        self == Piece::SentinelSquare
    }

    pub fn is_valid_piece(self) -> bool {
        (self as u8) >= 1 && (self as u8) <= 12
    }

    pub fn is_white(self) -> bool {
        self.is_color(Color::White)
    }

    fn is_color(self, color: Color) -> bool {
        if !self.is_valid_piece() {
            return false;
        }
        self.get_color() == color
    }

    pub fn is_opponent(self, color: Color) -> bool {
        self.is_valid_piece() && self.get_color() != color
    }

    pub fn is_friend(self, color: Color) -> bool {
        self.is_valid_piece() && self.get_color() == color
    }
}
