#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod value_holder;
pub mod constants;
pub mod precalculated_data;
pub mod castling_rights;
pub mod utils;
pub mod pieces;
pub mod move_data;
pub mod board;
pub mod perft;
pub mod bot;

use crate::bot::Bot;
use std::time::Instant;
use crate::move_data::flag;
use crate::board::Board;

pub const STARTING_POS:       &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE:           &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const TEST_POSITION_4:    &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
pub const DRAWN_ENDGAME:      &str = "8/8/8/3k4/R5p1/P5r1/4K3/8 w - - 0 1";
pub const MATE_IN_5:          &str = "4r3/7q/nb2prRp/pk1p3P/3P4/P7/1P2N1P1/1K1B1N2 w - - 0 1";
pub const PAWN_ENDGAME:       &str = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1";
pub const ONE_PAWN_ENDGAME:   &str = "8/8/1k6/8/8/1K6/1P6/8 w - - 0 1";
pub const ENDGAME_POSITION:   &str = "8/pk4p1/2prp3/3p1p2/3P2p1/R2BP3/2P2KPP/8 w - - 8 35";
pub const ZUGZWANG_MATE_IN_3: &str = "7k/5Q2/3p4/1p2r1p1/3B2Pp/1p5P/8/6K1 w - - 0 1";

fn main() {
	// precalculated_data::calculate();
	let mut bot = Bot::new(KIWIPETE);

	loop {
		let mut input = String::new();
		std::io::stdin().read_line(&mut input).expect("Failed to read stdin");
		input.pop(); // Pop the \n

		let split = input.split(' ').collect::<Vec<&str>>();

		match split[0] {
			"go" => {
				bot.go();
			}

			"move" => {
				if bot.board.try_move(split[1]) {
					bot.board.print();
				} else {
					println!("Illegal move.");
				}
			}

			"perft" => {
				if let Ok(depth) = split[1].parse::<u8>() {
					perft::run(bot.board.clone(), depth);
				}
			}

			"print" => {
				bot.board.print();
			}

			_ => {}
		}
	}
}