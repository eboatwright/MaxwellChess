use crate::value_holder::ValueHolder;
use crate::castling_rights::*;
use crate::constants::*;
use crate::utils::get_lsb;
use crate::utils::print_bitboard;
use crate::precalculated_data::*;
use crate::flag;
use crate::utils::pop_lsb;
use crate::pieces;
use crate::move_data::MoveData;

pub const ALL_MOVES: bool = false;
pub const CAPTURES_ONLY: bool = true;

#[derive(Copy, Clone)]
pub struct BoardState {
	pub castling_rights: CastlingRights,
	pub en_passant_square: u8,
}

#[derive(Clone)]
pub struct Board {
	pub piece_bitboards: [u64; pieces::COUNT as usize],
	pub color_bitboards: [u64; 2],
	pub white_to_move: bool,

	pub history: ValueHolder<BoardState>,
}

impl Board {
	pub fn new(fen: &'static str) -> Self {
		let fen_split = fen.split(' ').collect::<Vec<&str>>();
		let fen_pieces = fen_split[0].replace("/", "");

		let mut piece_bitboards = [0; pieces::COUNT as usize];
		let mut color_bitboards = [0; 2];
		let mut i = 0;

		for c in fen_pieces.chars() {
			if let Ok(empty_squares) = c.to_string().parse::<usize>() {
				i += empty_squares;
			} else {
				let piece = pieces::from_char(c);
				piece_bitboards[piece as usize] |= 1 << i;
				color_bitboards[pieces::get_color_index(piece)] |= 1 << i;

				i += 1;
			}
		}

		Self {
			piece_bitboards,
			color_bitboards,
			white_to_move: fen_split[1] == "w",

			history: ValueHolder::new(
				BoardState {
					castling_rights: CastlingRights::from_str(fen_split[2]),
					en_passant_square: square_to_index(fen_split[3]),
				}
			),
		}
	}

	pub fn occupied_bitboard(&self) -> u64 {
		self.color_bitboards[0] | self.color_bitboards[1]
	}

	/*
	The idea of this function is from Weiawaga and Tcheran (There's definitely more engines that use it,
	but that's where I saw it), and it's so elegant, but a little confusing so I wanted to explain it:
	what this function does is instead of calculating all the attacks of the enemy pieces, and then
	checking if the target square is in that set, you go backwards. For example: if you're trying to
	detect if a knight is checking the king, you place an "imaginary" knight on the king's square,
	and if that knight attacks any of the opponent's knights, you know that knight is putting you in check!
	*/
	pub fn get_attackers_of(&self, square: u8) -> u64 {
		let mut result = 0;

		let occupied = self.occupied_bitboard();
		let other_color = !self.white_to_move;
		let other_queens = self.piece_bitboards[pieces::build(other_color, pieces::QUEEN) as usize];

		result |=
			PAWN_ATTACKS[square as usize][self.white_to_move as usize]
			& self.piece_bitboards[pieces::build(other_color, pieces::PAWN) as usize];

		result |=
			KNIGHT_ATTACKS[square as usize]
			& self.piece_bitboards[pieces::build(other_color, pieces::KNIGHT) as usize];

		result |=
			get_bishop_moves(square, occupied)
			& (self.piece_bitboards[pieces::build(other_color, pieces::BISHOP) as usize] | other_queens);

		result |=
			get_rook_moves(square, occupied)
			& (self.piece_bitboards[pieces::build(other_color, pieces::ROOK) as usize] | other_queens);

		result |=
			KING_ATTACKS[square as usize]
			& self.piece_bitboards[pieces::build(other_color, pieces::KING) as usize];

		result
	}

	pub fn in_check(&self) -> bool {
		let king_bitboard = self.piece_bitboards[pieces::build(self.white_to_move, pieces::KING) as usize];
		self.get_attackers_of(get_lsb(king_bitboard)) != 0
	}

	pub fn print(&self) {
		for row in 0..8 {
			println!("---------------------------------");
			let mut buffer = "".to_string();
			for col in 0..8 {
				buffer += &format!("| {} ", pieces::to_char(self.get(row * 8 + col)));
			}
			println!("{}|", buffer);
		}
		println!("---------------------------------");
		println!("{} to move", if self.white_to_move { "White" } else { "Black" });
	}

	pub fn get(&self, i: u8) -> u8 {
		for piece in 0..pieces::COUNT {
			if self.piece_bitboards[piece as usize] & (1 << i) != 0 {
				return piece;
			}
		}

		pieces::NONE
	}

	pub fn move_piece(&mut self, data: &MoveData) {
		let is_white = pieces::is_white(data.piece);

		self.piece_bitboards[data.piece as usize] ^= 1 << data.from;

		if data.capture != pieces::NONE {
			self.piece_bitboards[data.capture as usize] ^= 1 << data.to;
			self.color_bitboards[!is_white as usize] ^= 1 << data.to;
		}

		if flag::is_promotion(data.flag) {
			self.piece_bitboards[(data.flag + pieces::get_color_offset(data.piece)) as usize] ^= 1 << data.to;
		} else {
			self.piece_bitboards[data.piece as usize] ^= 1 << data.to;
		}

		self.color_bitboards[is_white as usize] ^= 1 << data.from;
		self.color_bitboards[is_white as usize] ^= 1 << data.to;
	}

	pub fn undo_move_piece(&mut self, data: &MoveData) {
		let is_white = pieces::is_white(data.piece);

		self.piece_bitboards[data.piece as usize] ^= 1 << data.from;

		if data.capture != pieces::NONE {
			self.piece_bitboards[data.capture as usize] ^= 1 << data.to;
			self.color_bitboards[!is_white as usize] ^= 1 << data.to;
		}

		if flag::is_promotion(data.flag) {
			let promotion_piece = data.flag + pieces::get_color_offset(data.piece);

			self.piece_bitboards[promotion_piece as usize] ^= 1 << data.to;
		} else {
			self.piece_bitboards[data.piece as usize] ^= 1 << data.to;
		}

		self.color_bitboards[is_white as usize] ^= 1 << data.from;
		self.color_bitboards[is_white as usize] ^= 1 << data.to;
	}

	pub fn make_move(&mut self, data: &MoveData) -> bool {
		self.move_piece(data);

		let piece_type = pieces::get_type(data.piece);
		let mut current_state = self.history.peek();

		if piece_type == pieces::KING {
			current_state.castling_rights.remove_both(self.white_to_move);
		} else if piece_type == pieces::ROOK {
			current_state.castling_rights.remove_one(data.from);
		}

		if pieces::get_type(data.capture) == pieces::ROOK {
			current_state.castling_rights.remove_one(data.to);
		}

		self.history.push(current_state);

		if self.in_check() {
			self.white_to_move = !self.white_to_move;
			self.undo_move(data);
			return false;
		}

		self.white_to_move = !self.white_to_move;

		true
	}

	pub fn undo_move(&mut self, data: &MoveData) {
		if !self.history.is_empty() {
			self.undo_move_piece(data);

			self.history.pop();
			self.white_to_move = !self.white_to_move;
		}
	}

	pub fn get_moves_for_piece(&self, piece_index: u8, captures_only: bool) -> Vec<MoveData> {
		let mut moves = vec![];
		let piece = self.get(piece_index);
		let piece_type = pieces::get_type(piece);
		let is_white_piece = pieces::is_white(piece);
		let other_color = !is_white_piece as usize;

		if piece_type == pieces::PAWN {
			let rank = piece_index / 8;
			let will_promote = rank == SECOND_RANK[other_color];


			// Pushing
			if !captures_only {
				let single_push = (piece_index as i8 + PAWN_PUSH[is_white_piece as usize]) as u8;

				if self.get(single_push) == pieces::NONE {
					if will_promote {
						for promotion in pieces::KNIGHT..=pieces::QUEEN {
							moves.push(
								MoveData {
									from: piece_index,
									to: single_push,
									piece,
									capture: pieces::NONE,
									flag: promotion,
								}
							);
						}
					} else {
						moves.push(
							MoveData {
								from: piece_index,
								to: single_push,
								piece,
								capture: pieces::NONE,
								flag: flag::NONE,
							}
						);
					}

					if rank == SECOND_RANK[is_white_piece as usize] {
						let double_push = (piece_index as i8 + DOUBLE_PAWN_PUSH[is_white_piece as usize]) as u8;

						if self.get(double_push) == pieces::NONE {
							moves.push(
								MoveData {
									from: piece_index,
									to: double_push,
									piece,
									capture: pieces::NONE,
									flag: flag::DOUBLE_PAWN_PUSH,
								}
							);
						}
					}
				}
			}

			// Capturing
			let mut bitboard =
				  PAWN_ATTACKS[piece_index as usize][is_white_piece as usize]
				& self.color_bitboards[!is_white_piece as usize];

			while bitboard != 0 {
				let capture_index = pop_lsb(&mut bitboard);
				let capture = self.get(capture_index);

				if will_promote {
					for promotion in pieces::KNIGHT..=pieces::QUEEN {
						moves.push(
							MoveData {
								from: piece_index,
								to: capture_index,
								piece,
								capture,
								flag: promotion,
							}
						);
					}
				} else {
					moves.push(
						MoveData {
							from: piece_index,
							to: capture_index,
							piece,
							capture,
							flag: flag::NONE,
						}
					);
				}
			}
		} else {
			let mut bitboard =
				if piece_type == pieces::KNIGHT {
					KNIGHT_ATTACKS[piece_index as usize]
				} else if piece_type == pieces::BISHOP {
					get_bishop_moves(piece_index, self.occupied_bitboard())
				} else if piece_type == pieces::ROOK {
					get_rook_moves(piece_index, self.occupied_bitboard())
				} else if piece_type == pieces::QUEEN {
					get_bishop_moves(piece_index, self.occupied_bitboard()) | get_rook_moves(piece_index, self.occupied_bitboard())
				} else {
					KING_ATTACKS[piece_index as usize]
				}
				& !self.color_bitboards[is_white_piece as usize];

			if captures_only {
				bitboard &= self.color_bitboards[!is_white_piece as usize];
			} else if piece_type == pieces::KING {
				let castling_rights = self.history.peek().castling_rights;

				if castling_rights.kingside[is_white_piece as usize]
				&& CASTLE_KINGSIDE_MASK[is_white_piece as usize] & self.occupied_bitboard() == 0
				&& self.get_attackers_of(piece_index + 1) == 0 {
					moves.push(
						MoveData {
							from: piece_index,
							to: piece_index + 2,
							piece,
							capture: pieces::NONE,
							flag: flag::CASTLE_KINGSIDE,
						}
					);
				} else if castling_rights.queenside[is_white_piece as usize]
				&& CASTLE_QUEENSIDE_MASK[is_white_piece as usize] & self.occupied_bitboard() == 0
				&& self.get_attackers_of(piece_index - 1) == 0 {
					moves.push(
						MoveData {
							from: piece_index,
							to: piece_index - 2,
							piece,
							capture: pieces::NONE,
							flag: flag::CASTLE_QUEENSIDE,
						}
					);
				}
			}

			while bitboard != 0 {
				let move_index = pop_lsb(&mut bitboard);
				let capture = self.get(move_index);

				moves.push(
					MoveData {
						from: piece_index,
						to: move_index,
						piece,
						capture,
						flag: flag::NONE,
					}
				);
			}
		}

		moves
	}

	pub fn get_moves(&self, captures_only: bool) -> Vec<MoveData> {
		let mut moves = vec![];

		let pieces =
			if self.white_to_move {
				pieces::WHITE_PAWN..=pieces::WHITE_KING
			} else {
				pieces::BLACK_PAWN..=pieces::BLACK_KING
			};

		for piece in pieces {
			let mut bitboard = self.piece_bitboards[piece as usize];
			while bitboard != 0 {
				let piece_index = pop_lsb(&mut bitboard);
				moves.append(&mut self.get_moves_for_piece(piece_index, captures_only));
			}
		}

		moves
	}

	pub fn try_move(&mut self, coordinates: &str) -> bool {
		let data = MoveData::from_coordinates(coordinates);
		let piece = self.get(data.from);

		if piece == pieces::NONE
		|| pieces::is_white(piece) != self.white_to_move {
			return false;
		}

		let moves = self.get_moves_for_piece(data.from, ALL_MOVES);
		for m in moves {
			if m.to == data.to
			&& (data.flag == flag::NONE || data.flag == m.flag) {
				return self.make_move(&m);
			}
		}

		false
	}

	pub fn perspective(&self) -> i32 {
		if self.white_to_move { 1 } else { -1 }
	}

	pub fn simple_eval(&self) -> i32 {
		let mut material_balance = 0;

		for piece in 0..pieces::COUNT {
			let mut bitboard = self.piece_bitboards[piece as usize];
			while bitboard != 0 {
				let _ = pop_lsb(&mut bitboard);
				material_balance += pieces::VALUES[pieces::get_type(piece) as usize] * pieces::get_eval_multiplier(piece);
			}
		}

		material_balance * self.perspective()
	}
}