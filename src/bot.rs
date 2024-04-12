use crate::transposition_table::{TranspositionTable, TTEntry, EvalBound};
use crate::move_list::MoveList;
use crate::constants::CHECKMATE;
use crate::pieces;
use crate::move_data::{NULL_MOVE, MoveData};
use crate::board::*;
use std::time::Instant;

pub const MAX_DEPTH: u8 = 100;

pub const MVV_LVA: [i16; 36] = [
	15, 25, 35, 45, 55, 65, // Pawn
	14, 24, 34, 44, 54, 64, // Knight
	13, 23, 33, 43, 53, 63, // Bishop
	12, 22, 32, 42, 52, 62, // Rook
	11, 21, 31, 41, 51, 61, // Queen
	10, 20, 30, 40, 50, 60, // King
];

pub struct Bot {
	pub board: Board,
	pub transposition_table: TranspositionTable,

	pub nodes: u128,
	pub q_nodes: u128,

	pub best_move: MoveData,
	pub best_eval: i16,

	pub best_move_this_iteration: MoveData,
	pub best_eval_this_iteration: i16,

	pub timer: Instant,
	pub movetime: Option<f32>,
	pub search_cancelled: bool,
	pub searched_one_move: bool,
}

impl Bot {
	pub fn new(fen: &'static str, tt_mbs: usize) -> Self {
		Self {
			board: Board::new(fen),
			transposition_table: TranspositionTable::new(tt_mbs),

			nodes: 0,
			q_nodes: 0,

			best_move: NULL_MOVE,
			best_eval: 0,

			best_move_this_iteration: NULL_MOVE,
			best_eval_this_iteration: 0,

			timer: Instant::now(),
			movetime: None,
			search_cancelled: false,
			searched_one_move: false,
		}
	}

	pub fn go(&mut self, movetime: Option<f32>, max_depth: u8) {
		self.movetime = movetime;

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

				println!("depth: {}, best move: {}, eval: {}", depth, self.best_move.to_coordinates(), self.best_eval);
			}

			if self.search_cancelled {
				break;
			}
		}

		let seconds = self.timer.elapsed().as_secs_f32();
		let total_nodes = self.nodes + self.q_nodes;
		println!("({} nodes + {} q nodes = {} total nodes) / {} seconds = {} NPS", self.nodes, self.q_nodes, total_nodes, seconds, total_nodes as f32 / seconds);
		self.transposition_table.print();
	}

	pub fn should_cancel_search(&self) -> bool {
		self.search_cancelled || (self.movetime.is_some() && self.nodes % 50_000 == 0 && self.timer.elapsed().as_secs_f32() >= self.movetime.unwrap())
	}

	pub fn ab_search(&mut self, depth: u8, ply: u8, mut alpha: i16, beta: i16) -> i16 {
		if self.should_cancel_search() {
			self.search_cancelled = true;
			return 0;
		}

		if depth == 0 {
			return self.q_search(alpha, beta);
		}

		let (tt_eval, tt_move) = self.transposition_table.lookup(self.board.zobrist.key.peek(), depth, ply, alpha, beta);

		if ply != 0 {
			self.nodes += 1;

			if let Some(tt_eval) = tt_eval {
				return tt_eval;
			}
		}

		let mut found_legal_move = false;
		let mut best_move_this_search = NULL_MOVE;
		let mut move_list = self.board.get_moves(ALL_MOVES);
		self.score_move_list(&mut move_list, &tt_move.unwrap_or(NULL_MOVE));

		for i in 0..move_list.len() {
			let m = move_list.next(i);
			if !self.board.make_move(&m) { continue; }

			found_legal_move = true;

			let eval = -self.ab_search(depth - 1, ply + 1, -beta, -alpha);
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

				return beta;
			}

			if eval > alpha {
				alpha = eval;
				best_move_this_search = m;

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

		// TODO: try storing alpha evals
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
		self.score_move_list(&mut move_list, &NULL_MOVE);

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

	pub fn score_move_list(&mut self, move_list: &mut MoveList, tt_move: &MoveData) {
		for (m, score) in move_list.moves.iter_mut() {
			if *m == self.best_move {
				*score = i16::MAX;
			} else if m == tt_move {
				*score = i16::MAX - 1;
			} else {
				let capture = self.board.get(m.to);
				if capture != pieces::NONE {
					*score = MVV_LVA[(pieces::get_type(m.piece) * 6 + pieces::get_type(capture)) as usize];
				}
			}
		}
	}
}