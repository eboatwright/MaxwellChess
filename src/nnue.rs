/*
Current architecture:
768 -> 64 -> 1
*/

use crate::nnue_weights::*;
use crate::move_data::{SHORT_CASTLE_FLAG, LONG_CASTLE_FLAG, EN_PASSANT_FLAG, MoveData};
use crate::pieces::{WHITE_ROOK, BLACK_ROOK, NO_PIECE, PROMOTABLE, build_piece, is_piece_white, char_to_piece};
use crate::Board;
use rand::Rng;

pub const NNUE_EVAL_SCALE: f32 = 100.0; // TODO: I have no idea what this should be

pub struct NNUE {
	pub accumulators: Vec<f32>,

	pub hidden_layer_weights: Vec<f32>,
	pub hidden_layer_biases: Vec<f32>,

	pub output_layer_weights: Vec<f32>,
	pub output_layer_biases: Vec<f32>,
}

impl NNUE {
	pub fn new(
		hidden_layer_weights: Vec<f32>,
		hidden_layer_biases: Vec<f32>,
		output_layer_weights: Vec<f32>,
		output_layer_biases: Vec<f32>,
	) -> Self {
		Self {
			accumulators: hidden_layer_biases.clone(),

			hidden_layer_weights,
			hidden_layer_biases,

			output_layer_weights,
			output_layer_biases,
		}
	}

	pub fn initialize(
		board: &Board,
		
		hidden_layer_weights: Vec<f32>,
		hidden_layer_biases: Vec<f32>,
		output_layer_weights: Vec<f32>,
		output_layer_biases: Vec<f32>,
	) -> Self {
		let mut nnue = NNUE::new(
			hidden_layer_weights,
			hidden_layer_biases,

			output_layer_weights,
			output_layer_biases,
		);

		for i in 0..64 {
			let piece = board.get_piece(i);
			if piece != NO_PIECE {
				nnue.activate(i, piece as u8);
			}
		}

		nnue
	}

	pub fn setup_fen(&mut self, fen: &String) {
		self.accumulators = self.hidden_layer_biases.clone();

		let fen_split = fen.split(' ').collect::<Vec<&str>>();
		let piece_rows = fen_split[0].split('/').collect::<Vec<&str>>();

		let mut i = 0;

		for row in piece_rows {
			for piece in row.chars() {
				if let Ok(empty_squares) = piece.to_string().parse::<usize>() {
					i += empty_squares;
				} else {
					let piece = char_to_piece(piece);
					self.activate(i as u8, piece as u8);

					i += 1;
				}
			}
		}
	}

	pub fn get_index(square: u8, piece: u8) -> usize {
		square as usize * 12 + piece as usize
	}

	pub fn activate(&mut self, square: u8, piece: u8) {
		let length = self.accumulators.len();
		let index = Self::get_index(square, piece);

		for i in 0..length {
			self.accumulators[i] += self.hidden_layer_weights[index * length + i];
		}
	}

	pub fn deactivate(&mut self, square: u8, piece: u8) {
		let length = self.accumulators.len();
		let index = Self::get_index(square, piece);

		for i in 0..length {
			self.accumulators[i] -= self.hidden_layer_weights[index * length + i];
		}
	}

	pub fn make_move(&mut self, data: &MoveData) {
		self.deactivate(data.from, data.piece);

		if PROMOTABLE.contains(&data.flag) {
			let promotion = build_piece(is_piece_white(data.piece as usize), data.flag as usize);
			self.activate(data.to, promotion as u8);
		} else {
			self.activate(data.to, data.piece);

			if data.flag == SHORT_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.deactivate(63, WHITE_ROOK as u8);
					self.activate(61, WHITE_ROOK as u8);
				} else {
					self.deactivate(7, BLACK_ROOK as u8);
					self.activate(5, BLACK_ROOK as u8);
				}
			} else if data.flag == LONG_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.deactivate(56, WHITE_ROOK as u8);
					self.activate(59, WHITE_ROOK as u8);
				} else {
					self.deactivate(0, BLACK_ROOK as u8);
					self.activate(3, BLACK_ROOK as u8);
				}
			}
		}

		if data.capture != NO_PIECE as u8 {
			if data.flag == EN_PASSANT_FLAG {
				let en_passant_square =
					if is_piece_white(data.piece as usize) {
						data.to + 8
					} else {
						data.to - 8
					};

				self.deactivate(en_passant_square, data.capture);
			} else {
				self.deactivate(data.to, data.capture);
			}
		}
	}

	pub fn undo_move(&mut self, data: &MoveData) {
		self.activate(data.from, data.piece);

		if PROMOTABLE.contains(&data.flag) {
			let promotion = build_piece(is_piece_white(data.piece as usize), data.flag as usize);
			self.deactivate(data.to, promotion as u8);
		} else {
			self.deactivate(data.to, data.piece);

			if data.flag == SHORT_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.activate(63, WHITE_ROOK as u8);
					self.deactivate(61, WHITE_ROOK as u8);
				} else {
					self.activate(7, BLACK_ROOK as u8);
					self.deactivate(5, BLACK_ROOK as u8);
				}
			} else if data.flag == LONG_CASTLE_FLAG {
				if is_piece_white(data.piece as usize) {
					self.activate(56, WHITE_ROOK as u8);
					self.deactivate(59, WHITE_ROOK as u8);
				} else {
					self.activate(0, BLACK_ROOK as u8);
					self.deactivate(3, BLACK_ROOK as u8);
				}
			}
		}

		if data.capture != NO_PIECE as u8 {
			if data.flag == EN_PASSANT_FLAG {
				let en_passant_square =
					if is_piece_white(data.piece as usize) {
						data.to + 8
					} else {
						data.to - 8
					};

				self.activate(en_passant_square, data.capture);
			} else {
				self.activate(data.to, data.capture);
			}
		}
	}

	pub fn evaluate(&self) -> f32 { // total_piece_count: usize
		// There are a maximum of 32 pieces on a Chess board,
		// so our max index is: (32 - 1) / 4 = 7.75 which then gets rounded down because we're dividing integers, so 7 which is what we want
		// And there are a minimum of 2 pieces on a Chess board (Both kings)
		// so our min index is: (2 - 1) / 4 = 0.25 which gets rounded down to 0
		// let bucket = (total_piece_count - 1) / 4;
		let mut output = self.output_layer_biases[0];

		// let bucket_offset = bucket * self.accumulators.len();

		for i in 0..self.accumulators.len() {
			output += Self::clipped_relu(self.accumulators[i]) * self.output_layer_weights[i];
		}

		output
	}

	fn clipped_relu(x: f32) -> f32 {
		x.clamp(0.0, 1.0)
	}
}