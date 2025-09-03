pub mod board;
pub use crate::game_state::board::ChessBoard;
pub use crate::game_state::board::Color;
pub use crate::game_state::board::Move;
pub use crate::game_state::board::Piece;

#[derive(Clone)]
pub struct SearchConfiguration {
    pub wtime: Option<u64>,     // White time in milliseconds
    pub btime: Option<u64>,     // Black time in milliseconds
    pub winc: Option<u64>,      // White increment per move
    pub binc: Option<u64>,      // Black increment per move
    pub movestogo: Option<u64>, // Moves until next time control
    pub movetime: Option<u64>,  // Time limit for this move
    pub depth: Option<u64>,     // Search depth limit
    pub nodes: Option<u64>,     // Node count limit
    pub infinite: bool,         // Search until "stop" command
    pub searchmoves: Option<Vec<Move>>,
    pub ponder: bool,
    pub mate: Option<u32>,
}

impl SearchConfiguration {
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

    // Helper to calculate time allocation for current side to move
    pub fn time_for_move(&self, side_to_move: Color) -> Option<u64> {
        if self.infinite {
            return None;
        }
        
        if let Some(movetime) = self.movetime {
            return Some(movetime);
        }
        
        let (time_left, increment) = match side_to_move {
            Color::White => (self.wtime?, self.winc.unwrap_or(0)),
            Color::Black => (self.btime?, self.binc.unwrap_or(0)),
        };
        
        // Simple time management: use time_left / movestogo or a fraction
        let moves_to_go = self.movestogo.unwrap_or(20) as f64;
        let allocated_time = (time_left as f64 / moves_to_go).min(time_left as f64 * 0.9) as u64;
        
        Some(allocated_time + increment)
    }
}

pub struct GameState {
    ply_moves       : u64,
    side_to_move    : Color,
    search_control  : Option<SearchConfiguration>,

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

    pub fn set_time_control(&mut self, sc: &SearchConfiguration) {
        self.search_control = Some(sc.clone());
    }

    pub fn set_fen_position(&mut self, fen_str: &str) -> bool {
        // FEN: <position> <side to move> <castling rights> <en passant square> <half move number> <full move number>
        let mut fen = fen_str.split_whitespace();

        let mut board_8x8: [Piece; 64] = [Piece::EmptySquare; 64];
        // In a FEN string the black position comes first, but how I think about a chess board,
        // the white pieces start the lowest indexes a1 where the rook is positioned would be the
        // first position (index 0). That's why here I start the index at the end to get the black
        // pieces first, and decrement till we get to the white pieces.
        let mut board_idx = 64;

        // The first word is the FEN position
        if let Some(fen_position) = fen.next() {
            for c in fen_position.chars() {
                if let Some(num_of_empty_squares) = c.to_digit(10) {
                    for _i in 1..=num_of_empty_squares {
                        board_idx -= 1;
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

                board_idx -= 1;
                board_8x8[board_idx] = piece;
            }
        } else {
            return false;
        }

        if board_idx != 0 {
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

    pub fn make_move(&mut self, algebraic_notation: &str) {
        if let Some(play) = Move::from_uci(algebraic_notation) {
            self.board.make_move(play);
        }
    }

    pub fn search(&self) -> Option<&str> {
        // The pieces positions was already defined on the board and
        // the time control was set with the time requirements
        // needed to perform a search
        //let sc = self.search_control.as_ref().expect("No time control set");

        // Stub for now
        Some("e2e4")
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
            search_control: None,
            castling_rights : (true, true, true, true),
            board: ChessBoard::default()
        }
    }
}