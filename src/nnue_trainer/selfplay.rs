use std::process::Stdio;
use std::process::Command;
use rand::{thread_rng, Rng};
use crate::bot::{BotConfig, Bot};
use crate::board::{Board};
use super::config;
// use super::network::Network;
use super::nnue_trainer::DataPoint;
use std::{
	thread,
	sync::mpsc,
	io::{stdout, Write},
};

// network: Network
pub fn play_games() -> Vec<DataPoint> {
	let mut data_points = vec![];
	let mut games_completed = 0;
	let mut concurrent_games = 0;

	let (sender, receiver) = mpsc::channel();

	while games_completed < config::GAMES_PER_BATCH {
		while concurrent_games < config::CONCURRENT_GAMES
		&& games_completed + concurrent_games < config::GAMES_PER_BATCH {
			concurrent_games += 1;

			// let _network = network.clone();
			let _sender = sender.clone();
			thread::spawn(move || {
				let data_points = play_game(); // _network
				_sender.send(data_points).expect("Failed to send data points from thread");
			});
		}

		print!("Playing self-play games... {}/{}\r", games_completed, config::GAMES_PER_BATCH);
		stdout().flush().expect("Failed to flush stdout");

		if let Ok(mut _data_points) = receiver.recv() {
			data_points.append(&mut _data_points);

			games_completed += 1;
			concurrent_games -= 1;
		}
	}

	println!("Completed {} self-play games.              \n", config::GAMES_PER_BATCH);

	data_points
}

// network: Network
fn play_game() -> Vec<DataPoint> {
	let mut data_points = vec![];

	let opening_book =
		Command::new("python3")
			.arg("src/nnue_trainer/opening_book/opening_book.py")
			.stdout(Stdio::piped())
			.spawn()
			.expect("Failed to open opening book");

	let opening_book_output = opening_book.wait_with_output().expect("Failed to get opening book output");
	let opening_fen = String::from_utf8_lossy(&opening_book_output.stdout);

	// let mut rng = thread_rng();
	let mut board = Board::from_fen(
		&opening_fen,

		vec![],
		vec![],
		vec![],
		vec![],
		
		// network.hidden_layer.weights.data,
		// network.hidden_layer.biases.data,
		// network.output_layer.weights.data,
		// network.output_layer.biases.data,
	);
	let mut bot = Bot::new(BotConfig {
		fen: opening_fen.to_string(),
		info_output: false,
		debug_output: false,
		hash_size: 256,
	});

	loop {
		// let move_to_play =
		// 	if rng.gen_range(0..100) < config::PERC_CHANCE_FOR_RANDOM_MOVE {
		// 		let moves = board.get_pseudo_legal_moves_for_color(board.white_to_move, false);
		// 		let mut random_move = moves[rng.gen_range(0..moves.len())];

		// 		while !board.make_move(random_move) {
		// 			random_move = moves[rng.gen_range(0..moves.len())];
		// 		}

		// 		board.undo_last_move();

		// 		random_move
		// 	} else {
		// 		bot.start(&mut board, config::DEPTH_PER_MOVE);
		// 		bot.best_move
		// 	};

		// board.make_move(move_to_play);

		bot.start(&mut board, config::DEPTH_PER_MOVE);
		board.make_move(bot.best_move);

		data_points.push(
			DataPoint {
				fen: board.calculate_fen(),
				outcome: 0.5,
			},
		);

		if board.moves.len() >= config::MAX_PLY {
			break; // This calls a draw after too many moves, and since all the outcomes are set to a draw by default, you can just break
		}

		if let Some(winner) = board.get_winner() {
			if winner != 0.0 {
				let winner = if winner == -1.0 { 0.0 } else { 1.0 };
				for point in data_points.iter_mut() {
					point.outcome = winner;
				}
			}

			break;
		}
	}

	data_points
}