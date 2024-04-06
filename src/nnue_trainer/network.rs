use crate::pieces::{char_to_piece, is_piece_white, get_piece_type, NO_PIECE, PAWN, KING};
use crate::nnue_weights::*;
use crate::piece_square_tables::BASE_WORTHS_OF_PIECE_TYPE;
use std::io::{stdout, Write};
use std::fs::File;
use super::{
	config,
	math_funcs::*,
	nnue_trainer::DataPoint,
	matrix::Matrix,
};

type ActivationFn = fn(f32) -> f32;

#[derive(Clone)]
pub struct Layer {
	pub weights: Vec<Matrix>,
	pub biases: Vec<Matrix>,
	activation_fn: ActivationFn,
	derivative_fn: ActivationFn,

	outputs: Matrix,
	bucket: usize,
}

impl Layer {
	fn new(weights: Vec<Matrix>, biases: Vec<Matrix>, activation_fn: ActivationFn, derivative_fn: ActivationFn) -> Self {
		let node_count = biases[0].data.len();

		Self {
			weights,
			biases,
			activation_fn,
			derivative_fn,

			outputs: Matrix::empty(1, node_count),
			bucket: 0,
		}
	}

	fn weights(&self) -> &Matrix {
		&self.weights[self.bucket]
	}

	fn biases(&self) -> &Matrix {
		&self.biases[self.bucket]
	}

	fn forward_pass(&mut self, inputs: &Matrix) {
		self.outputs = Matrix::map(
			&Matrix::add(
				&Matrix::dot(inputs, self.weights()),
				self.biases(),
			),
			self.activation_fn,
		);
	}
}

#[derive(Clone)]
pub struct Network {
	pub inputs: Matrix,
	pub hidden_layer: Layer,
	pub output_layer: Layer,
}

impl Network {
	pub fn new() -> Self {
		let length = config::INPUT_NODES * config::HIDDEN_NODES;

		let mut output_layer_weights = vec![];
		let mut output_layer_biases = vec![];

		for _ in 0..config::OUTPUT_BUCKETS {
			output_layer_weights.push(Matrix::random(config::HIDDEN_NODES, 1));
			output_layer_biases.push(Matrix::random(1, 1));
		}

		Self {
			inputs: Matrix::empty(1, config::INPUT_NODES),
			hidden_layer: Layer::new(
				// vec![
				// 	Matrix {
				// 		rows: config::INPUT_NODES,
				// 		cols: config::HIDDEN_NODES,
				// 		data: HIDDEN_LAYER_WEIGHTS[..length].to_vec(),
				// 	},
				// 	Matrix {
				// 		rows: config::INPUT_NODES,
				// 		cols: config::HIDDEN_NODES,
				// 		data: HIDDEN_LAYER_WEIGHTS[length..].to_vec(),
				// 	},
				// ],
				// vec![
				// 	Matrix {
				// 		rows: 1,
				// 		cols: config::HIDDEN_NODES,
				// 		data: HIDDEN_LAYER_BIASES[..config::HIDDEN_NODES].to_vec(),
				// 	},
				// 	Matrix {
				// 		rows: 1,
				// 		cols: config::HIDDEN_NODES,
				// 		data: HIDDEN_LAYER_BIASES[config::HIDDEN_NODES..].to_vec(),
				// 	},
				// ],
				vec![Matrix::random(config::INPUT_NODES, config::HIDDEN_NODES), Matrix::random(config::INPUT_NODES, config::HIDDEN_NODES)],
				vec![Matrix::random(1, config::HIDDEN_NODES), Matrix::random(1, config::HIDDEN_NODES)],
				clipped_relu,
				clipped_relu_derivative,
			),
			output_layer: Layer::new(
				// vec![Matrix {
				// 	rows: config::HIDDEN_NODES,
				// 	cols: config::OUTPUT_NODES,
				// 	data: OUTPUT_LAYER_WEIGHTS.to_vec(),
				// }],
				// vec![Matrix {
				// 	rows: 1,
				// 	cols: config::OUTPUT_NODES,
				// 	data: OUTPUT_LAYER_BIASES.to_vec(),
				// }],
				output_layer_weights,
				output_layer_biases,
				sigmoid,
				sigmoid_derivative,
			),
		}
	}

	pub fn setup(&mut self, fen: &String) {
		self.inputs.fill_zeros();

		let fen_split = fen.split(' ').collect::<Vec<&str>>();
		let row_split = fen_split[0].split('/').collect::<Vec<&str>>();
		let mut i = 0;

		let mut piece_count = 0;

		for row in row_split {
			for piece_char in row.chars() {
				if let Ok(empty_squares) = piece_char.to_string().parse::<usize>() {
					i += empty_squares;
				} else {
					piece_count += 1;
					i += 1;
				}
			}
		}

		self.output_layer.bucket = (piece_count - 1) / (32 / config::OUTPUT_BUCKETS);
	}

	pub fn feed_forward(&mut self) -> &Matrix {
		self.hidden_layer.forward_pass(&self.inputs);
		self.output_layer.forward_pass(&self.hidden_layer.outputs);

		&self.output_layer.outputs
	}

	pub fn back_prop(&mut self, data_index: usize, data_batch: &[DataPoint]) {
		let mut total_error = 0.0;

		let mut new_output_layer_biases = self.output_layer.biases.clone();
		let mut new_output_layer_weights = self.output_layer.weights.clone();

		let mut new_hidden_layer_biases = self.hidden_layer.biases.clone();
		let mut new_hidden_layer_weights = self.hidden_layer.weights.clone();

		for data_point in data_batch.iter() {
			self.setup(&data_point.fen);
			let output = self.feed_forward().data[0];
			let error = data_point.outcome - output;

			total_error += error.powf(2.0);

			let output_layer_error = Matrix { rows: 1, cols: 1, data: vec![error] };

			let mut output_layer_gradients = Matrix::map(&self.output_layer.outputs, self.output_layer.derivative_fn);
			output_layer_gradients.multiply_mut(&output_layer_error);
			output_layer_gradients.multiply_by_num_mut(config::LEARNING_RATE);

			new_output_layer_biases[self.output_layer.bucket].add_mut(&output_layer_gradients);
			new_output_layer_weights[self.output_layer.bucket].add_mut(&Matrix::dot(&self.hidden_layer.outputs.transposed(), &output_layer_gradients));


			let hidden_layer_error = Matrix::dot(&output_layer_error, &self.output_layer.weights().transposed());

			let mut hidden_layer_gradients = Matrix::map(&self.hidden_layer.outputs, self.hidden_layer.derivative_fn);
			hidden_layer_gradients.multiply_mut(&hidden_layer_error);
			hidden_layer_gradients.multiply_by_num_mut(config::LEARNING_RATE);

			new_hidden_layer_biases[self.hidden_layer.bucket].add_mut(&hidden_layer_gradients);
			new_hidden_layer_weights[self.hidden_layer.bucket].add_mut(&Matrix::dot(&self.inputs.transposed(), &hidden_layer_gradients));
		}

		self.output_layer.biases = new_output_layer_biases;
		self.output_layer.weights = new_output_layer_weights;

		self.hidden_layer.biases = new_hidden_layer_biases;
		self.hidden_layer.weights = new_hidden_layer_weights;

		println!("Batch {}~{} error: {}", data_index, data_index + data_batch.len(), total_error / data_batch.len() as f32);
	}

	pub fn save_weights(&self, comment: String) {
		print!("Saving weights... ");
		stdout().flush().expect("Failed to flush stdout");

		let mut hidden_layer_weights = vec![];
		let mut hidden_layer_biases = vec![];

		for i in 0..config::HIDDEN_BUCKETS {
			hidden_layer_weights.extend(self.hidden_layer.weights[i].data.clone());
			hidden_layer_biases.extend(self.hidden_layer.biases[i].data.clone());
		}

		let mut output_layer_weights = vec![];
		let mut output_layer_biases = vec![];

		for i in 0..config::OUTPUT_BUCKETS {
			output_layer_weights.extend(self.output_layer.weights[i].data.clone());
			output_layer_biases.extend(self.output_layer.biases[i].data.clone());
		}

		let mut output_file = File::create("./src/nnue_trainer/output_weights.rs").expect("Failed to create weight output file");

		write!(output_file, "// {}\n\npub const HIDDEN_LAYER_WEIGHTS: [f32; {}] = {:?};\npub const HIDDEN_LAYER_BIASES: [f32; {}] = {:?};\npub const OUTPUT_LAYER_WEIGHTS: [f32; {}] = {:?};\npub const OUTPUT_LAYER_BIASES: [f32; {}] = {:?};",
			comment,

			hidden_layer_weights.len(),
			hidden_layer_weights,

			hidden_layer_biases.len(),
			hidden_layer_biases,

			output_layer_weights.len(),
			output_layer_weights,

			output_layer_biases.len(),
			output_layer_biases,
		).expect("Failed to save weights");

		println!("Done!\n");
	}
}