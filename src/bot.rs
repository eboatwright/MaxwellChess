use std::time::Instant;
use crate::Board;

pub const MAX_DEPTH: u8 = 100;

pub struct Bot {
	pub timer: Instant,
	pub board: Board,
	pub nodes: u128,
}

impl Bot {
	pub fn new(fen: &'static str) -> Self {
		Self {
			timer: Instant::now(),
			board: Board::new(fen),
			nodes: 0,
		}
	}

	pub fn go(&mut self) {
		self.timer = Instant::now();
		// for depth in 1..=MAX_DEPTH {
		self.ab_search(8, 0, -i32::MAX, i32::MAX);
		// }
		let seconds = self.timer.elapsed().as_secs_f32();
		println!("{} nodes / {} seconds = {} NPS", self.nodes, seconds, self.nodes as f32 / seconds);
	}

	pub fn ab_search(&mut self, depth: u8, ply: u8, mut alpha: i32, beta: i32) -> i32 {
		if ply != 0 {
			self.nodes += 1;
		}

		if depth == 0 {
			return self.q_search();
		}

		for m in self.board.get_moves() {
			let evaluation = -self.ab_search(depth - 1, ply + 1, -beta, -alpha);

			if evaluation >= beta {
				return beta;
			}

			if evaluation > alpha {
				alpha = evaluation;
			}
		}

		alpha
	}

	pub fn q_search(&mut self) -> i32 {
		0
	}
}