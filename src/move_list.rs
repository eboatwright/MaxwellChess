use crate::move_data::MoveData;

pub struct MoveList {
	pub moves: Vec<MoveData>,
}

impl MoveList {
	pub fn new() -> Self {
		Self {
			moves: vec![],
		}
	}

	pub fn len(&self) -> usize {
		self.moves.len()
	}

	pub fn push(&mut self, value: MoveData) {
		self.moves.push(value);
	}

	pub fn pop(&mut self) -> Option<MoveData> {
		self.moves.pop()
	}
}