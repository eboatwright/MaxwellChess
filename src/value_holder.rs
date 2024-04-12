#[derive(Clone)]
pub struct ValueHolder<T: Copy> {
	pub index: usize,
	pub history: Vec<T>
}

impl<T: Copy> ValueHolder<T> {
	pub fn new(initial_value: T) -> Self {
		Self {
			index: 0,
			history: vec![initial_value],
		}
	}

	pub fn peek(&self) -> T {
		self.history[self.index]
	}

	pub fn peek_mut(&mut self) -> &mut T {
		&mut self.history[self.index]
	}

	pub fn is_empty(&self) -> bool {
		self.index == 0
	}

	pub fn push(&mut self, value: T) {
		if self.index == self.history.len() - 1 {
			self.history.push(value);
		} else {
			self.history[self.index + 1] = value;
		}

		self.index += 1;
	}

	pub fn pop(&mut self) {
		self.index -= 1;
	}
}