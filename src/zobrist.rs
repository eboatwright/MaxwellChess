use crate::flag;
use crate::board::{Board, BoardState};
use crate::move_data::MoveData;
use crate::pieces;
use crate::value_holder::ValueHolder;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

pub const SEED: u64 = 19274892;

#[derive(Clone)]
pub struct Zobrist {
	pub key: ValueHolder<u64>,

	pieces: [[u64; pieces::COUNT as usize]; 64],
	castling_rights: [u64; 16],
	en_passant: [u64; 8],
	side_to_move: u64,
}

impl Zobrist {
	pub fn empty() -> Self {
		Self {
			key: ValueHolder::new(0),

			pieces: [[0; pieces::COUNT as usize]; 64],
			castling_rights: [0; 16],
			en_passant: [0; 8],
			side_to_move: 0,
		}
	}

	pub fn calculate(board: &Board) -> Self {
		let mut rng = Pcg64::seed_from_u64(SEED);
		let mut zobrist = Zobrist::empty();
		let mut key = 0;
		let board_state = board.history.peek();

		for square in 0..64 {
			for piece in 0..pieces::COUNT as usize {
				zobrist.pieces[square][piece] = rng.gen::<u64>();
			}

			let piece = board.get(square as u8);
			if piece != pieces::NONE {
				key ^= zobrist.pieces[square][piece as usize];
			}
		}

		for i in 0..16 {
			zobrist.castling_rights[i] = rng.gen::<u64>();
		}

		key ^= zobrist.castling_rights[board_state.castling_rights.0 as usize];

		for i in 0..8 {
			zobrist.en_passant[i] = rng.gen::<u64>();
		}

		if board_state.en_passant_square != 0 {
			key ^= zobrist.en_passant[board_state.en_passant_square as usize % 8];
		}

		zobrist.side_to_move = rng.gen::<u64>();

		if board.white_to_move {
			key ^= zobrist.side_to_move;
		}

		*zobrist.key.peek_mut() = key;
		zobrist
	}

	pub fn make_move(&mut self, m: &MoveData, previous_state: &BoardState, current_state: &BoardState) {
		let mut new_key = self.key.peek();

		new_key ^= self.pieces[m.from as usize][m.piece as usize];

		if flag::is_promotion(m.flag) {
			new_key ^= self.pieces[m.to as usize][pieces::build(pieces::is_white(m.piece), m.flag) as usize];
		} else {
			new_key ^= self.pieces[m.to as usize][m.piece as usize];

			if m.flag == flag::CASTLE_KINGSIDE {
				let rook = pieces::build(pieces::is_white(m.piece), pieces::ROOK) as usize;
				new_key ^= self.pieces[m.to as usize + 1][rook];
				new_key ^= self.pieces[m.to as usize - 1][rook];
			} else if m.flag == flag::CASTLE_QUEENSIDE {
				let rook = pieces::build(pieces::is_white(m.piece), pieces::ROOK) as usize;
				new_key ^= self.pieces[m.to as usize - 2][rook];
				new_key ^= self.pieces[m.to as usize + 1][rook];
			}
		}

		new_key ^= previous_state.castling_rights.0 as u64;
		new_key ^= current_state.castling_rights.0 as u64;

		if previous_state.en_passant_square != 0 {
			new_key ^= (previous_state.en_passant_square % 8) as u64;
		}

		if current_state.en_passant_square != 0 {
			new_key ^= (current_state.en_passant_square % 8) as u64;
		}

		new_key ^= self.side_to_move;

		self.key.push(new_key);
	}

	// pub fn undo_move(&mut self, m: &MoveData) {
	// 	// TODO is it faster to incrementally undo, or use history?
	// 	self.key.pop();
	// }
}