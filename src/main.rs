#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// #![warn(clippy::cast_lossless)]

pub mod piece_square_tables;
pub mod texel;
pub mod value_holder;
pub mod constants;
pub mod precalculated_data;
pub mod castling_rights;
pub mod move_list;
pub mod utils;
pub mod pieces;
pub mod move_data;
pub mod zobrist;
pub mod transposition_table;
pub mod board;
pub mod perft;
pub mod bot;

use crate::move_data::MoveData;
use crate::texel::texel_tuning;
use crate::bot::*;
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
	let mut bot = Bot::new(STARTING_POS, 256);

	loop {
		let mut input = String::new();
		std::io::stdin().read_line(&mut input).expect("Failed to read stdin");
		input.pop(); // Pop the \n

		let split = input.split(' ').collect::<Vec<&str>>();

		match split[0] {
			// UCI commands

			"uci" => {
				println!("id name Maxwell v4.0");
				println!("id author eboatwright");
				println!("option name Hash type spin default 256 min 0 max 4000");

				println!("uciok");
			}

			"setoption" => {
				if let Some(option_name) = split.get(2) {
					if let Some(value) = split.get(4) {
						match *option_name {
							"Hash" => {
								bot.transposition_table.resize(value.parse::<usize>().unwrap_or(256));
							}

							"Threads" => {
								// TODO
							}

							_ => {}
						}
					}
				}
			}

			"isready" => {
				println!("readyok");
			}

			"ucinewgame" => {
				let mbs = bot.transposition_table.mbs;
				bot = Bot::new(STARTING_POS, mbs);
			}

			"position" => {
				if let Some(position_type) = split.get(1) {
					match *position_type {
						"startpos" => {
							bot.board = Board::new(STARTING_POS);
						}

						// position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e7e5
						"fen" => {
							// I just pass in everything after "fen", because Board::new() will just ignore what it doesn't need :)
							let fen = &split[2..].join(" ");
							bot.board = Board::new(fen);
						}

						_ => {}
					}

					if let Some(mut moves_start) = split.iter().position(|&x| x == "moves") {
						while moves_start < split.len() - 1 {
							moves_start += 1;

							if !bot.board.try_move(split[moves_start]) {
								println!("Illegal move! {}", split[moves_start]);
								break;
							}
						}
					}
				}
			}

			"go" => {
				let mut movetime = None;
				let mut depth = MAX_DEPTH;

				if let Some(prefix) = split.get(1) {
					match *prefix {
						"fulltime" => { // This isn't part of the UCI interface, but it's useful :)
							if let Some(_movetime) = split.get(2) {
								if let Ok(_movetime) = _movetime.parse::<f32>() {
									movetime = Some(_movetime / 1000.0);
								}
							}
						}

						// Engine expects this to be: go wtime X btime Y
						"wtime" => {
							let time_index =
								if bot.board.white_to_move {
									2
								} else {
									4
								};

							if let Some(_movetime) = split.get(time_index) {
								if let Ok(_movetime) = _movetime.parse::<f32>() {
									movetime = Some(bot.partition_time(_movetime / 1000.0));
								}
							}
						}

						"movetime" => {
							if let Some(_movetime) = split.get(2) {
								if let Ok(_movetime) = _movetime.parse::<f32>() {
									movetime = Some(bot.partition_time(_movetime / 1000.0));
								}
							}
						}

						"depth" => {
							if let Some(_depth) = split.get(2) {
								if let Ok(_depth) = _depth.parse::<u8>() {
									depth = _depth;
								}
							}
						}

						_ => {}
					}
				}

				bot.go(movetime, depth, BotOutput::Uci);
			}

			"stop" => {
				// This should stop the engine once I get it multithreaded
				bot.search_cancelled = true;
			}

			"quit" => {
				return;
			}

			// My commands

			"move" => {
				if let Some(coordinates) = split.get(1) {
					if bot.board.try_move(coordinates) {
						bot.board.print();
					} else {
						println!("Illegal move.");
					}
				}
			}

			"perft" => {
				if let Some(depth) = split.get(1) {
					if let Ok(depth) = depth.parse::<u8>() {
						perft::run(bot.board.clone(), depth);
					}
				}
			}

			"print" => {
				bot.board.print();
			}

			"texel" => {
				texel_tuning();
			}

			_ => {}
		}
	}
}