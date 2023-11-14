use crate::SQUARE_SIZE;
use macroquad::prelude::{Vec2, mouse_position};
use crate::heatmaps::*;
use crate::piece::*;

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

pub fn get_worth_for_piece(piece: u8, mut i: usize) -> i32 {
	if !is_white(piece) {
		// let mut p = Point::from_index(i);
		// p.y = 7 - p.y;
		// i = (p.x + p.y * 8) as usize;
		i = 63 - i;
	}

	let worth = match get_piece_type(piece) {
		PAWN => 100   + PAWN_HEATMAP[i],
		KNIGHT => 300 + KNIGHT_HEATMAP[i],
		BISHOP => 320 + BISHOP_HEATMAP[i],
		ROOK => 500   + ROOK_HEATMAP[i],
		QUEEN => 900  + QUEEN_HEATMAP[i],
		KING => 20000 + KING_MIDDLEGAME_HEATMAP[i],

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
	8 - (index as f32 / 8.0).floor() as u8
}