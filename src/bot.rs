use crate::move_list::MoveList;
use crate::constants::CHECKMATE;
use crate::pieces;
use crate::move_data::{NULL_MOVE, MoveData};
use crate::board::*;
use std::time::Instant;

pub const MAX_DEPTH: u8 = 100;

pub const MVV_LVA: [i32; 36] = [
	15, 25, 35, 45, 55, 65, // Pawn
	14, 24, 34, 44, 54, 64, // Knight
	13, 23, 33, 43, 53, 63, // Bishop
	12, 22, 32, 42, 52, 62, // Rook
	11, 21, 31, 41, 51, 61, // Queen
	10, 20, 30, 40, 50, 60, // King
];

pub struct Bot {
	pub timer: Instant,
	pub board: Board,

	pub nodes: u128,
	pub q_nodes: u128,

	pub best_move: MoveData,
	pub best_eval: i32,

	pub best_move_this_iteration: MoveData,
	pub best_eval_this_iteration: i32,
}

impl Bot {
	pub fn new(fen: &'static str) -> Self {
		Self {
			timer: Instant::now(),
			board: Board::new(fen),
			nodes: 0,
			q_nodes: 0,

			best_move: NULL_MOVE,
			best_eval: 0,

			best_move_this_iteration: NULL_MOVE,
			best_eval_this_iteration: 0,
		}
	}

	pub fn go(&mut self) {
		self.nodes = 0;
		self.q_nodes = 0;

		self.best_move = NULL_MOVE;
		self.best_eval = 0;

		self.timer = Instant::now();

		// for depth in 1..=MAX_DEPTH {
		self.ab_search(8, 0, -i32::MAX, i32::MAX);

		self.best_move = self.best_move_this_iteration;
		self.best_eval = self.best_eval_this_iteration;
		// }

		let seconds = self.timer.elapsed().as_secs_f32();
		let total_nodes = self.nodes + self.q_nodes;
		println!("({} nodes + {} q nodes = {} total nodes) / {} seconds = {} NPS", self.nodes, self.q_nodes, total_nodes, seconds, total_nodes as f32 / seconds);
		println!("best move: {}, eval: {}", self.best_move.to_coordinates(), self.best_eval);
	}

	pub fn ab_search(&mut self, depth: u8, ply: u8, mut alpha: i32, beta: i32) -> i32 {
		if depth == 0 {
			return self.q_search(alpha, beta);
		}

		if ply != 0 {
			self.nodes += 1;
		}

		let mut found_legal_move = false;
		let mut move_list = self.board.get_moves(ALL_MOVES);
		self.score_move_list(&mut move_list);

		for i in 0..move_list.len() {
			let m = move_list.next(i);
			if !self.board.make_move(&m) { continue; }

			found_legal_move = true;

			let eval = -self.ab_search(depth - 1, ply + 1, -beta, -alpha);
			self.board.undo_move(&m);

			if eval >= beta {
				return beta;
			}

			if eval > alpha {
				alpha = eval;

				if ply == 0 {
					self.best_move_this_iteration = m;
					self.best_eval_this_iteration = eval;
				}
			}
		}

		if !found_legal_move {
			if self.board.in_check() {
				return -(CHECKMATE - ply as i32);
			}

			return 0; // Stalemate
		}

		alpha
	}

	// Do I really need to check for mate or stalemate here?
	pub fn q_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
		self.q_nodes += 1;

		let eval = self.board.simple_eval();
		if eval >= beta {
			return beta;
		}

		if eval > alpha {
			alpha = eval;
		}

		let mut move_list = self.board.get_moves(CAPTURES_ONLY);
		self.score_move_list(&mut move_list);

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

	pub fn score_move_list(&mut self, move_list: &mut MoveList) {
		for (m, score) in move_list.moves.iter_mut() {
			let capture = self.board.get(m.to);
			if capture != pieces::NONE {
				*score = MVV_LVA[(pieces::get_type(m.piece) * 6 + pieces::get_type(capture)) as usize];
			}
		}
	}
}