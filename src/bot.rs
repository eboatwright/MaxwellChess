use crate::constants::{is_checkmate, ply_from_mate};
use crate::transposition_table::{TranspositionTable, TTEntry, EvalBound};
use crate::move_list::MoveList;
use crate::constants::CHECKMATE;
use crate::pieces;
use crate::move_data::{NULL_MOVE, MoveData};
use crate::board::*;
use std::time::Instant;

pub const MAX_DEPTH: u8 = 100;
pub const MAX_KILLER_MOVE_PLY: u8 = 30;
pub const KILLER_MOVES_PER_PLY: usize = 2;

#[derive(Copy, Clone, PartialEq)]
pub enum BotOutput {
	None,
	Uci,
}

pub const MVV_LVA: [i16; 36] = [
	15, 25, 35, 45, 55, 65, // Pawn
	14, 24, 34, 44, 54, 64, // Knight
	13, 23, 33, 43, 53, 63, // Bishop
	12, 22, 32, 42, 52, 62, // Rook
	11, 21, 31, 41, 51, 61, // Queen
	10, 20, 30, 40, 50, 60, // King
];

#[derive(Copy, Clone)]
pub struct KillerMoves {
	pub moves: [MoveData; KILLER_MOVES_PER_PLY],
}

impl KillerMoves {
	pub fn empty() -> Self {
		Self {
			moves: [NULL_MOVE; KILLER_MOVES_PER_PLY],
		}
	}

	pub fn add(&mut self, new_move: MoveData) {
		if self.is_killer(&new_move) {
			return;
		}

		self.moves.rotate_right(1);
		self.moves[0] = new_move;
	}

	pub fn is_killer(&self, new_move: &MoveData) -> bool {
		   &self.moves[0] == new_move
		|| &self.moves[1] == new_move
	}
}

pub struct Bot {
	pub board: Board,
	pub transposition_table: TranspositionTable,
	pub history: [[[i16; 64]; 64]; 2],
	pub killer_moves: [KillerMoves; MAX_KILLER_MOVE_PLY as usize],

	pub nodes: u128,
	pub q_nodes: u128,

	pub best_move: MoveData,
	pub best_eval: i16,

	pub best_move_this_iteration: MoveData,
	pub best_eval_this_iteration: i16,
	pub seldepth: u8,

	pub timer: Instant,
	pub movetime: Option<f32>,
	pub output: BotOutput,
	pub search_cancelled: bool,
	pub searched_one_move: bool,
}

impl Bot {
	pub fn new(fen: &str, tt_mbs: usize) -> Self {
		Self {
			board: Board::new(fen),
			transposition_table: TranspositionTable::new(tt_mbs),
			history: [[[0; 64]; 64]; 2],
			killer_moves: [KillerMoves::empty(); MAX_KILLER_MOVE_PLY as usize],

			nodes: 0,
			q_nodes: 0,

			best_move: NULL_MOVE,
			best_eval: 0,

			best_move_this_iteration: NULL_MOVE,
			best_eval_this_iteration: 0,
			seldepth: 0,

			timer: Instant::now(),
			movetime: None,
			output: BotOutput::Uci,
			search_cancelled: false,
			searched_one_move: false,
		}
	}

	pub fn clear_short_term_mem(&mut self) {
		self.history = [[[0; 64]; 64]; 2];
		self.killer_moves = [KillerMoves::empty(); MAX_KILLER_MOVE_PLY as usize];
	}

	pub fn go(&mut self, movetime: Option<f32>, max_depth: u8, output: BotOutput) {
		self.output = output;
		self.movetime = movetime;

		self.clear_short_term_mem();

		self.nodes = 0;
		self.q_nodes = 0;

		self.best_move = NULL_MOVE;
		self.best_eval = 0;

		self.search_cancelled = false;
		self.timer = Instant::now();

		for depth in 1..=max_depth {
			self.best_move_this_iteration = NULL_MOVE;
			self.best_eval_this_iteration = 0;
			self.searched_one_move = false;

			self.ab_search(depth, 0, -i16::MAX, i16::MAX);

			if self.searched_one_move {
				self.best_move = self.best_move_this_iteration;
				self.best_eval = self.best_eval_this_iteration;

				if is_checkmate(self.best_eval) {
					let ply = ply_from_mate(self.best_eval);

					if ply <= depth {
						self.print_uci_info(
							depth,
							"mate",
							(ply as f32 * 0.5).ceil() as i16 * self.board.perspective()
						);
					}
				} else {
					self.print_uci_info(
						depth,
						"cp",
						self.best_eval,
					);
				}
			}

			if self.search_cancelled {
				break;
			}
		}

		if self.output == BotOutput::Uci {
			println!("bestmove {}", self.best_move.to_coordinates());
		}
	}

	pub fn should_cancel_search(&self) -> bool {
		self.search_cancelled || (self.movetime.is_some() && self.nodes % 20_000 == 0 && self.timer.elapsed().as_secs_f32() >= self.movetime.unwrap())
	}

	pub fn ab_search(&mut self, mut depth: u8, ply: u8, mut alpha: i16, beta: i16) -> i16 {
		if self.should_cancel_search() {
			self.search_cancelled = true;
			return 0;
		}

		let not_root = ply != 0;

		if not_root {
			self.seldepth = u8::max(self.seldepth, ply);
			self.nodes += 1;

			if self.board.is_draw() {
				return 0;
			}

			if is_checkmate(alpha)
			|| is_checkmate(beta) {
				// Mate Distance Pruning
				let mate_value = CHECKMATE - ply as i16;
				let alpha = i16::max(alpha, -mate_value);
				let beta = i16::min(beta, mate_value - 1);
				if alpha >= beta {
					return alpha;
				}
			}
		}

		let (tt_eval, tt_move) = self.transposition_table.lookup(self.board.zobrist.key.peek(), depth, ply, alpha, beta);
		// let not_pv = alpha == beta - 1;

		if not_root {
			if let Some(tt_eval) = tt_eval {
				return tt_eval;
			}
		}

		let in_check = self.board.in_check();
		if in_check {
			depth += 1;
		}

		if depth == 0 {
			self.nodes -= 1;
			return self.q_search(alpha, beta);
		}

		let mut found_legal_move = false;
		let mut found_pv = false;
		let mut best_move_this_search = NULL_MOVE;
		let mut move_list = self.board.get_moves(ALL_MOVES);
		self.score_move_list(
			&mut move_list,
			&(if ply == 0
			&& self.best_move != NULL_MOVE {
				self.best_move
			} else {
				tt_move.unwrap_or(NULL_MOVE)
			}),
			ply,
		);

		for i in 0..move_list.len() {
			let m = move_list.next(i);
			if !self.board.make_move(&m) { continue; }
			let board_state_after_move = self.board.history.peek();

			found_legal_move = true;

			let mut eval = 0;
			let mut needs_fuller_search = true;

			if found_pv {
				// PVS
				eval = -self.ab_search(depth - 1, ply + 1, -alpha - 1, -alpha);
				needs_fuller_search = eval > alpha;
			}

			if needs_fuller_search {
				// Normal search
				eval = -self.ab_search(depth - 1, ply + 1, -beta, -alpha);
			}

			self.board.undo_move(&m);

			if self.should_cancel_search() {
				self.search_cancelled = true;
				return 0;
			}

			if eval >= beta {
				self.transposition_table.store(
					TTEntry {
						key: self.board.zobrist.key.peek(),
						best_move: m,
						eval_bound: EvalBound::Beta,
						eval: beta,
						depth,
					},
					ply,
				);

				if board_state_after_move.capture == pieces::NONE {
					self.history[self.board.white_to_move as usize][m.from as usize][m.to as usize] += depth as i16 * depth as i16;
					if ply < MAX_KILLER_MOVE_PLY {
						self.killer_moves[ply as usize].add(m);
					}
				}

				return beta;
			}

			if eval > alpha {
				alpha = eval;
				best_move_this_search = m;
				found_pv = true;

				if ply == 0 {
					self.searched_one_move = true;
					self.best_move_this_iteration = m;
					self.best_eval_this_iteration = eval;
				}
			}
		}

		if !found_legal_move {
			if self.board.in_check() {
				return -(CHECKMATE - ply as i16);
			}

			return 0; // Stalemate
		}

		// TODO try storing alpha evals
		if best_move_this_search != NULL_MOVE {
			self.transposition_table.store(
				TTEntry {
					key: self.board.zobrist.key.peek(),
					best_move: best_move_this_search,
					eval_bound: EvalBound::Exact,
					eval: alpha,
					depth,
				},
				ply,
			);
		}

		alpha
	}

	// Do I really need to check for mate or stalemate here?
	pub fn q_search(&mut self, mut alpha: i16, beta: i16) -> i16 {
		self.q_nodes += 1;

		let eval = self.board.simple_eval();
		if eval >= beta {
			return beta;
		}

		if eval > alpha {
			alpha = eval;
		}

		let mut move_list = self.board.get_moves(CAPTURES_ONLY);
		self.score_move_list(&mut move_list, &NULL_MOVE, MAX_KILLER_MOVE_PLY);

		for i in 0..move_list.len() {
			let m = move_list.next(i);
			if !self.board.make_move(&m) { continue; }

			let eval = -self.q_search(-beta, -alpha);
			self.board.undo_move(&m);

			if eval >= beta {
				return beta;
			}

			if eval > alpha {
				alpha = eval;
			}
		}

		alpha
	}

	// TODO incremental move sorting?
	pub fn score_move_list(&mut self, move_list: &mut MoveList, best_move: &MoveData, ply: u8) {
		for (m, score) in move_list.moves.iter_mut() {
			if m == best_move {
				*score = 30_000;
			} else {
				let capture = self.board.get(m.to);
				if capture == pieces::NONE {
					*score += self.history[self.board.white_to_move as usize][m.from as usize][m.to as usize];

					if ply < MAX_KILLER_MOVE_PLY
					&& self.killer_moves[ply as usize].is_killer(m) {
						*score += 10_000; // TODO should this be higher or lower than a capture?
					}
				} else {
					*score = 20_000 + MVV_LVA[(pieces::get_type(m.piece) * 6 + pieces::get_type(capture)) as usize];
				}
			}
		}
	}

	// TODO tweak this
	pub fn partition_time(&self, total_time: f32) -> f32 {
		total_time * 0.05
	}

	pub fn print_uci_info(&mut self, depth: u8, score_type: &'static str, score: i16) {
		if self.output != BotOutput::Uci {
			return;
		}

		let time_elapsed = self.timer.elapsed();
		let total_nodes = self.nodes + self.q_nodes;
		let pv = self.find_pv(depth);

		println!("info depth {depth} seldepth {seldepth} score {score_type} {score} currmove {currmove} pv {pv}nodes {nodes} time {time} nps {nps}",
			depth = depth,
			seldepth = self.seldepth,
			score_type = score_type,
			score = score,
			currmove = self.best_move.to_coordinates(),
			pv = pv,
			nodes = total_nodes,
			time = time_elapsed.as_millis(),
			nps = total_nodes as f32 / time_elapsed.as_secs_f32(),
		);
	}

	pub fn find_pv(&mut self, depth: u8) -> String {
		if depth == 0 {
			return String::new();
		}

		if let Some(entry) = self.transposition_table.get(self.board.zobrist.key.peek()) {
			// let hash_move = MoveData::from_binary(entry.best_move);
			let m = entry.best_move.to_coordinates();

			if !self.board.try_move(&m) {
				return String::new();
			}

			let pv = format!("{} {}", m, self.find_pv(depth - 1));

			self.board.undo_move(&entry.best_move);

			return pv;
		}

		String::new()
	}
}