use crate::move_data::NULL_MOVE;
use crate::MAX_DEPTH;
use crate::bot::*;
use std::process::Command;
use std::process::Stdio;

pub const TIME_PER_MOVE: f32 = 1.0;

struct Position {
	fen: String,
	eval: f32,
	result: f32,
}

pub fn texel_tuning() {
	let mg_pawn_pst = [100; 64];
	let eg_pawn_pst = [100; 64];
	let mg_knight_pst = [320; 64];
	let eg_knight_pst = [320; 64];
	let mg_bishop_pst = [330; 64];
	let eg_bishop_pst = [330; 64];
	let mg_rook_pst = [500; 64];
	let eg_rook_pst = [500; 64];
	let mg_queen_pst = [900; 64];
	let eg_queen_pst = [900; 64];
	let mg_king_pst = [0; 64];
	let eg_king_pst = [0; 64];

	play_game();
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

	let mut white_bot = Bot::new(&opening_fen.clone(), 256);
	let mut black_bot = Bot::new(&opening_fen.clone(), 256);

	loop {
		let move_to_play =
			if white_bot.board.white_to_move {
				white_bot.go(Some(TIME_PER_MOVE), MAX_DEPTH, BotOutput::None);
				white_bot.best_move
			} else {
				black_bot.go(Some(TIME_PER_MOVE), MAX_DEPTH, BotOutput::None);
				black_bot.best_move
			};

		if move_to_play == NULL_MOVE {
			break;
		}

		white_bot.board.make_move(&move_to_play);
		black_bot.board.make_move(&move_to_play);
	}

	positions
}