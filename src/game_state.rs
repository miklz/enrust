pub mod board;
pub use board::CastlingRights;
pub use board::ChessBoard;
pub use board::moves::Move;
pub use board::piece::{Color, Piece};

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
    ply_moves: u64,
    side_to_move: Color,
    search_control: Option<SearchConfiguration>,

    board: ChessBoard,
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

        // The first word is the FEN position
        if let Some(fen_position) = fen.next() {
            let rank_strings: Vec<&str> = fen_position.split('/').collect();
            // FEN has 8 ranks, from rank 8 (black side) to rank 1 (white side)
            for (rank_index, rank_str) in rank_strings.iter().enumerate() {
                let mut file_index = 0;

                for c in rank_str.chars() {
                    if file_index >= 8 {
                        break;
                    }

                    if let Some(num_of_empty_squares) = c.to_digit(10) {
                        for _i in 1..=num_of_empty_squares {
                            file_index += 1;
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
                        _ => {
                            println!("Invalid FEN character {}\n", c);
                            return false;
                        }
                    };

                    let board_index = (7 - rank_index) * 8 + file_index;
                    board_8x8[board_index] = piece;
                    file_index += 1;
                }
            }
        } else {
            return false;
        }

        self.board.set_board(&board_8x8);

        // Side to move
        if let Some(side_to_move) = fen.next() {
            match side_to_move {
                "w" => self.side_to_move = Color::White,
                "b" => self.side_to_move = Color::Black,
                _ => return false,
            }
        } else {
            return false;
        }

        let mut white_queenside = false;
        let mut white_kingside = false;
        let mut black_queenside = false;
        let mut black_kingside = false;

        // Castling rights
        if let Some(castling_rights) = fen.next() {
            for c in castling_rights.chars() {
                match c {
                    '-' => break,
                    'K' => white_kingside = true,
                    'Q' => white_queenside = true,
                    'k' => black_kingside = true,
                    'q' => black_queenside = true,
                    _ => return false,
                }
            }
        }

        let castling_rights = CastlingRights {
            white_queenside,
            white_kingside,
            black_queenside,
            black_kingside,
        };
        self.board.set_castling_rights(&castling_rights);

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

        true
    }

    pub fn create_move(&self, algebraic_notation: &str) -> Option<Move> {
        self.board.from_uci(algebraic_notation)
    }

    pub fn make_move(&mut self, algebraic_notation: &str) {
        match self.create_move(algebraic_notation) {
            Some(mv) => {
                self.board.make_move(&mv);
                self.side_to_move = self.side_to_move.opposite();
            }
            None => (),
        }
    }

    pub fn unmake_move(&mut self, algebraic_notation: &str) {
        match self.create_move(algebraic_notation) {
            Some(mv) => {
                self.board.unmake_move(&mv);
                self.side_to_move = self.side_to_move.opposite();
            }
            None => (),
        }
    }

    pub fn generate_moves(&mut self) -> Vec<String> {
        let moves = self.board.generate_moves(self.side_to_move);

        let move_ucis: Vec<String> = moves.iter().map(|mv| self.board.move_to_uci(mv)).collect();
        move_ucis
    }

    pub fn search(&mut self) -> String {
        // The pieces positions was already defined on the board and
        // the time control was set with the time requirements
        // needed to perform a search
        //let sc = self.search_control.as_ref().expect("No time control set");

        match self.board.search(self.side_to_move) {
            Some(mv) => self.board.move_to_uci(&mv),
            None => "0000".to_string(),
        }
    }

    pub fn perft_debug(&mut self, depth: u64, print: bool) -> u64 {
        if depth == 0 {
            return 1;
        }

        let color = self.side_to_move;
        let moves = self.board.generate_moves(color);

        if print {
            println!("Depth {}: {} moves", depth, moves.len());
        }

        let mut total_nodes = 0;

        for mv in moves {
            self.board.make_move(&mv);
            self.side_to_move = self.side_to_move.opposite();

            let nodes = if depth == 1 {
                1
            } else {
                self.perft_debug(depth - 1, false)
            };

            if print {
                println!("{}: {}", self.board.move_to_uci(&mv), nodes);
            }

            total_nodes += nodes;

            self.side_to_move = self.side_to_move.opposite();
            self.board.unmake_move(&mv);
        }

        if print {
            println!("Nodes searched: {}", total_nodes);
        }

        total_nodes
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
            board: ChessBoard::default(),
        }
    }
}
