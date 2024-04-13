use std::fs::File;
use std::sync::mpsc;
use std::thread;
use std::io::Write;
use std::io::stdout;
use crate::board::CAPTURES_ONLY;
use crate::pieces;
use crate::Board;
use crate::STARTING_POS;
use crate::piece_square_tables::*;
use crate::constants::is_checkmate;
use crate::move_data::NULL_MOVE;
use crate::MAX_DEPTH;
use crate::bot::*;
use std::process::Command;
use std::process::Stdio;

pub const GAMES_PER_MATCH: usize = 1000;
pub const THREADS: usize = 4;
pub const TIME_PER_MOVE: f32 = 0.08;
pub const SCALE: f32 = 128.0;

struct Position {
	fen: String,
	result: f32,
}

pub fn sigmoid(x: i16) -> f32 {
	1.0 / (1.0 + f32::exp(-(x as f32 / SCALE)))
	// 1.0 / (1.0 + 10.0f32.powf(-K * x as f32 / 400.0))
}

fn print(s: &str) {
	print!("{}                           \r", s);
	stdout().flush().expect("Failed to flush stdout");
}

pub fn texel_tuning() {
	let mut params = [MG_PAWN_PST, MG_KNIGHT_PST, MG_BISHOP_PST, MG_ROOK_PST, MG_QUEEN_PST, MG_KING_PST].concat();

	println!("### MAXWELL TEXEL TUNER ###");

	let mut tune_rate = 20;
	let mut training_cycle = 0;
	loop {
		training_cycle += 1;
		println!("Training cycle {}:", training_cycle);

		let positions = play_games();

		// print("Saving positions...");

		// let mut positions_output = File::create(&format!("texel_positions/{}", positions.len())).expect("Failed to create position output file");
		// for position in positions.iter() {
		// 	writeln!(
		// 		positions_output,
		// 		"{}:{}",
		// 		position.fen.split(' ').collect::<Vec<&str>>()[0..2].join(" "), // The pieces and the side to move
		// 		position.result,
		// 	).expect("Failed to write position");
		// }

		print("Calculating error...");

		let mut current_error = error(&positions, &params);

		print(&format!("Tuning... (Rate={})", tune_rate));

		for pi in 0..params.len() {
			print(&format!("Tuning... (Rate={}, {}/{})", tune_rate, pi, params.len()));

			let mut momentum_direction = 1;
			let mut improved_weight = true;
			while improved_weight {
				improved_weight = false;
				let old_value = params[pi];

				// +1
				params[pi] += tune_rate * momentum_direction;
				let new_error = error(&positions, &params);

				if new_error < current_error {
					current_error = new_error;
					improved_weight = true;
				} else {
					// If the +1 didn't help, try the opposite direction
					// -1
					params[pi] = old_value - tune_rate * momentum_direction;
					let new_error = error(&positions, &params);

					if new_error < current_error {
						current_error = new_error;
						improved_weight = true;
						// If going the other direction helped more, then reverse the momentum and keep going
						momentum_direction *= -1;
					} else {
						// Set it back where it was
						params[pi] = old_value;
					}
				}
			}
		}
		println!("Finished tuning!                          ");
		if tune_rate > 1 {
			tune_rate -= 1;
			println!("(Reducing rate to {}.)", tune_rate);
		}

		print("Writing to file...");
		let mut output_file = File::create(&format!("texel_psts/{}", training_cycle)).expect("Failed to create output file");
		writeln!(
			output_file,
			"pub const MG_PAWN_PST: [i16; 64] = {:?};
pub const MG_KNIGHT_PST: [i16; 64] = {:?};
pub const MG_BISHOP_PST: [i16; 64] = {:?};
pub const MG_ROOK_PST: [i16; 64] = {:?};
pub const MG_QUEEN_PST: [i16; 64] = {:?};
pub const MG_KING_PST: [i16; 64] = {:?};",
			params[0..64].to_vec(),
			params[64..128].to_vec(),
			params[128..192].to_vec(),
			params[192..256].to_vec(),
			params[256..320].to_vec(),
			params[320..384].to_vec(),
		).expect("Failed to write output file");
		println!("Wrote to file.                \n");
	}
}

fn play_games() -> Vec<Position> {
	let mut positions = vec![];
	let mut threads = 0;
	let mut games = 0;
	let (sender, receiver) = mpsc::channel();

	while games < GAMES_PER_MATCH {
		while threads < THREADS
		&& games + threads < GAMES_PER_MATCH {
			threads += 1;

			let _sender = sender.clone();
			thread::spawn(move || {
				let positions = play_game();
				_sender.send(positions).expect("Failed to send positions from threads");
			});
		}

		print(&format!("Playing games... ({}/{})", games, GAMES_PER_MATCH));

		if let Ok(mut _positions) = receiver.recv() {
			positions.append(&mut _positions);

			threads -= 1;
			games += 1;
		}
	}
	println!("Played {} games.                     ", GAMES_PER_MATCH);

	positions
}

fn play_game() -> Vec<Position> {
	let opening_book =
		Command::new("python3")
			.arg("src/texel/opening_book.py")
			.stdout(Stdio::piped())
			.spawn()
			.expect("Failed to start 'opening_book.py'");

	let opening_book_output = opening_book.wait_with_output().expect("Failed to get output from 'opening_book.py'");
	let opening_fen = String::from_utf8_lossy(&opening_book_output.stdout);

	let mut positions = vec![];
	let mut result = 0.5;
	let mut bot = Bot::new(&opening_fen.clone(), 256);

	loop {
		bot.go(Some(TIME_PER_MOVE), MAX_DEPTH, BotOutput::None);

		if bot.best_move == NULL_MOVE {
			break;
		}

		bot.board.make_move(&bot.best_move);

		if is_checkmate(bot.best_eval) {
			if bot.best_eval * bot.board.perspective() > 0 {
				result = 1.0;
			} else {
				result = 0.0;
			}

			break;
		}

		positions.push(
			Position {
				fen: bot.board.generate_fen(),
				result: 0.5,
			}
		);

		if bot.board.is_draw() {
			break;
		}
	}

	if result != 0.5 {
		for position in positions.iter_mut() {
			position.result = result;
		}
	}

	positions
}

fn error(positions: &Vec<Position>, params: &[i16]) -> f32 {
	let mut bot = Bot::new(STARTING_POS, 0);
	let mut error = 0.0;

	for position in positions {
		bot.board = Board::new(&position.fen);
		let eval = q_search(&mut bot, -i16::MAX, i16::MAX, params);
		error += (position.result - sigmoid(eval)).powf(2.0);
	}

	error / positions.len() as f32
}

fn q_search(bot: &mut Bot, mut alpha: i16, beta: i16, params: &[i16]) -> i16 {
	let mut eval = 0;

	for i in 0..64 {
		let piece = bot.board.get(i);
		if piece != pieces::NONE {
			let piece_type = pieces::get_type(piece);
			if pieces::is_white(piece) {
				eval += params[(piece_type * 64 + i) as usize];
			} else {
				eval -= params[(piece_type * 64 + (63 - i)) as usize];
			}
		}
	}

	if eval >= beta {
		return beta;
	}

	if eval > alpha {
		alpha = eval;
	}

	let mut move_list = bot.board.get_moves(CAPTURES_ONLY);
	bot.score_move_list(&mut move_list, &NULL_MOVE);

	for i in 0..move_list.len() {
		let m = move_list.next(i);
		if !bot.board.make_move(&m) { continue; }

		let eval = -q_search(bot, -beta, -alpha, params);
		bot.board.undo_move(&m);

		if eval >= beta {
			return beta;
		}

		if eval > alpha {
			alpha = eval;
		}
	}

	alpha
}