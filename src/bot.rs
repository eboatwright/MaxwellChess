use crate::constants::{is_checkmate, ply_from_mate};
use crate::transposition_table::{TranspositionTable, TTEntry, EvalBound};
use crate::move_list::MoveList;
use crate::constants::CHECKMATE;
use crate::pieces;
use crate::move_data::{NULL_MOVE, MoveData};
use crate::board::*;
use std::time::Instant;

pub const MAX_DEPTH: u8 = 100;

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

pub struct Bot {
	pub board: Board,
	pub transposition_table: TranspositionTable,

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

	pub fn go(&mut self, movetime: Option<f32>, max_depth: u8, output: BotOutput) {
		self.output = output;
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

		println!("bestmove {}", self.best_move.to_coordinates());
	}

	pub fn should_cancel_search(&self) -> bool {
		self.search_cancelled || (self.movetime.is_some() && self.nodes % 20_000 == 0 && self.timer.elapsed().as_secs_f32() >= self.movetime.unwrap())
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
			self.seldepth = u8::max(self.seldepth, depth);

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

	// TODO: tweak this
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