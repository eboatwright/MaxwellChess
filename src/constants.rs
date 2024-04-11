use std::ops::Range;

pub const CHECKMATE: i32 = 100_000;

pub const A_FILE: u64 = 0x_8080808080808080;
pub const B_FILE: u64 = A_FILE >> 1;
pub const C_FILE: u64 = A_FILE >> 2;
pub const D_FILE: u64 = A_FILE >> 3;
pub const E_FILE: u64 = A_FILE >> 4;
pub const F_FILE: u64 = A_FILE >> 5;
pub const G_FILE: u64 = A_FILE >> 6;
pub const H_FILE: u64 = A_FILE >> 7;

pub const NOT_A_FILE: u64 = u64::MAX ^ A_FILE;
pub const NOT_AB_FILES: u64 = NOT_A_FILE ^ B_FILE;

pub const NOT_H_FILE: u64 = u64::MAX ^ H_FILE;
pub const NOT_GH_FILES: u64 = NOT_H_FILE ^ G_FILE;

pub const NO:   i8 = -8;
pub const EA:   i8 =  1;
pub const SO:   i8 =  8;
pub const WE:   i8 = -1;
pub const NOEA: i8 = -7;
pub const SOWE: i8 =  7;
pub const SOEA: i8 =  9;
pub const NOWE: i8 = -9;

pub const ROOK_DIRECTIONS: Range<usize>   = 0..4;
pub const BISHOP_DIRECTIONS: Range<usize> = 4..8;
pub const QUEEN_DIRECTIONS: Range<usize>  = 0..8;

pub const DIRECTION_OFFSETS: [i8; 8] = [NO, EA, SO, WE, NOEA, SOEA, SOWE, NOWE];

pub const SECOND_RANK: [u8; 2] = [1, 6];
pub const PAWN_PUSH: [i8; 2] = [SO, NO];
pub const DOUBLE_PAWN_PUSH: [i8; 2] = [SO * 2, NO * 2];

pub const SQUARES: [&str; 64] = [
	"a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
	"a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
	"a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
	"a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
	"a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
	"a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
	"a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
	"a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
];

// #[derive(Copy, Clone, PartialEq)]
// pub enum Square {
// 	A8, B8, C8, D8, E8, F8, G8, H8,
// 	A7, B7, C7, D7, E7, F7, G7, H7,
// 	A6, B6, C6, D6, E6, F6, G6, H6,
// 	A5, B5, C5, D5, E5, F5, G5, H5,
// 	A4, B4, C4, D4, E4, F4, G4, H4,
// 	A3, B3, C3, D3, E3, F3, G3, H3,
// 	A2, B2, C2, D2, E2, F2, G2, H2,
// 	A1, B1, C1, D1, E1, F1, G1, H1,
// }

pub fn square_to_index(square: &str) -> u8 {
	for i in 0..64 {
		if SQUARES[i] == square {
			return i as u8;
		}
	}

	0
}