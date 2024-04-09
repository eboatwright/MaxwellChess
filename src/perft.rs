use std::time::Instant;
use crate::Board;

pub struct PerftResults {
	pub depth: u8,
	pub total_nodes: u128,
	pub leaf_nodes: u128,
}

pub fn run(mut board: Board, depth: u8) -> PerftResults {
	let mut results = PerftResults {
		depth,
		total_nodes: 0,
		leaf_nodes: 0,
	};

	let timer = Instant::now();
	perft(&mut board, depth, &mut results, true);
	let seconds = timer.elapsed().as_secs_f32();

	println!();
	println!("Total nodes: {}", results.total_nodes);
	println!("Leaf nodes: {}", results.leaf_nodes);
	println!("NPS: {}", results.total_nodes as f32 / seconds);
	println!("{} seconds", seconds);

	results
}

fn perft(board: &mut Board, depth: u8, results: &mut PerftResults, root: bool) {
	if !root {
		results.total_nodes += 1;
	}

	if depth == 0 {
		results.leaf_nodes += 1;
		return;
	}

	let moves = board.get_moves();
	for m in moves {
		let leaf_nodes_before_move = results.leaf_nodes;

		board.make_move(&m);
		perft(board, depth - 1, results, false);
		board.undo_move(&m);

		if root {
			println!("{}: {}", m.to_coordinates(), results.leaf_nodes - leaf_nodes_before_move);
		}
	}
}