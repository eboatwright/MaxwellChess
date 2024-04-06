use rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct Matrix {
	pub rows: usize,
	pub cols: usize,
	pub data: Vec<f32>,
}

impl Matrix {
	pub fn empty(rows: usize, cols: usize) -> Self {
		Self {
			rows,
			cols,
			data: vec![0.0; rows * cols],
		}
	}

	pub fn random(rows: usize, cols: usize) -> Self {
		let mut data = vec![0.0; rows * cols];
		let mut rng = thread_rng();

		for i in 0..data.len() {
			data[i] = rng.gen_range(0.2..0.8);
		}

		Self {
			rows,
			cols,
			data,
		}
	}

	pub fn fill_zeros(&mut self) {
		self.data.fill(0.0)
	}

	pub fn transposed(&self) -> Self {
		let mut data = vec![0.0; self.data.len()];

		for row in 0..self.rows {
			for col in 0..self.cols {
				data[col * self.rows + row] = self.data[row * self.cols + col];
			}
		}

		Self {
			rows: self.cols,
			cols: self.rows,
			data,
		}
	}

	pub fn map_mut(&mut self, func: fn(f32) -> f32) {
		for i in 0..self.data.len() {
			self.data[i] = func(self.data[i]);
		}
	}

	pub fn map(m: &Self, func: fn(f32) -> f32) -> Self {
		let mut data = vec![0.0; m.data.len()];

		for i in 0..data.len() {
			data[i] = func(m.data[i]);
		}

		Self {
			rows: m.rows,
			cols: m.cols,
			data,
		}
	}



	pub fn add_mut(&mut self, other: &Self) {
		for i in 0..self.data.len() {
			self.data[i] += other.data[i];
		}
	}

	pub fn add(a: &Self, b: &Self) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i] + b.data[i];
		}

		result
	}



	pub fn subtract_mut(&mut self, other: &Self) {
		for i in 0..self.data.len() {
			self.data[i] -= other.data[i];
		}
	}

	pub fn subtract(a: &Self, b: &Self) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i] - b.data[i];
		}

		result
	}



	pub fn multiply_mut(&mut self, other: &Self) {
		for i in 0..self.data.len() {
			self.data[i] *= other.data[i];
		}
	}

	pub fn multiply(a: &Self, b: &Self) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i] * b.data[i];
		}

		result
	}



	pub fn multiply_by_num_mut(&mut self, other: f32) {
		for i in 0..self.data.len() {
			self.data[i] *= other;
		}
	}

	pub fn multiply_by_num(a: &Self, b: f32) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i] * b;
		}

		result
	}



	pub fn divide_mut(&mut self, other: &Self) {
		for i in 0..self.data.len() {
			self.data[i] /= other.data[i];
		}
	}

	pub fn divide(a: &Self, b: &Self) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i] / b.data[i];
		}

		result
	}



	pub fn divide_by_num_mut(&mut self, other: f32) {
		for i in 0..self.data.len() {
			self.data[i] /= other;
		}
	}

	pub fn divide_by_num(a: &Self, b: f32) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i] / b;
		}

		result
	}



	pub fn pow_mut(&mut self, other: f32) {
		for i in 0..self.data.len() {
			self.data[i] = self.data[i].powf(other);
		}
	}

	pub fn pow(a: &Self, b: f32) -> Self {
		let mut result = Self::empty(a.rows, a.cols);

		for i in 0..result.data.len() {
			result.data[i] = a.data[i].powf(b);
		}

		result
	}




	pub fn dot_mut(&mut self, other: &Self) {
		let mut data = vec![0.0; self.rows * other.cols];

		for row in 0..self.rows {
			for col in 0..other.cols {
				let mut sum = 0.0;

				for i in 0..self.cols {
					sum += self.data[row * self.cols + i] * other.data[i * other.cols + col];
				}

				data[row * other.cols + col] = sum;
			}
		}

		self.data = data;
	}

	pub fn dot(a: &Self, b: &Self) -> Self {
		let mut data = vec![0.0; a.rows * b.cols];

		for row in 0..a.rows {
			for col in 0..b.cols {
				let mut sum = 0.0;

				for i in 0..a.cols {
					sum += a.data[row * a.cols + i] * b.data[i * b.cols + col];
				}

				data[row * b.cols + col] = sum;
			}
		}

		Self {
			rows: a.rows,
			cols: b.cols,
			data,
		}
	}



	pub fn print(&self) {
		for row in 0..self.rows {
			let mut row_str = "[".to_string();
			for col in 0..self.cols {
				row_str += &format!("{}, ", self.data[row * self.cols + col]);
			}

			row_str.pop();
			row_str.pop();

			println!("{}],", row_str);
		}
	}
}