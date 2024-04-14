// 768 -> 256 -> 1

use crate::Board;
use crate::pieces;
use crate::move_data::MoveData;

pub const HIDDEN_LAYER_SIZE: usize = 256;

const HIDDEN_LAYER_WEIGHTS: &[f32] = &[];
const HIDDEN_LAYER_BIASES: &[f32] = &[];
const OUTPUT_LAYER_WEIGHTS: &[f32] = &[];
const OUTPUT_LAYER_BIASES: &[f32] = &[];

#[derive(Clone, Default)]
pub struct NNUE {
	pub accumulators: Vec<f32>,
}

impl NNUE {
	pub fn initialize(board: &Board) -> Self {
		let mut nnue = Self {
			accumulators: HIDDEN_LAYER_BIASES.to_vec(),
		};

		for i in 0..64 {
			let piece = board.get(i);
			if piece != pieces::NONE {
				// TODO is this right?
				let input_index = (i * 12 + piece) as usize;
				for i in 0..HIDDEN_LAYER_SIZE {
					nnue.accumulators[input_index] += HIDDEN_LAYER_WEIGHTS[input_index + i];
				}
			}
		}

		nnue
	}

	pub fn make_move(&mut self, m: &MoveData) {
		// TODO
	}

	pub fn undo_move(&mut self, m: &MoveData) {
		// TODO
	}

	pub fn eval(&self) -> i16 {
		let mut eval = OUTPUT_LAYER_BIASES[0];

		for i in 0..HIDDEN_LAYER_SIZE {
			eval += self.accumulators[i] + OUTPUT_LAYER_WEIGHTS[i];
		}

		eval as i16 // TODO do the whole conversion / scaling ting :P
	}
}