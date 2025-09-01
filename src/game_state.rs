pub mod board;
pub use crate::game_state::board::ChessBoard;
pub use crate::game_state::board::Color;
pub use crate::game_state::board::Move;
pub use crate::game_state::board::Piece;


pub struct GameState {
    ply_moves       : u64,
    side_to_move    : Color,

    // position 0: White king side castle rights
    // position 1: White queen side castle rights
    // position 2: Black king side castle rights
    // position 3: Black queen side castle rights
    castling_rights : (bool, bool, bool, bool),

    board           : ChessBoard,
}

impl GameState {
    pub fn start_position(&mut self) {
        let fen_start_pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        self.set_fen_position(fen_start_pos);
    }

    pub fn set_fen_position(&mut self, fen_str: &str) -> bool {
        // FEN: <position> <side to move> <castling rights> <en passant square> <half move number> <full move number>
        let mut fen = fen_str.split_whitespace();

        let mut board_8x8: [Piece; 64] = [Piece::EmptySquare; 64];
        let mut board_idx = 0;

        // The first word is the FEN position
        if let Some(fen_position) = fen.next() {
            for c in fen_position.chars() {
                if let Some(num_of_empty_squares) = c.to_digit(10) {
                    for _i in 1..=num_of_empty_squares {
                        board_idx += 1;
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
                    '/' => continue,
                    _   => {
                        println!("Invalid FEN character {}\n", c);
                        return false
                    },
                };

                board_8x8[board_idx] = piece;
                board_idx += 1;
            }
        } else {
            return false;
        }

        if board_idx != 64 {
            println!("Invalid FEN string");
            return false;
        }
        
        self.board.set_board(&board_8x8);

        // Side to move
        if let Some(side_to_move) = fen.next() {
            match side_to_move {
                "w" => self.side_to_move = Color::White,
                "b" => self.side_to_move = Color::Black,
                _   => return false,
            }
        } else {
            return false;
        }

        self.castling_rights.0 = false;
        self.castling_rights.1 = false;
        self.castling_rights.2 = false;
        self.castling_rights.3 = false;

        // Castling rights
        if let Some(castling_rights) = fen.next() {
            for c in castling_rights.chars() {
                match c {
                    '-' => break,
                    'K' => self.castling_rights.0 = true,
                    'Q' => self.castling_rights.1 = true,
                    'k' => self.castling_rights.2 = true,
                    'q' => self.castling_rights.3 = true,
                    _   => return false
                }
            }
        }

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

        return true;
    }

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

    /* Convertuci ucialgebraic notation format:
     * <from square><to square>[<promoted to>]
     * to board move
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

    pub fn make_move(&mut self, algebraic_notation: &str) {
        if let Some(play) = Self::parse_algebraic_move(algebraic_notation) {
            self.board.make_move(play);
        }
    }

    pub fn print_board(&self) {
        self.board.print_board();
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            ply_moves: 0,
            side_to_move: Color::White,
            castling_rights : (true, true, true, true),
            board: ChessBoard::default()
        }
    }
}