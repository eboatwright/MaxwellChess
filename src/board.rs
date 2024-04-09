use crate::utils::print_bitboard;
use crate::precalculated_data::*;
use crate::flag;
use crate::utils::pop_lsb;
use crate::pieces;
use crate::move_data::MoveData;

pub const ALL_MOVES: bool = false;
pub const CAPTURES_ONLY: bool = true;

#[derive(Clone)]
pub struct Board {
	pub piece_bitboards: [u64; pieces::COUNT as usize],
	pub color_bitboards: [u64; 2],
	pub attacked_squares: [u64; 2],
	pub white_to_move: bool,
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
			attacked_squares: [0; 2],
			white_to_move: fen_split[1] == "w",
		}
	}

	pub fn occupied_bitboard(&self) -> u64 {
		self.color_bitboards[0] | self.color_bitboards[1]
	}

	pub fn calculate_attacked_squares(&mut self) {
		let other_color = !self.white_to_move as usize;
		if self.attacked_squares[other_color] == 0 {
			let mut result = 0;

			let pieces =
				if !self.white_to_move {
					pieces::WHITE_PAWN..=pieces::WHITE_KING
				} else {
					pieces::BLACK_PAWN..=pieces::BLACK_KING
				};

			for piece in pieces {
				let mut bitboard = self.piece_bitboards[piece as usize];
				while bitboard != 0 {
					let piece_index = pop_lsb(&mut bitboard);
					let piece_type = pieces::get_type(piece);

					result |=
						if piece_type == pieces::PAWN {
							PAWN_ATTACKS[piece_index as usize][other_color]
						} else if piece_type == pieces::KNIGHT {
							KNIGHT_ATTACKS[piece_index as usize]
						} else if piece_type == pieces::BISHOP {
							get_bishop_moves(piece_index, self.occupied_bitboard())
						} else if piece_type == pieces::ROOK {
							get_rook_moves(piece_index, self.occupied_bitboard())
						} else if piece_type == pieces::QUEEN {
							get_bishop_moves(piece_index, self.occupied_bitboard()) | get_rook_moves(piece_index, self.occupied_bitboard())
						} else { // King
							KING_ATTACKS[piece_index as usize]
						};
				}
			}

			self.attacked_squares[other_color] = result;
		}
	}

	pub fn king_in_check(&mut self) -> bool {
		self.calculate_attacked_squares();
		self.piece_bitboards[pieces::build_piece(self.white_to_move, pieces::KING) as usize] & self.attacked_squares[!self.white_to_move as usize] != 0
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
			self.color_bitboards[(!is_white) as usize] ^= 1 << data.to;
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
			self.color_bitboards[(!is_white) as usize] ^= 1 << data.to;
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

		self.attacked_squares[!self.white_to_move as usize] = 0;

		if self.king_in_check() {
			self.white_to_move = !self.white_to_move;
			self.undo_move(data);
			return false;
		}

		self.white_to_move = !self.white_to_move;

		true
	}

	pub fn undo_move(&mut self, data: &MoveData) {
		self.undo_move_piece(data);

		self.white_to_move = !self.white_to_move;
	}

	pub fn get_moves_for_piece(&self, piece_index: u8, captures_only: bool) -> Vec<MoveData> {
		let mut moves = vec![];
		let piece = self.get(piece_index);
		let piece_type = pieces::get_type(piece);
		let is_white_piece = pieces::is_white(piece);
		let other_color = (!is_white_piece) as usize;

		if piece_type == pieces::PAWN {
			let rank = piece_index / 8;
			let will_promote;

			// TODO: clean this up!!!
			if is_white_piece {
				will_promote = rank == 1;

				if !captures_only {
					// Pushing
					if self.get(piece_index - 8) == pieces::NONE {
						if will_promote {
							for promotion in pieces::KNIGHT..=pieces::QUEEN {
								moves.push(
									MoveData {
										from: piece_index,
										to: piece_index - 8,
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
									to: piece_index - 8,
									piece,
									capture: pieces::NONE,
									flag: flag::NONE,
								}
							);
						}

						if rank == 6
						&& self.get(piece_index - 16) == pieces::NONE {
							moves.push(
								MoveData {
									from: piece_index,
									to: piece_index - 16,
									piece,
									capture: pieces::NONE,
									flag: flag::DOUBLE_PAWN_PUSH,
								}
							);
						}
					}
				}
			} else {
				will_promote = rank == 6;

				if !captures_only {
					// Pushing
					if self.get(piece_index + 8) == pieces::NONE {
						if will_promote {
							for promotion in pieces::KNIGHT..=pieces::QUEEN {
								moves.push(
									MoveData {
										from: piece_index,
										to: piece_index + 8,
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
									to: piece_index + 8,
									piece,
									capture: pieces::NONE,
									flag: flag::NONE,
								}
							);
						}

						if rank == 1
						&& self.get(piece_index + 16) == pieces::NONE {
							moves.push(
								MoveData {
									from: piece_index,
									to: piece_index + 16,
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
				& self.color_bitboards[(!is_white_piece) as usize];

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
				bitboard &= self.color_bitboards[(!is_white_piece) as usize];
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

	pub fn get_moves(&self) -> Vec<MoveData> {
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
				moves.append(&mut self.get_moves_for_piece(piece_index, ALL_MOVES));
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
}