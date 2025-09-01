use crate::game_state::board::Color;
use crate::game_state::board::Piece;
use crate::game_state::board::Move;
use crate::game_state::board::ChessBoard;


pub struct PieceList {
    white_king_list         : Vec<i16>,
    white_queen_list        : Vec<i16>,
    white_rook_list         : Vec<i16>,
    white_light_bishop_list : Vec<i16>,
    white_dark_bishop_list  : Vec<i16>,
    white_knight_list       : Vec<i16>,
    white_pawn_list         : Vec<i16>,

    black_king_list         : Vec<i16>,
    black_queen_list        : Vec<i16>,
    black_rook_list         : Vec<i16>,
    black_light_bishop_list : Vec<i16>,
    black_dark_bishop_list  : Vec<i16>,
    black_knight_list       : Vec<i16>,
    black_pawn_list         : Vec<i16>,
}

impl PieceList {
    fn generate_king_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let king_list = match color {
            Color::White => &self.white_king_list,
            Color::Black => &self.black_king_list
        };

        let king_rays: [i16; 8] = [-1, 1, chess_board.board_width, -chess_board.board_width,
                        chess_board.board_width + 1, -chess_board.board_width + 1,
                        chess_board.board_width - 1, -chess_board.board_width - 1];

        for &king in king_list {
            for ray in king_rays {
                let piece = chess_board.get_piece_on_square(king + ray);
                if piece.is_empty() {
                    moves.push(Move {from: king, to: king + ray});
                }

                if piece.is_opponent(color) {
                    moves.push(Move {from: king, to: king + ray});
                }
            }
        }

        moves
    }

    fn generate_queen_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let queen_list = match color {
            Color::White => &self.white_queen_list,
            Color::Black => &self.black_queen_list
        };

        let queen_rays: [i16; 8] = [-1, 1, chess_board.board_width, -chess_board.board_width,
                        chess_board.board_width + 1, -chess_board.board_width + 1,
                        chess_board.board_width - 1, -chess_board.board_width - 1];

        for &queen in queen_list {
            for ray in queen_rays {
                let mut position = queen;
                loop {
                    position += ray;

                    let piece = chess_board.get_piece_on_square(position);
                    if piece.is_empty() {
                        moves.push(Move {from: queen, to: position});
                    }

                    if piece.is_opponent(color) {
                        moves.push(Move {from: queen, to: position});
                    }
                }
            }
        }

        moves
    }

    fn generate_rook_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let rook_list = match color {
            Color::White => &self.white_rook_list,
            Color::Black => &self.black_rook_list
        };

        let rook_rays: [i16; 4] = [1, -1, -chess_board.board_width, chess_board.board_width];

        for &rook in rook_list {
            for ray in rook_rays {
                let mut position = rook;
                loop {
                    position += ray;

                    let piece = chess_board.get_piece_on_square(position);
                    if piece.is_empty() {
                        moves.push(Move {from: rook, to: position});
                    }

                    if piece.is_opponent(color) {
                        moves.push(Move {from: rook, to: position});
                    }
                }
            }
        }

        moves
    }

    fn generate_bishop_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let bishop_light_list = match color {
            Color::White => &self.white_light_bishop_list,
            Color::Black => &self.black_light_bishop_list
        };

        let bishop_dark_list = match color {
            Color::White => &self.white_dark_bishop_list,
            Color::Black => &self.black_dark_bishop_list
        };

        let bishop_rays: [i16; 4] = [chess_board.board_width + 1, chess_board.board_width - 1,
                            -chess_board.board_width + 1, chess_board.board_width - 1];

        for &bishop in bishop_light_list {
            for ray in bishop_rays {
                let mut position = bishop;
                loop {
                    position += ray;
                    
                    let piece = chess_board.get_piece_on_square(position);
                    if piece.is_empty() {
                        moves.push(Move {from: bishop, to: position});
                    }

                    if piece.is_opponent(color) {
                        moves.push(Move {from: bishop, to: position});
                    }
                }
            }
        }

        for &bishop in bishop_dark_list {
            for ray in bishop_rays {
                let mut position = bishop;
                loop {
                    position += ray;

                    let piece = chess_board.get_piece_on_square(position);
                    if piece.is_empty() {
                        moves.push(Move {from: bishop, to: position});
                    }

                    if piece.is_opponent(color) {
                        moves.push(Move {from: bishop, to: position});
                    }
                }
            }
        }

        moves
    }

    fn generate_knight_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let knight_list = match color {
            Color::White => &self.white_knight_list,
            Color::Black => &self.black_knight_list
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

        for &knight in knight_list {
            for ray in knight_rays {
                let piece = chess_board.get_piece_on_square(knight + ray);
                if piece.is_empty() {
                    moves.push(Move {from: knight, to: knight + ray});
                }

                if piece.is_opponent(color) {
                    moves.push(Move {from: knight, to: knight + ray});
                }
            }
        }

        moves
    }

    fn generate_pawn_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let pawn_list = match color {
            Color::White => &self.white_pawn_list,
            Color::Black => &self.black_pawn_list,
        };

        let direction: i16 = match color {
            Color::White => chess_board.board_width,
            Color::Black => -chess_board.board_width,
        };

        for &pawn in pawn_list {
            if chess_board.get_piece_on_square(pawn + direction).is_empty() {
                moves.push(Move {from: pawn, to: pawn + direction});
            }

            if chess_board.get_piece_on_square(pawn + direction + 1).is_opponent(color) {
                moves.push(Move {from: pawn, to: pawn + direction + 1});
            } else if pawn + direction + 1 == chess_board.en_passant_square() {
                moves.push(Move {from: pawn, to: pawn + direction + 1});
            }

            if chess_board.get_piece_on_square(pawn + direction - 1).is_opponent(color) {
                moves.push(Move {from: pawn, to: pawn + direction - 1});
            }  else if pawn + direction - 1 == chess_board.en_passant_square() {
                moves.push(Move {from: pawn, to: pawn + direction - 1});
            }

            if (color == Color::White) && (chess_board.square_rank(pawn) == 2) {
                if chess_board.get_piece_on_square(pawn + 2 * direction).is_empty() {
                    moves.push(Move {from: pawn, to: pawn + 2 * direction});
                }
            }

            if (color == Color::Black) && (chess_board.square_rank(pawn) == 7) {
                if chess_board.get_piece_on_square(pawn + 2 * direction).is_empty() {
                    moves.push(Move {from: pawn, to: pawn + 2 * direction});
                }
            }
        }

        moves
    }

    pub fn init(&mut self) {
        self.white_king_list.push(25);

        self.white_queen_list.push(24);

        self.white_rook_list.push(21);
        self.white_rook_list.push(28);

        self.white_light_bishop_list.push(26);

        self.white_light_bishop_list.push(23);

        self.white_knight_list.push(22);
        self.white_knight_list.push(27);

        for white_pawn_square in 31..=38 {
            self.white_pawn_list.push(white_pawn_square);
        }

        self.black_king_list.push(95);

        self.black_queen_list.push(94);

        self.black_rook_list.push(91);
        self.black_rook_list.push(98);
        
        self.black_light_bishop_list.push(93);

        self.black_light_bishop_list.push(96);

        self.black_knight_list.push(92);
        self.black_knight_list.push(97);

        for black_pawn_square in 81..=88 {
            self.black_pawn_list.push(black_pawn_square);
        }
    }

    pub fn make_move(&mut self, chess_board: &ChessBoard, play: &Move) {
        let piece = chess_board.get_piece_on_square(play.from);

        let mut piece_list = match piece {
            Piece::WhitePawn    => &mut self.white_pawn_list,
            Piece::WhiteRook    => &mut self.white_rook_list,
            Piece::WhiteKnight  => &mut self.white_knight_list,
            //Piece::WhiteBishop  => &mut self.white_bishop_list,
            Piece::WhiteQueen   => &mut self.white_queen_list,
            Piece::WhiteKing    => &mut self.white_king_list,
            Piece::BlackPawn    => &mut self.black_pawn_list,
            Piece::BlackRook    => &mut self.black_rook_list,
            Piece::BlackKnight  => &mut self.black_knight_list,
            //Piece::BlackBishop  => &mut self.black_bishop_list,
            Piece::BlackQueen   => &mut self.black_queen_list,
            Piece::BlackKing    => &mut self.black_king_list,
            _ => return,
        };

        if let Some(index) = piece_list.iter().position(|&piece_square| piece_square == play.from) {
            piece_list[index] = play.to;
        }

        /* Next step is to handle promotion
        if let Some(promotion) = play.promotion {
            piece_list[index] = promotion;
        } else {
            piece_list[index] = play.to;
        }
        */
    }

    fn bishop_move(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        // Sanity check, the squares can't be the same
        if from == to {
            return false
        }
        // Check if the squares are in the same diagonal.
        let same_diagonal = chess_board.are_on_the_same_diagonal(from, to);
        if !same_diagonal {
            // If they aren't in the same diagonal the bishop can't move there
            return false
        }

        // The squares are in the same diagonal, now we need to get in which
        // direction the bishop should move.
        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);
        let row_dir : i16 = if row2 > row1 { 1 } else { -1 };

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_rank(to);
        let col_dir : i16 = if col2 > col1 { 1 } else { -1 };

        let direction = row_dir * chess_board.board_width + col_dir;


        let moving_color = chess_board.get_piece_on_square(from).color();
        let mut position = from;
        while position != to {
            position += direction;
            
            let piece = chess_board.get_piece_on_square(position);
            if piece.is_empty() {
                continue;
            } else {
                // If this is the destination square, allow capture if colors differ
                if position == to {
                    return piece.is_opponent(moving_color);
                } else {
                    // Blocked by a piece before reaching destination
                    return false;
                }
            }
        }

        // The path is clear for the bishop to move there
        true
    }

    fn rook_move(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
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
            if distance > 0 {1} else {-1}
        } else {
            if distance > 0 {chess_board.board_width} else {-chess_board.board_width}
        };

        let moving_color = chess_board.get_piece_on_square(from).color();
        let mut position = from;
        while position != to {
            position += direction;
            
            let piece = chess_board.get_piece_on_square(position);
            if piece.is_empty() {
                continue;
            } else {
                // If this is the destination square, allow capture if colors differ
                if position == to {
                    return piece.is_opponent(moving_color);
                } else {
                    // Blocked by a piece before reaching destination
                    return false;
                }
            }
        }

        // The path is clear for the rook to move there
        true
    }

    fn queen_move(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        // Check if the queen can move like a bishop to the 'to' square
        let bishop = PieceList::bishop_move(chess_board, from, to);
        if bishop {
            return true;
        }

        // Check if the queen can move like a rook to the 'to' square
        let rook = PieceList::rook_move(chess_board, from, to);
        if rook {
            return true;
        }

        // Queen can't move there
        false
    }

    fn king_move(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if row_diff <= 1 && col_diff <= 1 {
            let moving_color = chess_board.get_piece_on_square(from).color();

            let piece = chess_board.get_piece_on_square(to);
            if piece.is_empty() {
                return true;
            } else if piece.is_opponent(moving_color) {
                return true;
            } else if piece.is_friend(moving_color) {
                return false;
            } else {
                return false;
            }
        }

        // King can't move more than a square distance
        false
    }

    fn knight_move(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
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
            let moving_color = chess_board.get_piece_on_square(from).color();

            let piece = chess_board.get_piece_on_square(to);
            if piece.is_empty() {
                return true;
            } else if piece.is_opponent(moving_color) {
                return true;
            } else if piece.is_friend(moving_color) {
                return false;
            } else {
                return false;
            }
        }

        // Movement doesn't follow an L-shape
        false
    }

    fn pawn_move(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if col_diff > 1 && row_diff > 2 {
            // Capture can only happen with a square distance,
            // and the first move can be at most two squares.
            return false;
        }

        let piece_dest = chess_board.get_piece_on_square(to);

        let moving_color = chess_board.get_piece_on_square(from).color();
        if col_diff == 1 {
            // The capture can only happen if the destination square has an oponnent piece or
            // if the el passant is valid for that square.
            if piece_dest.is_opponent(moving_color) || (chess_board.en_passant_square() == to) {
                return true;
            } else {
                return false;
            }
        }

        if row_diff == 1 {
            if piece_dest.is_empty() {
                return true;
            } else {
                return false;
            }
        }

        if row_diff == 2 {
            // This move can only be valid if the pawn was at the starting rank
            if moving_color == Color::White && row1 == 2 {
                // We need to check if there isn't a piece blocking the pawn
                if chess_board.get_piece_on_square(from + chess_board.board_width).is_empty()
            && piece_dest.is_empty() {
                    return true;
                }
            }

            if moving_color == Color::Black && row1 == 6 {
                // We need to check if there isn't a piece blocking the pawn
                if chess_board.get_piece_on_square(from - chess_board.board_width).is_empty()
            && piece_dest.is_empty() {
                    return true;
                }
            }
        }

        // We only get here if we got lost from the true path.
        false
    }
}

impl Default for PieceList {
    fn default() -> Self {
        PieceList {
            white_king_list         : Vec::new(),
            white_queen_list        : Vec::new(),
            white_rook_list         : Vec::new(),
            white_light_bishop_list : Vec::new(),
            white_dark_bishop_list  : Vec::new(),
            white_knight_list       : Vec::new(),
            white_pawn_list         : Vec::new(),

            black_king_list         : Vec::new(),
            black_queen_list        : Vec::new(),
            black_rook_list         : Vec::new(),
            black_light_bishop_list : Vec::new(),
            black_dark_bishop_list  : Vec::new(),
            black_knight_list       : Vec::new(),
            black_pawn_list         : Vec::new(),
        }
    }
}