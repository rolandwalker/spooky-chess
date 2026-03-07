use arrayvec::ArrayVec;

use crate::bitboard::{nw_for_board, Bitboard};
use crate::color::Color;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use std::fmt;
use std::hash::{Hash, Hasher};

pub const STANDARD_COLS: usize = 8;
pub const STANDARD_ROWS: usize = 8;

#[derive(Clone, Debug)]
pub struct Board<const NW: usize> {
    pawns: Bitboard<NW>,
    knights: Bitboard<NW>,
    bishops: Bitboard<NW>,
    rooks: Bitboard<NW>,
    queens: Bitboard<NW>,
    kings: Bitboard<NW>,
    white: Bitboard<NW>,
    black: Bitboard<NW>,
    width: usize,
    height: usize,
}

#[hotpath::measure_all]
impl<const NW: usize> PartialEq for Board<NW> {
    fn eq(&self, other: &Self) -> bool {
        self.pawns == other.pawns
            && self.knights == other.knights
            && self.bishops == other.bishops
            && self.rooks == other.rooks
            && self.queens == other.queens
            && self.kings == other.kings
            && self.white == other.white
            && self.black == other.black
            && self.width == other.width
            && self.height == other.height
    }
}

impl<const NW: usize> Eq for Board<NW> {}

#[hotpath::measure_all]
impl<const NW: usize> Hash for Board<NW> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pawns.hash(state);
        self.knights.hash(state);
        self.bishops.hash(state);
        self.rooks.hash(state);
        self.queens.hash(state);
        self.kings.hash(state);
        self.white.hash(state);
        self.black.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

#[hotpath::measure_all]
impl<const NW: usize> Board<NW> {
    pub fn new(width: usize, height: usize, fen: &str) -> Result<Self, String> {
        let mut board = Self::empty(width, height);
        board.load_fen(fen)?;
        Ok(board)
    }

    pub fn empty(width: usize, height: usize) -> Self {
        Board {
            pawns: Bitboard::empty(),
            knights: Bitboard::empty(),
            bishops: Bitboard::empty(),
            rooks: Bitboard::empty(),
            queens: Bitboard::empty(),
            kings: Bitboard::empty(),
            white: Bitboard::empty(),
            black: Bitboard::empty(),
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn index(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    #[inline]
    pub fn occupied(&self) -> Bitboard<NW> {
        self.white | self.black
    }

    #[inline]
    pub fn color_bb(&self, color: Color) -> Bitboard<NW> {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    #[inline]
    pub fn piece_type_bb(&self, pt: PieceType) -> Bitboard<NW> {
        match pt {
            PieceType::Pawn => self.pawns,
            PieceType::Knight => self.knights,
            PieceType::Bishop => self.bishops,
            PieceType::Rook => self.rooks,
            PieceType::Queen => self.queens,
            PieceType::King => self.kings,
        }
    }

    #[inline]
    fn piece_type_bb_mut(&mut self, pt: PieceType) -> &mut Bitboard<NW> {
        match pt {
            PieceType::Pawn => &mut self.pawns,
            PieceType::Knight => &mut self.knights,
            PieceType::Bishop => &mut self.bishops,
            PieceType::Rook => &mut self.rooks,
            PieceType::Queen => &mut self.queens,
            PieceType::King => &mut self.kings,
        }
    }

    #[inline]
    fn color_bb_mut(&mut self, color: Color) -> &mut Bitboard<NW> {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    #[inline]
    pub fn piece_type_at(&self, index: usize) -> Option<PieceType> {
        // Branchless: extract one bit from each piece-type bitboard in parallel,
        // combine into a 6-bit key, and do a single table lookup.
        let key = self.pawns.bit_at(index)
            | (self.knights.bit_at(index) << 1)
            | (self.bishops.bit_at(index) << 2)
            | (self.rooks.bit_at(index) << 3)
            | (self.queens.bit_at(index) << 4)
            | (self.kings.bit_at(index) << 5);

        // Only keys with a single bit set (or 0) are valid on a correct board.
        const TABLE: [Option<PieceType>; 64] = {
            let mut t: [Option<PieceType>; 64] = [None; 64];
            t[1] = Some(PieceType::Pawn);
            t[2] = Some(PieceType::Knight);
            t[4] = Some(PieceType::Bishop);
            t[8] = Some(PieceType::Rook);
            t[16] = Some(PieceType::Queen);
            t[32] = Some(PieceType::King);
            t
        };

        TABLE[key as usize]
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Piece> {
        if !pos.is_valid(self.width, self.height) {
            return None;
        }
        let idx = self.index(pos.col, pos.row);
        let pt = self.piece_type_at(idx)?;
        let color = if self.white.get(idx) {
            Color::White
        } else {
            Color::Black
        };
        Some(Piece::new(pt, color))
    }

    pub fn set_piece(&mut self, pos: &Position, piece: Option<Piece>) {
        if !pos.is_valid(self.width, self.height) {
            return;
        }
        let idx = self.index(pos.col, pos.row);

        // Clear existing piece at this index
        self.pawns.clear(idx);
        self.knights.clear(idx);
        self.bishops.clear(idx);
        self.rooks.clear(idx);
        self.queens.clear(idx);
        self.kings.clear(idx);
        self.white.clear(idx);
        self.black.clear(idx);

        if let Some(p) = piece {
            self.piece_type_bb_mut(p.piece_type).set(idx);
            self.color_bb_mut(p.color).set(idx);
        }
    }

    /// Remove a known piece from the board. Caller must guarantee `piece` matches what's at `pos`.
    #[inline]
    pub fn remove_piece(&mut self, pos: &Position, piece: &Piece) {
        let idx = self.index(pos.col, pos.row);
        self.piece_type_bb_mut(piece.piece_type).clear(idx);
        self.color_bb_mut(piece.color).clear(idx);
    }

    /// Place a piece on the board. The target square must be empty.
    #[inline]
    pub fn place_piece(&mut self, pos: &Position, piece: &Piece) {
        let idx = self.index(pos.col, pos.row);
        self.piece_type_bb_mut(piece.piece_type).set(idx);
        self.color_bb_mut(piece.color).set(idx);
    }

    pub fn clear(&mut self) {
        self.pawns = Bitboard::empty();
        self.knights = Bitboard::empty();
        self.bishops = Bitboard::empty();
        self.rooks = Bitboard::empty();
        self.queens = Bitboard::empty();
        self.kings = Bitboard::empty();
        self.white = Bitboard::empty();
        self.black = Bitboard::empty();
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for row in (0..self.height).rev() {
            let mut empty_count = 0;

            for col in 0..self.width {
                let pos = Position::new(col, row);
                if let Some(piece) = self.get_piece(&pos) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(piece.to_char());
                } else {
                    empty_count += 1;
                }
            }

            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            if row > 0 {
                fen.push('/');
            }
        }

        fen
    }

    fn load_fen(&mut self, fen: &str) -> Result<(), String> {
        self.clear();

        let parts: ArrayVec<&str, 32> = fen.split('/').collect();

        if parts.len() != self.height {
            return Err(format!(
                "Invalid FEN: expected {} rows, got {}",
                self.height,
                parts.len()
            ));
        }

        for (row_idx, row_str) in parts.iter().enumerate() {
            let row = self.height - 1 - row_idx;
            let mut col = 0;
            let mut chars = row_str.chars().peekable();

            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    // Collect all consecutive digits to handle multi-digit numbers
                    let mut num_str = String::new();
                    num_str.push(c);

                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_ascii_digit() {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    let skip = num_str
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid FEN number: {}", num_str))?;
                    col += skip;
                } else if let Some(piece) = Piece::from_char(c) {
                    if col >= self.width {
                        return Err("Invalid FEN: col index out of bounds".to_string());
                    }
                    self.set_piece(&Position::new(col, row), Some(piece));
                    col += 1;
                } else {
                    return Err(format!("Invalid FEN character: {}", c));
                }
            }

            if col != self.width {
                return Err(format!(
                    "Invalid FEN: row {} has wrong number of squares",
                    row
                ));
            }
        }

        Ok(())
    }

    pub fn pieces(&self, color: Color) -> Vec<(Position, Piece)> {
        self.pieces_iter(color).collect()
    }

    #[inline]
    pub fn pieces_iter(&self, color: Color) -> PieceIterator<'_, NW> {
        PieceIterator {
            board: self,
            color,
            bit_iter: self.color_bb(color).iter_ones(),
        }
    }

    pub fn find_king(&self, color: Color) -> Option<Position> {
        let king_bb = self.kings & self.color_bb(color);
        king_bb
            .lowest_bit_index()
            .map(|idx| Position::from_index(idx, self.width))
    }
}

pub struct PieceIterator<'a, const NW: usize> {
    board: &'a Board<NW>,
    color: Color,
    bit_iter: crate::bitboard::BitIterator<NW>,
}

impl<'a, const NW: usize> Iterator for PieceIterator<'a, NW> {
    type Item = (Position, Piece);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.bit_iter.next()?;
        let pt = self.board.piece_type_at(idx).unwrap();
        let pos = Position::from_index(idx, self.board.width);
        Some((pos, Piece::new(pt, self.color)))
    }
}

#[hotpath::measure_all]
impl Board<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }> {
    pub fn standard() -> Self {
        Self::new(
            STANDARD_COLS,
            STANDARD_ROWS,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
        )
        .expect("Failed to create standard board")
    }
}

#[hotpath::measure_all]
impl<const NW: usize> Default for Board<NW> {
    fn default() -> Self {
        Self::empty(STANDARD_COLS, STANDARD_ROWS)
    }
}

#[hotpath::measure_all]
impl<const NW: usize> fmt::Display for Board<NW> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in (0..self.height).rev() {
            write!(f, "{:2} ", row + 1)?;
            for col in 0..self.width {
                let pos = Position::new(col, row);
                if let Some(piece) = self.get_piece(&pos) {
                    write!(f, "{} ", piece.to_char())?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }

        write!(f, "   ")?;

        for col in 0..self.width {
            if col < 26 {
                write!(f, "{} ", (b'a' + col as u8) as char)?;
            } else {
                write!(f, "{} ", col)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type StdBoard = Board<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }>;

    #[test]
    fn test_custom_board_creation() {
        let board: Board<{ nw_for_board(6, 6) }> =
            Board::new(6, 6, "rnbqk1/pppppp/6/6/PPPPPP/RNBQK1")
                .expect("Failed to create custom board");
        assert_eq!(board.width(), 6);
        assert_eq!(board.height(), 6);
    }

    #[test]
    fn test_custom_board_creation_invalid() {
        let board: Result<Board<{ nw_for_board(6, 6) }>, _> =
            Board::new(6, 6, "rnbqk1/pppppp/1/6/PPPPPP/RNBQK1");
        assert!(board.is_err(), "Expected error for invalid FEN");
    }

    #[test]
    fn test_board_piece_placement() {
        let mut board = StdBoard::empty(8, 8);
        let king = Piece::new(PieceType::King, Color::White);
        let pos = Position::new(4, 0);

        board.set_piece(&pos, Some(king));
        assert_eq!(board.get_piece(&pos), Some(king));

        board.set_piece(&pos, None);
        assert_eq!(board.get_piece(&pos), None);
    }

    #[test]
    fn test_board_standard_position() {
        let board = StdBoard::standard();

        // Check white pieces
        assert_eq!(
            board
                .get_piece(&Position::new(0, 0))
                .expect("Expected piece at position (0,0)")
                .piece_type,
            PieceType::Rook
        );
        assert_eq!(
            board
                .get_piece(&Position::new(0, 0))
                .expect("Expected piece at position (0,0)")
                .color,
            Color::White
        );
        assert_eq!(
            board
                .get_piece(&Position::new(4, 0))
                .expect("Expected piece at position (4,0)")
                .piece_type,
            PieceType::King
        );

        // Check white pawns
        for col in 0..8 {
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 1))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},1)", col))
                    .piece_type,
                PieceType::Pawn
            );
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 1))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},1)", col))
                    .color,
                Color::White
            );
        }

        // Check black pieces
        assert_eq!(
            board
                .get_piece(&Position::new(0, 7))
                .expect("Expected piece at position (0,7)")
                .piece_type,
            PieceType::Rook
        );
        assert_eq!(
            board
                .get_piece(&Position::new(0, 7))
                .expect("Expected piece at position (0,7)")
                .color,
            Color::Black
        );
        assert_eq!(
            board
                .get_piece(&Position::new(4, 7))
                .expect("Expected piece at position (4,7)")
                .piece_type,
            PieceType::King
        );

        // Check black pawns
        for col in 0..8 {
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 6))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},6)", col))
                    .piece_type,
                PieceType::Pawn
            );
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 6))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},6)", col))
                    .color,
                Color::Black
            );
        }
    }

    #[test]
    fn test_board_fen_conversion() {
        let board = StdBoard::standard();

        let fen = board.to_fen();
        assert_eq!(fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");

        let new_board = StdBoard::new(8, 8, &fen).expect("Failed to parse FEN string");

        // Verify the boards are identical
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(col, row);
                assert_eq!(board.get_piece(&pos), new_board.get_piece(&pos));
            }
        }
    }
}
