use crate::NO_PIECE;
use crate::move_data::*;
use crate::Board;
use std::time::Instant;
use crate::pieces::PROMOTABLE;

#[derive(Default, Debug)]
pub struct PerftResults {
	pub depth: u8,

	pub positions: u128,
	pub nodes: u128,
	pub captures: u128,
	pub en_passants: u128,
	pub castles: u128,
	pub promotions: u128,
	pub checks: u128,
	// pub checkmates: u128,
}

impl PerftResults {
	pub fn new(depth: u8) -> Self {
		Self {
			depth,
			..Default::default()
		}
	}

	pub fn calculate(board: &mut Board, depth: u8) {
		let mut results = PerftResults::new(depth);
		let timer = Instant::now();

		perft(board, &mut results, depth, 0);

		let seconds = timer.elapsed().as_secs_f32();
		println!("\n{} nodes / {} seconds = {:.1} NPS\n", results.nodes, seconds, results.nodes as f32 / seconds);
		println!("{:#?}", results);
	}
}

fn perft(board: &mut Board, results: &mut PerftResults, depth: u8, ply: u8) {
	if ply > 0 {
		results.nodes += 1;
	}

	if depth == 0 {
		results.positions += 1;
		return;
	}

	for data in board.get_pseudo_legal_moves_for_color(board.white_to_move, false) {
		if !board.make_move(data) {
			continue;
		}

		let position_count_before_move = results.positions;

		if data.capture != NO_PIECE as u8 {
			results.captures += 1;

			if data.flag == EN_PASSANT_FLAG {
				results.en_passants += 1;
			}
		} else if data.flag == SHORT_CASTLE_FLAG
		|| data.flag == LONG_CASTLE_FLAG {
			results.castles += 1;
		}

		if PROMOTABLE.contains(&data.flag) {
			results.promotions += 1;
		}

		if board.king_in_check(board.white_to_move) {
			results.checks += 1;
		}

		perft(board, results, depth - 1, ply + 1);
		board.undo_last_move();

		if ply == 0 {
			let positions_this_move = results.positions - position_count_before_move;
			println!("{}: {}", data.to_coordinates(), positions_this_move);
		}
	}
}