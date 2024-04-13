use crate::piece_square_tables::PSTS;
use crate::zobrist::Zobrist;
use crate::move_list::MoveList;
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
	pub capture: u8,
	pub castling_rights: CastlingRights,
	pub en_passant_square: u8,
	pub halfmove_clock: u8,
}

#[derive(Clone)]
pub struct Board {
	pub piece_bitboards: [u64; pieces::COUNT as usize],
	pub color_bitboards: [u64; 2],
	pub white_to_move: bool,

	pub zobrist: Zobrist,
	pub history: ValueHolder<BoardState>,
}

impl Board {
	pub fn new(fen: &str) -> Self {
		let fen_split = fen.split(' ').collect::<Vec<&str>>();
		let fen_pieces = fen_split[0].replace('/', "");

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

		let mut board = Self {
			piece_bitboards,
			color_bitboards,
			white_to_move: fen_split.get(1).unwrap_or(&"w") == &"w",

			zobrist: Zobrist::empty(),
			history: ValueHolder::new(
				BoardState {
					capture: pieces::NONE,
					castling_rights: CastlingRights::from_str(fen_split.get(2).unwrap_or(&"-")),
					en_passant_square: square_to_index(fen_split.get(3).unwrap_or(&"-")),
					// TODO this doesn't work currently, and I don't need it so I guess I'll come back to it later
					// halfmove_clock: fen_split.get(4).unwrap_or(&"0").parse::<u8>().unwrap_or(0),
					halfmove_clock: 0,
				}
			),
		};

		board.zobrist = Zobrist::calculate(&board);

		board
	}

	pub fn generate_fen(&self) -> String {
		let mut result = String::new();

		for rank in 0..8 {
			let mut empty_squares = 0;

			for file in 0..8 {
				let piece = self.get(rank * 8 + file);
				if piece == pieces::NONE {
					empty_squares += 1;
				} else {
					if empty_squares > 0 {
						result += &empty_squares.to_string();
					}

					result += &pieces::to_char(piece).to_string();

					empty_squares = 0;
				}
			}

			if empty_squares > 0 {
				result += &empty_squares.to_string();
			}

			result += "/";
		}

		result.pop(); // Pop the last '/'

		let state = self.history.peek();

		result += &format!(" {side_to_move} {castling_rights} {en_passant_square} {halfmove_clock} {fullmove_counter}",
			side_to_move=if self.white_to_move { "w" } else { "b" },
			castling_rights=&state.castling_rights.to_str(),
			en_passant_square=if state.en_passant_square == 0 { "-" } else { SQUARES[state.en_passant_square as usize] },
			halfmove_clock=state.halfmove_clock,
			fullmove_counter=(self.history.len() + 1) / 2,
		);

		result
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
		println!("FEN: {}", self.generate_fen());
		println!("Zobrist key: {}", self.zobrist.key.peek());
	}

	pub fn get(&self, i: u8) -> u8 {
		for piece in 0..pieces::COUNT {
			if self.piece_bitboards[piece as usize] & (1 << i) != 0 {
				return piece;
			}
		}

		pieces::NONE
	}

	// Since I'm using ^= this function is used for placing and removing pieces
	pub fn toggle_piece(&mut self, piece: u8, square: u8) {
		let square = 1 << square;
		self.piece_bitboards[piece as usize] ^= square;
		self.color_bitboards[pieces::get_color_index(piece)] ^= square;
	}

	pub fn move_piece(&mut self, piece: u8, from: u8, to: u8) {
		let color = pieces::get_color_index(piece);
		let from = 1 << from;
		let to = 1 << to;

		self.piece_bitboards[piece as usize] ^= from;
		self.piece_bitboards[piece as usize] ^= to;

		self.color_bitboards[color] ^= from;
		self.color_bitboards[color] ^= to;
	}

	pub fn make_move(&mut self, data: &MoveData) -> bool {
		let mut new_state = self.history.peek();
		new_state.capture = self.get(data.to);
		new_state.halfmove_clock += 1;

		if flag::is_promotion(data.flag) {
			self.toggle_piece(data.piece, data.from);
			self.toggle_piece(pieces::build(self.white_to_move, data.flag), data.to);
		} else {
			self.move_piece(data.piece, data.from, data.to);
		}

		if data.flag == flag::DOUBLE_PAWN_PUSH {
			new_state.en_passant_square = (data.to as i8 - PAWN_PUSH[self.white_to_move as usize]) as u8;
		} else {
			if data.flag == flag::EN_PASSANT { // If en passant was played, capture will actually be pieces::NONE because new_state.capture is set at the top of the function
				let pawn_square = new_state.en_passant_square as i8 - PAWN_PUSH[self.white_to_move as usize];
				let pawn_captured = pieces::build(!self.white_to_move, pieces::PAWN);
				self.toggle_piece(pawn_captured, pawn_square as u8);
				new_state.capture = pawn_captured; // Set the capture to the correct pawn, because this is used in undo_move
			} else {
				let piece_type = pieces::get_type(data.piece);

				if piece_type == pieces::KING {
					new_state.castling_rights.remove_both(self.white_to_move);

					if data.flag == flag::CASTLE_KINGSIDE {
						let rook = pieces::build(self.white_to_move, pieces::ROOK);
						self.move_piece(rook, data.to + 1, data.to - 1);
					} else if data.flag == flag::CASTLE_QUEENSIDE {
						let rook = pieces::build(self.white_to_move, pieces::ROOK);
						self.move_piece(rook, data.to - 2, data.to + 1);
					}
				} else if piece_type == pieces::ROOK {
					new_state.castling_rights.remove_one(data.from);
				}

				if new_state.capture != pieces::NONE {
					self.toggle_piece(new_state.capture, data.to);

					if pieces::get_type(new_state.capture) == pieces::ROOK {
						new_state.castling_rights.remove_one(data.to);
					}
				}
			}

			// Reset the en passant square because you can only en passant the turn after the double pawn push
			// This also has to be set after checking if the move was en passant, because we need to check the en passant square (I'm leaving this so I remember lol)
			new_state.en_passant_square = 0;
		}

		if new_state.capture != pieces::NONE
		|| pieces::get_type(data.piece) == pieces::PAWN {
			new_state.halfmove_clock = 0;
		}

		self.zobrist.make_move(&data, &self.history.peek(), &new_state);
		self.history.push(new_state);

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
			let last_state = self.history.peek();
			self.zobrist.key.pop();
			self.history.pop();
			self.white_to_move = !self.white_to_move;

			if flag::is_promotion(data.flag) {
				self.toggle_piece(data.piece, data.from);
				self.toggle_piece(pieces::build(self.white_to_move, data.flag), data.to);
			} else {
				self.move_piece(data.piece, data.to, data.from);
			}

			if data.flag == flag::EN_PASSANT {
				let pawn_square = self.history.peek().en_passant_square as i8 - PAWN_PUSH[self.white_to_move as usize];
				self.toggle_piece(last_state.capture, pawn_square as u8);
			} else if last_state.capture != pieces::NONE {
				self.toggle_piece(last_state.capture, data.to);
			} else if data.flag == flag::CASTLE_KINGSIDE {
				let rook = pieces::build(self.white_to_move, pieces::ROOK);
				self.move_piece(rook, data.to - 1, data.to + 1);
			} else if data.flag == flag::CASTLE_QUEENSIDE {
				let rook = pieces::build(self.white_to_move, pieces::ROOK);
				self.move_piece(rook, data.to + 1, data.to - 2);
			}
		}
	}

	pub fn get_moves_for_piece(&self, piece: u8, piece_index: u8, captures_only: bool, move_list: &mut MoveList) {
		let piece_type = pieces::get_type(piece);
		let is_white = self.white_to_move;
		let other_color = !is_white as usize;

		if piece_type == pieces::PAWN {
			let rank = piece_index / 8;
			let will_promote = rank == SECOND_RANK[other_color];


			// Pushing
			if !captures_only {
				let single_push = (piece_index as i8 + PAWN_PUSH[is_white as usize]) as u8;

				if self.get(single_push) == pieces::NONE {
					if will_promote {
						for promotion in pieces::KNIGHT..=pieces::QUEEN {
							move_list.push(
								MoveData {
									from: piece_index,
									to: single_push,
									piece,
									flag: promotion,
								}
							);
						}
					} else {
						move_list.push(
							MoveData {
								from: piece_index,
								to: single_push,
								piece,
								flag: flag::NONE,
							}
						);
					}

					if rank == SECOND_RANK[is_white as usize] {
						let double_push = (piece_index as i8 + DOUBLE_PAWN_PUSH[is_white as usize]) as u8;

						if self.get(double_push) == pieces::NONE {
							move_list.push(
								MoveData {
									from: piece_index,
									to: double_push,
									piece,
									flag: flag::DOUBLE_PAWN_PUSH,
								}
							);
						}
					}
				}
			}

			// Capturing
			let mut bitboard =
				  PAWN_ATTACKS[piece_index as usize][is_white as usize]
				& self.color_bitboards[!is_white as usize];

			while bitboard != 0 {
				let capture_index = pop_lsb(&mut bitboard);

				if will_promote {
					for promotion in pieces::KNIGHT..=pieces::QUEEN {
						move_list.push(
							MoveData {
								from: piece_index,
								to: capture_index,
								piece,
								flag: promotion,
							}
						);
					}
				} else {
					move_list.push(
						MoveData {
							from: piece_index,
							to: capture_index,
							piece,
							flag: flag::NONE,
						}
					);
				}
			}


			// En passant
			let en_passant_square = self.history.peek().en_passant_square;
			if en_passant_square != 0
			&& PAWN_ATTACKS[piece_index as usize][is_white as usize] & (1 << en_passant_square) != 0 {
				move_list.push(
					MoveData {
						from: piece_index,
						to: en_passant_square,
						piece,
						flag: flag::EN_PASSANT,
					}
				);
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
				& !self.color_bitboards[is_white as usize];

			if captures_only {
				bitboard &= self.color_bitboards[!is_white as usize];
			} else if piece_type == pieces::KING {
				let castling_rights = self.history.peek().castling_rights;

				if castling_rights.kingside(is_white)
				&& CASTLE_KINGSIDE_MASK[is_white as usize] & self.occupied_bitboard() == 0
				&& !self.in_check()
				&& self.get_attackers_of(piece_index + 1) == 0 {
					move_list.push(
						MoveData {
							from: piece_index,
							to: piece_index + 2,
							piece,
							flag: flag::CASTLE_KINGSIDE,
						}
					);
				}

				if castling_rights.queenside(is_white)
				&& CASTLE_QUEENSIDE_MASK[is_white as usize] & self.occupied_bitboard() == 0
				&& !self.in_check()
				&& self.get_attackers_of(piece_index - 1) == 0 {
					move_list.push(
						MoveData {
							from: piece_index,
							to: piece_index - 2,
							piece,
							flag: flag::CASTLE_QUEENSIDE,
						}
					);
				}
			}

			while bitboard != 0 {
				let move_index = pop_lsb(&mut bitboard);

				move_list.push(
					MoveData {
						from: piece_index,
						to: move_index,
						piece,
						flag: flag::NONE,
					}
				);
			}
		}
	}

	pub fn get_moves(&self, captures_only: bool) -> MoveList {
		let mut move_list = MoveList::new();

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
				self.get_moves_for_piece(piece, piece_index, captures_only, &mut move_list);
			}
		}

		move_list
	}

	pub fn try_move(&mut self, coordinates: &str) -> bool {
		let data = MoveData::from_coordinates(coordinates);
		let piece = self.get(data.from);

		if piece == pieces::NONE
		|| pieces::is_white(piece) != self.white_to_move {
			return false;
		}

		let mut move_list = MoveList::new();
		self.get_moves_for_piece(piece, data.from, ALL_MOVES, &mut move_list);
		for (m, _) in move_list.moves {
			if m.to == data.to
			&& (data.flag == flag::NONE || data.flag == m.flag) {
				return self.make_move(&m);
			}
		}

		false
	}

	pub fn perspective(&self) -> i16 {
		if self.white_to_move { 1 } else { -1 }
	}

	pub fn simple_eval(&self) -> i16 {
		let mut material_balance = 0;

		for piece in 0..pieces::COUNT {
			let mut bitboard = self.piece_bitboards[piece as usize];
			while bitboard != 0 {
				let mut square = pop_lsb(&mut bitboard) as usize;
				let color = pieces::get_eval_multiplier(piece);

				if color == -1 {
					square = 63 - square;
				}

				material_balance += PSTS[pieces::get_type(piece) as usize][square] * color;
			}
		}

		material_balance * self.perspective()
	}

	pub fn insufficient_checkmating_material(&self) -> bool {
		let occupied = self.occupied_bitboard().count_ones();
		if occupied == 2 { // 2 Kings
			true
		} else if occupied == 3 { // If you only have one piece other than kings, you need a pawn, rook or queen
			  self.piece_bitboards[pieces::WHITE_PAWN as usize] | self.piece_bitboards[pieces::BLACK_PAWN as usize]
			| self.piece_bitboards[pieces::WHITE_ROOK as usize] | self.piece_bitboards[pieces::BLACK_ROOK as usize]
			| self.piece_bitboards[pieces::WHITE_QUEEN as usize] | self.piece_bitboards[pieces::BLACK_QUEEN as usize] == 0
		} else {
			false
		}
	}

	pub fn is_repetition(&self) -> bool {
		// TODO take into account null moves when I add NMP
		let halfmove_clock = self.history.peek().halfmove_clock as usize;
		let current_key = &self.zobrist.key.peek();

		self.zobrist.key.history[(self.zobrist.key.index - halfmove_clock)..self.zobrist.key.index]
			.iter()
			.rev()
			.skip(1)
			.step_by(2)
			.any(|key| key == current_key)
	}

	pub fn is_draw(&self) -> bool {
		   self.history.peek().halfmove_clock >= 100
		|| self.insufficient_checkmating_material()
		|| self.is_repetition()
	}
}