use crate::precalculated_data::*;
use crate::flag;
use crate::utils::pop_lsb;
use crate::pieces;
use crate::move_data::MoveData;

#[derive(Clone)]
pub struct Board {
	pub pieces: [u8; 64],
	pub piece_bitboards: [u64; pieces::COUNT as usize],
	pub color_bitboards: [u64; 2],
	pub whites_turn: bool,
}

impl Board {
	pub fn new(fen: &'static str) -> Self {
		let fen_split = fen.split(' ').collect::<Vec<&str>>();
		let fen_pieces = fen_split[0].replace("/", "");

		let mut pieces = [pieces::NONE; 64];
		let mut piece_bitboards = [0; pieces::COUNT as usize];
		let mut color_bitboards = [0; 2];
		let mut i = 0;

		for c in fen_pieces.chars() {
			if let Ok(empty_squares) = c.to_string().parse::<usize>() {
				i += empty_squares;
			} else {
				let piece = pieces::from_char(c);
				pieces[i] = piece;
				piece_bitboards[piece as usize] |= 1 << i;
				color_bitboards[pieces::get_color_index(piece)] |= 1 << i;

				i += 1;
			}
		}

		Self {
			pieces,
			piece_bitboards,
			color_bitboards,
			whites_turn: fen_split[1] == "w",
		}
	}

	pub fn occupied_bitboard(&self) -> u64 {
		self.color_bitboards[0] | self.color_bitboards[1]
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
		println!("{} to move", if self.whites_turn { "White" } else { "Black" });
	}

	pub fn get(&self, i: u8) -> u8 {
		self.pieces[i as usize]

		// for piece in 0..pieces::COUNT {
		// 	if self.piece_bitboards[piece as usize] & (1 << i) != 0 {
		// 		return piece;
		// 	}
		// }

		// pieces::NONE
	}

	pub fn move_piece(&mut self, data: &MoveData) {
		self.pieces[data.from as usize] = pieces::NONE;
		self.piece_bitboards[data.piece as usize] ^= 1 << data.from;

		if data.capture != pieces::NONE {
			self.piece_bitboards[data.capture as usize] ^= 1 << data.to;
		}

		if flag::is_promotion(data.flag) {
			let promotion_piece = data.flag + pieces::get_color_offset(data.piece);

			self.pieces[data.to as usize] = promotion_piece;
			self.piece_bitboards[promotion_piece as usize] ^= 1 << data.to;
		} else {
			self.pieces[data.to as usize] = data.piece;
			self.piece_bitboards[data.piece as usize] ^= 1 << data.to;
		}

		let color = pieces::get_color_index(data.piece);
		self.color_bitboards[color] ^= 1 << data.from;
		self.color_bitboards[color] ^= 1 << data.to;
	}

	pub fn undo_move_piece(&mut self, data: &MoveData) {
		self.piece_bitboards[data.piece as usize] ^= 1 << data.from;

		if data.capture != pieces::NONE {
			self.piece_bitboards[data.capture as usize] ^= 1 << data.to;
		}

		if flag::is_promotion(data.flag) {
			let promotion_piece = data.flag + pieces::get_color_offset(data.piece);

			self.piece_bitboards[promotion_piece as usize] ^= 1 << data.to;
		} else {
			self.piece_bitboards[data.piece as usize] ^= 1 << data.to;
		}

		self.pieces[data.from as usize] = data.piece;
		self.pieces[data.to as usize] = data.capture;

		let color = pieces::get_color_index(data.piece);
		self.color_bitboards[color] ^= 1 << data.from;
		self.color_bitboards[color] ^= 1 << data.to;
	}

	pub fn make_move(&mut self, data: &MoveData) {
		self.move_piece(data);

		self.whites_turn = !self.whites_turn;
	}

	pub fn undo_move(&mut self, data: &MoveData) {
		self.undo_move_piece(data);

		self.whites_turn = !self.whites_turn;
	}

	pub fn get_moves_for_piece(&self, piece_index: u8) -> Vec<MoveData> {
		let mut moves = vec![];
		let piece = self.get(piece_index);
		let piece_type = pieces::get_type(piece);
		let is_white_piece = pieces::is_white(piece);
		let other_color = (!is_white_piece) as usize;

		if piece_type == pieces::PAWN {
			let rank = piece_index / 8;
			let will_promote;


			if is_white_piece {


				will_promote = rank == 1;

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


			} else {


				will_promote = rank == 6;

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
			if self.whites_turn {
				pieces::WHITE_PAWN..=pieces::WHITE_KING
			} else {
				pieces::BLACK_PAWN..=pieces::BLACK_KING
			};

		for piece in pieces {
			let mut bitboard = self.piece_bitboards[piece as usize];
			while bitboard != 0 {
				let piece_index = pop_lsb(&mut bitboard);
				moves.append(&mut self.get_moves_for_piece(piece_index));
			}
		}

		moves
	}

	pub fn try_move(&mut self, coordinates: &str) -> bool {
		let data = MoveData::from_coordinates(coordinates);
		let piece = self.get(data.from);

		if piece == pieces::NONE
		|| pieces::is_white(piece) != self.whites_turn {
			return false;
		}

		let moves = self.get_moves_for_piece(data.from);
		for m in moves {
			if m.to == data.to
			&& (data.flag == flag::NONE || data.flag == m.flag) {
				self.make_move(&m);
				return true;
			}
		}

		false
	}
}