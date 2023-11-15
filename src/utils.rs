use crate::Resources;
use crate::Board;
use crate::SQUARE_SIZE;
use macroquad::prelude::{Vec2, mouse_position};
use crate::heatmaps::*;
use crate::piece::*;

pub const MAX_LEGAL_MOVES: usize = 218;
pub const CHECKMATE_EVAL: i32 = 100000;

pub fn evaluation_is_mate(evaluation: i32) -> bool {
	evaluation.abs() > CHECKMATE_EVAL - 100
}

pub fn moves_from_mate(evaluation: i32) -> i32 {
	(CHECKMATE_EVAL - evaluation.abs()) / 2
}

pub fn get_image_index_for_piece(piece: u8) -> usize {
	if piece == 0 {
		return 0;
	}

	let base = match get_piece_type(piece) {
		PAWN => 1,
		KNIGHT => 2,
		BISHOP => 3,
		ROOK => 4,
		QUEEN => 5,
		KING => 6,

		_ => 0,
	};

	if is_white(piece) {
		return base;
	} else {
		return base + 6;
	}
}

pub fn get_full_piece_worth(piece: u8, mut i: usize) -> i32 {
	if !is_white(piece) {
		// let mut p = Point::from_index(i);
		// p.y = 7 - p.y;
		// i = (p.x + p.y * 8) as usize;
		i = 63 - i;
	}

	let worth = match get_piece_type(piece) {
		PAWN => PAWN_WORTH     + PAWN_HEATMAP[i],
		KNIGHT => KNIGHT_WORTH + KNIGHT_HEATMAP[i],
		BISHOP => BISHOP_WORTH + BISHOP_HEATMAP[i],
		ROOK => ROOK_WORTH     + ROOK_HEATMAP[i],
		QUEEN => QUEEN_WORTH   + QUEEN_HEATMAP[i],
		KING => KING_WORTH     + KING_MIDDLEGAME_HEATMAP[i],

		_ => 0,
	};

	worth
}

// If somebody knows a better way to do this please @ me :/
pub fn index_from_coordinate(coordinate: &'static str) -> Option<usize> {
	if coordinate.len() != 2 {
		return None;
	}


	let split = coordinate.to_string().chars().collect::<Vec<char>>();



	let file_index = match split[0] {
		'a' => 0,
		'b' => 1,
		'c' => 2,
		'd' => 3,
		'e' => 4,
		'f' => 5,
		'g' => 6,
		'h' => 7,
		_ => 69,
	};

	let rank = if split[1].is_digit(10) {
		split[1].to_digit(10).unwrap() as usize
	} else {
		69
	};



	let full_index = file_index + rank * 8;



	if full_index >= 64 {
		return None;
	}
	Some(full_index)
}

// This is only here for Rust borrowing reasons :P
pub fn mouse_position_vec2() -> Vec2 { mouse_position().into() }

pub fn get_mouse_position_as_index() -> usize {
	let square_mouse = (mouse_position_vec2() / SQUARE_SIZE).floor();
	(square_mouse.x + square_mouse.y * 8.0) as usize
}

pub fn rank_of_index(index: usize) -> u8 {
	8 - (index / 8) as u8
}

pub fn piece_to_char(piece: u8) -> char {
	match (piece & COLOR_MASK, piece & PIECE_MASK) {
		(WHITE, PAWN) => '♟',
		(WHITE, KNIGHT) => '♞',
		(WHITE, BISHOP) => '♝',
		(WHITE, ROOK) => '♜',
		(WHITE, QUEEN) => '♛',
		(WHITE, KING) => '♚',

		(BLACK, PAWN) => '♙',
		(BLACK, KNIGHT) => '♘',
		(BLACK, BISHOP) => '♗',
		(BLACK, ROOK) => '♖',
		(BLACK, QUEEN) => '♕',
		(BLACK, KING) => '♔',

		_ => '.'
	}
}

pub fn position_counter_test(board: &mut Board, depth: u8, total_captures: &mut u64, total_checks: &mut u64) -> u64 {
	if depth == 0 {
		return 1;
	}

	let mut total_positions = 0;

	let legal_moves = board.get_legal_moves_for_color(board.whites_turn);
	for legal_move in legal_moves.iter() {
		board.make_move(*legal_move);

		if get_move_capture(*legal_move) != 0 {
			*total_captures += 1;
			// board.print_to_console();
		}

		if board.king_in_check(board.whites_turn) {
			*total_checks += 1;
		}

		// thread::sleep(Duration::from_millis(100));

		total_positions += position_counter_test(board, depth - 1, total_captures, total_checks);

		board.undo_last_move();
	}

	total_positions
}