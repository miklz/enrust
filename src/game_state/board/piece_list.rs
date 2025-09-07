use crate::game_state::board::Color;
use crate::game_state::board::Piece;
use crate::game_state::board::Move;
use crate::game_state::board::ChessBoard;

#[derive(Clone)]
pub struct PieceList {
    white_king_list     : Vec<i16>,
    white_queen_list    : Vec<i16>,
    white_rook_list     : Vec<i16>,
    white_bishop_list   : Vec<i16>,
    white_knight_list   : Vec<i16>,
    white_pawn_list     : Vec<i16>,

    black_king_list     : Vec<i16>,
    black_queen_list    : Vec<i16>,
    black_rook_list     : Vec<i16>,
    black_bishop_list   : Vec<i16>,
    black_knight_list   : Vec<i16>,
    black_pawn_list     : Vec<i16>,
}

impl PieceList {
    fn is_king_safe(&mut self, chess_board: &mut ChessBoard, color: Color, mv: &Move) -> bool {
        chess_board.make_move(&mv);

        match color {
            Color::White => {
                if let Some(&white_king) = self.white_king_list.get(0) {

                    for black_queen in self.black_queen_list.iter_mut() {
                        if Self::queen_move(chess_board, *black_queen, white_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for black_rook in self.black_rook_list.iter_mut() {
                        if Self::rook_move(chess_board, *black_rook, white_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for black_bishop in self.black_bishop_list.iter_mut() {
                        if Self::bishop_move(chess_board, *black_bishop, white_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for black_knight in self.black_knight_list.iter_mut() {
                        if Self::knight_move(chess_board, *black_knight, white_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for black_pawn in self.black_pawn_list.iter_mut() {
                        if Self::pawn_move(chess_board, *black_pawn, white_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for black_king in self.black_king_list.iter_mut() {
                        if Self::king_move(chess_board, *black_king, white_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }
                }
            },
            Color::Black => {
                if let Some(&black_king) = self.black_king_list.get(0) {

                    for white_queen in self.white_queen_list.iter_mut() {
                        if Self::queen_move(chess_board, *white_queen, black_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for white_rook in self.white_rook_list.iter_mut() {
                        if Self::rook_move(chess_board, *white_rook, black_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for white_bishop in self.white_bishop_list.iter_mut() {
                        if Self::bishop_move(chess_board, *white_bishop, black_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for white_knight in self.white_knight_list.iter_mut() {
                        if Self::knight_move(chess_board, *white_knight, black_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for white_pawn in self.white_pawn_list.iter_mut() {
                        if Self::pawn_move(chess_board, *white_pawn, black_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }

                    for white_king in self.white_king_list.iter_mut() {
                        if Self::king_move(chess_board, *white_king, black_king) {
                            chess_board.unmake_move(&mv);
                            return false
                        }
                    }
                }
            },
        }

        chess_board.unmake_move(&mv);
        println!("King is not in check");

        true
    }

    pub fn generate_legal_moves(&mut self, chess_board: &mut ChessBoard, color: Color) -> Vec<Move> {
        let mut all_moves = self.generate_moves(chess_board, color);

        all_moves.retain(|mv| self.is_king_safe(chess_board, color, &mv));
        all_moves
    }

    fn generate_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut all_moves = self.generate_king_moves(chess_board, color);
        all_moves.append(&mut self.generate_queen_moves(chess_board, color));
        all_moves.append(&mut self.generate_rook_moves(chess_board, color));
        all_moves.append(&mut self.generate_bishop_moves(chess_board, color));
        all_moves.append(&mut self.generate_knight_moves(chess_board, color));
        all_moves.append(&mut self.generate_pawn_moves(chess_board, color));

        all_moves
    }

    fn generate_king_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let (king, king_list) = match color {
            Color::White => (Piece::WhiteKing, &self.white_king_list),
            Color::Black => (Piece::BlackKing, &self.black_king_list)
        };

        let king_rays: [i16; 8] = [-1, 1, chess_board.board_width, -chess_board.board_width,
                        chess_board.board_width + 1, -chess_board.board_width + 1,
                        chess_board.board_width - 1, -chess_board.board_width - 1];

        for &square in king_list {
            for ray in king_rays {
                let target = chess_board.get_piece_on_square(square + ray);
                if target.is_empty() || target.is_opponent(color) {
                    moves.push(chess_board.create_move(
                        square,
                        square + ray,
                        king,
                        target
                    ));
                }
            }
        }

        moves
    }

    fn generate_queen_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let (queen, queen_list) = match color {
            Color::White => (Piece::WhiteQueen, &self.white_queen_list),
            Color::Black => (Piece::BlackQueen, &self.black_queen_list)
        };

        let queen_rays: [i16; 8] = [-1, 1, 8, -chess_board.board_width,
                        chess_board.board_width + 1, -chess_board.board_width + 1,
                        chess_board.board_width - 1, -chess_board.board_width - 1];

        for &square in queen_list {
            for ray in queen_rays {
                let mut position = square;
                loop {
                    position += ray;

                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() || target.is_opponent(color) {
                        moves.push(chess_board.create_move(
                            square,
                            position,
                            queen,
                            target,
                        ));
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }
                }
            }
        }

        moves
    }

    fn generate_rook_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let (rook, rook_list) = match color {
            Color::White => (Piece::WhiteRook, &self.white_rook_list),
            Color::Black => (Piece::BlackRook, &self.black_rook_list)
        };

        let rook_rays: [i16; 4] = [1, -1, -chess_board.board_width, chess_board.board_width];

        for &square in rook_list {
            for ray in rook_rays {
                let mut position = square;
                loop {
                    position += ray;

                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() || target.is_opponent(color) {
                        moves.push(chess_board.create_move(
                            square,
                            position,
                            rook,
                            target
                        ));
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }
                }
            }
        }

        moves
    }

    fn generate_bishop_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let (bishop, bishop_list) = match color {
            Color::White => (Piece::WhiteBishop, &self.white_bishop_list),
            Color::Black => (Piece::BlackBishop, &self.black_bishop_list)
        };

        let bishop_rays: [i16; 4] = [chess_board.board_width + 1, chess_board.board_width - 1,
                            -chess_board.board_width + 1, -chess_board.board_width - 1];

        for &square in bishop_list {
            for ray in bishop_rays {
                let mut position = square;
                loop {
                    position += ray;
                    
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() || target.is_opponent(color) {
                        moves.push(chess_board.create_move(
                            square,
                            position,
                            bishop,
                            target
                        ));
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }
                }
            }
        }

        moves
    }

    fn generate_knight_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let (knight, knight_list) = match color {
            Color::White => (Piece::WhiteKnight, &self.white_knight_list),
            Color::Black => (Piece::BlackKnight, &self.black_knight_list)
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
            for ray in knight_rays {
                let target = chess_board.get_piece_on_square(square + ray);
                if target.is_empty() || target.is_opponent(color) {
                    moves.push(chess_board.create_move(
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

    fn generate_pawn_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
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
            Color::White => [Piece::WhiteQueen, Piece::WhiteRook, Piece::WhiteBishop, Piece::WhiteKnight],
            Color::Black => [Piece::BlackQueen, Piece::BlackRook, Piece::BlackBishop, Piece::BlackKnight],
        };

        let promotion_rank = match color {
            Color::White => 8,
            Color::Black => 1,
        };

        for &square in pawn_list {
            let target = chess_board.get_piece_on_square(square + direction);
            if target.is_empty() {
                if square + direction != promotion_rank {
                    moves.push(chess_board.create_pawn_move(
                        square,
                        square + direction,
                        pawn,
                        target,
                        None,
                        false,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(chess_board.create_pawn_move(
                            square,
                            square + direction,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                        ));
                    }
                }
            }

            let target = chess_board.get_piece_on_square(square + direction + 1);
            if target.is_opponent(color) {
                if square + direction + 1 != promotion_rank {
                    moves.push(chess_board.create_pawn_move(
                        square,
                        square + direction + 1,
                        pawn,
                        target,
                        None,
                        false,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(chess_board.create_pawn_move(
                            square,
                            square + direction + 1,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                        ));
                    }
                }
            } else if Some(square + direction + 1) == chess_board.get_en_passant_square() {
                moves.push(chess_board.create_pawn_move(
                    square,
                    square + direction + 1,
                    pawn,
                    target,
                    None,
                    false,
                ));
            }

            let target = chess_board.get_piece_on_square(square + direction - 1);
            if target.is_opponent(color) {
                if square + direction - 1 != promotion_rank {
                    moves.push(chess_board.create_pawn_move(
                        square,
                        square + direction - 1,
                        pawn,
                        target,
                        None,
                        false,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(chess_board.create_pawn_move(
                            square,
                            square + direction - 1,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                        ));
                    }
                }
            }  else if Some(square + direction - 1) == chess_board.get_en_passant_square() {
                moves.push(chess_board.create_pawn_move(
                    square,
                    square + direction - 1,
                    pawn,
                    target,
                    None,
                    true,
                ));
            }

            let target = chess_board.get_piece_on_square(square + 2 * direction);
            if (color == Color::White) && (chess_board.square_rank(square) == 2) {
                if target.is_empty() {
                    moves.push(chess_board.create_pawn_move(
                        square,
                        square + 2 * direction,
                        pawn,
                        target,
                        None,
                        false,
                    ));
                }
            }

            if (color == Color::Black) && (chess_board.square_rank(square) == 7) {
                if target.is_empty() {
                    moves.push(chess_board.create_pawn_move(
                        square,
                        square + 2 * direction,
                        pawn,
                        target,
                        None,
                        false,
                    ));
                }
            }
        }

        moves
    }

    pub fn update_lists(&mut self, board_position : &[Piece; 120]) {
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
                Piece::WhitePawn    => self.white_pawn_list.push(i16_square),
                Piece::WhiteRook    => self.white_rook_list.push(i16_square),
                Piece::WhiteKnight  => self.white_knight_list.push(i16_square),
                Piece::WhiteBishop  => self.white_bishop_list.push(i16_square),
                Piece::WhiteQueen   => self.white_queen_list.push(i16_square),
                Piece::WhiteKing    => self.white_king_list.push(i16_square),
                Piece::BlackPawn    => self.black_pawn_list.push(i16_square),
                Piece::BlackRook    => self.black_rook_list.push(i16_square),
                Piece::BlackKnight  => self.black_knight_list.push(i16_square),
                Piece::BlackBishop  => self.black_bishop_list.push(i16_square),
                Piece::BlackQueen   => self.black_queen_list.push(i16_square),
                Piece::BlackKing    => self.black_king_list.push(i16_square),
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
            let capture_square = if mv.piece.get_color() == Color::White {
                mv.to - 10 // Todo: think of a better way to pass the board width
            } else { 
                mv.to + 10
            };
            let captured_pawn = if mv.piece.get_color() == Color::White {
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

        if mv.captured_piece != Piece::EmptySquare && mv.captured_piece != Piece::SentinelSquare {
            self.add_piece(mv.captured_piece, mv.to);
        }

        if mv.en_passant {
            let capture_square = if mv.piece.get_color() == Color::White {
                mv.to - 10 // Todo: think of a better way to pass the board width
            } else { 
                mv.to + 10
            };
            let captured_pawn = if mv.piece.get_color() == Color::White {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.add_piece(captured_pawn, capture_square);
        }
        
        // Add the piece to its new location (or promoted piece)
        let final_piece = mv.promotion.unwrap_or(mv.piece);

        // Move the piece
        self.remove_piece(final_piece, mv.to);
        self.add_piece(mv.piece, mv.from);
        
        // Handle castling
        if let Some(castling) = &mv.castling {
            self.remove_piece(castling.rook_piece, castling.rook_to);
            self.add_piece(castling.rook_piece, castling.rook_from);
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
            let squares: Vec<String> = list.iter()
                .map(|&sq| format!("{}", sq))
                .collect();
            println!("{:20}: {}", name, squares.join(" "));
        }
        
        print_list("White Kings", &self.white_king_list);
        print_list("White Queens", &self.white_queen_list);
        print_list("White Rooks", &self.white_rook_list);
        print_list("White Light Bishops", &self.white_bishop_list);
        print_list("White Knights", &self.white_knight_list);
        print_list("White Pawns", &self.white_pawn_list);
        
        print_list("Black Kings", &self.black_king_list);
        print_list("Black Queens", &self.black_queen_list);
        print_list("Black Rooks", &self.black_rook_list);
        print_list("Black Light Bishops", &self.black_bishop_list);
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
    
    fn remove_piece(&mut self, piece: Piece, square: i16) {
        let list = self.get_list_mut(piece);
        if let Some(list) = list {
            if let Some(index) = list.iter().position(|&s| s == square) {
                list.remove(index);
            }
        }
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


        let moving_color = chess_board.get_piece_on_square(from).get_color();
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

        let moving_color = chess_board.get_piece_on_square(from).get_color();
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
            let moving_color = chess_board.get_piece_on_square(from).get_color();

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
            let moving_color = chess_board.get_piece_on_square(from).get_color();

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

        let moving_color = chess_board.get_piece_on_square(from).get_color();
        if col_diff == 1 {
            // The capture can only happen if the destination square has an oponnent piece or
            // if the el passant is valid for that square.
            if let Some(en_passant) = chess_board.get_en_passant_square() {
                if piece_dest.is_opponent(moving_color) || (en_passant == to) {
                    return true;
                } else {
                    return false;
                }
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
            white_king_list     : Vec::new(),
            white_queen_list    : Vec::new(),
            white_rook_list     : Vec::new(),
            white_bishop_list   : Vec::new(),
            white_knight_list   : Vec::new(),
            white_pawn_list     : Vec::new(),

            black_king_list     : Vec::new(),
            black_queen_list    : Vec::new(),
            black_rook_list     : Vec::new(),
            black_bishop_list   : Vec::new(),
            black_knight_list   : Vec::new(),
            black_pawn_list     : Vec::new(),
        }
    }
}