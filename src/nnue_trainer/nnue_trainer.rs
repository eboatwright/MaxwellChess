use std::io;
use std::io::BufRead;
use std::fs::File;
use std::io::BufReader;
use super::selfplay::play_games;
use super::network::Network;
use crate::nnue_trainer::config;
use rand::thread_rng;
use rand::prelude::SliceRandom;

pub struct DataPoint {
	pub fen: String,
	pub outcome: f32,
}

fn load_training_data() -> Vec<DataPoint> {
	let file = File::open("training_data/10800552_positions").expect("Failed to open training data");
	let reader = BufReader::new(file);

	let mut data_points = vec![];

	for line in reader.lines() {
		let line = line.expect("Invalid line (?)");
		let split = line.split(',').collect::<Vec<&str>>();

		data_points.push(DataPoint {
			fen: split[0].to_string(),
			outcome: split[1].parse::<f32>().expect("Invalid training data 'outcome'"),
		})
	}

	data_points
}

pub fn nnue_train() {
	println!("### MAXWELL NNUE TRAINER 2.0 ###\n");

	print!("Loading data points...");

	let mut data_points = load_training_data();

	println!(" Done!");

	let mut network = Network::new();
	let mut rng = thread_rng();

	// let mut training_cycle = 0;
	// let mut total_games = 0;
	// let mut total_positions = 0;

	let mut epoch = 1;

	// network.save_weights("Random weights".to_string());

	loop {
		// println!("Training cycle {}:", training_cycle + 1);

		// let mut data_points = play_games(); // network.clone()

		// total_games += config::GAMES;
		// total_positions += data_points.len();

		// println!("Training network...");

		println!("Epoch {}...", epoch);

		data_points.shuffle(&mut rng);

		let mut data_point_index = 0;
		while data_point_index < data_points.len() {
			let next_index = usize::min(data_point_index + config::MINIBATCH_SIZE, data_points.len());

			network.back_prop(data_point_index, &data_points[data_point_index..next_index]);

			data_point_index = next_index;
		}

		network.save_weights(format!("Trained on {} positions, over {} epochs.", data_points.len(), epoch));

		epoch += 1;

		// training_cycle += 1;

		// println!("Total games: {}", total_games);
		// println!("Total positions: {}\n", total_positions);
	}
}