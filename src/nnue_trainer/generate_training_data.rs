use std::io::Write;
use std::fs::File;
use super::selfplay::play_games;

pub fn generate_training_data() {
	let mut i = 0;
	loop {
		i += 1;

		let data_points = play_games();
		let mut output_file = File::create(format!("training_data/{}_positions{}", data_points.len(), i)).expect("Failed to create output file");

		println!("Writing file...");

		let mut output = String::new();

		for point in data_points {
			output += &format!("{},{}\n", point.fen, point.outcome);
		}

		output.pop(); // Pop the last \n

		write!(output_file, "{}", output).expect("Failed to write to output file");

		println!("Done.");
	}
}