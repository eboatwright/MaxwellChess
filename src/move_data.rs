use crate::constants::{SQUARES, square_to_index};
use crate::pieces;

pub mod flag {
	pub const NONE:             u8 = 0;
	pub const PROMOTE_KNIGHT:   u8 = 1;
	pub const PROMOTE_BISHOP:   u8 = 2;
	pub const PROMOTE_ROOK:     u8 = 3;
	pub const PROMOTE_QUEEN:    u8 = 4;
	pub const DOUBLE_PAWN_PUSH: u8 = 5;
	pub const EN_PASSANT:       u8 = 6;
	pub const CASTLE_SHORT:     u8 = 7;
	pub const CASTLE_LONG:      u8 = 8;

	pub fn is_promotion(flag: u8) -> bool {
		   flag == PROMOTE_KNIGHT
		|| flag == PROMOTE_BISHOP
		|| flag == PROMOTE_ROOK
		|| flag == PROMOTE_QUEEN
	}
}

pub const NULL_MOVE: MoveData = MoveData {
	from: 0,
	to: 0,
	piece: pieces::NONE,
	capture: pieces::NONE,
	flag: flag::NONE,
};

#[derive(Copy, Clone)]
pub struct MoveData {
	pub from: u8,
	pub to: u8,
	pub piece: u8,
	pub capture: u8,
	pub flag: u8,
}

impl Default for MoveData {
	fn default() -> Self {
		Self {
			from: 0,
			to: 0,
			piece: pieces::NONE,
			capture: pieces::NONE,
			flag: flag::NONE,
		}
	}
}

impl MoveData {
	pub fn from_binary(binary: u16) -> Self {
		Self {
			from: ((binary & 0b111111_000000) >> 6) as u8,
			to: (binary & 0b000000_111111) as u8,
			..Default::default()
		}
	}

	pub fn from_coordinates(coordinates: &str) -> Self {
		Self {
			from: square_to_index(&coordinates[0..2]),
			to: square_to_index(&coordinates[2..4]),
			flag:
				if let Some(promotion) = coordinates.chars().nth(4) {
					pieces::from_char(promotion)
				} else {
					flag::NONE
				},
			..Default::default()
		}
	}

	pub fn to_binary(&self) -> u16 {
		((self.from as u16) << 6) | (self.to as u16)
	}

	pub fn to_coordinates(&self) -> String {
		format!("{}{}{}",
			SQUARES[self.from as usize],
			SQUARES[self.to as usize],
			if flag::is_promotion(self.flag) {
				pieces::to_str(self.flag)
			} else {
				""
			},
		)
	}
}