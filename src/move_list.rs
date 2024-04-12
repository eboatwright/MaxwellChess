use crate::move_data::MoveData;

pub struct MoveList {
	pub moves: Vec<(MoveData, i32)>,
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
		self.moves.push((value, 0));
	}

	pub fn pop(&mut self) -> Option<(MoveData, i32)> {
		self.moves.pop()
	}

	pub fn next(&mut self, i: usize) -> MoveData {
		for j in (i + 1)..self.len() {
			if self.moves[j].1 > self.moves[i].1 {
				self.moves.swap(i, j);
			}
		}

		self.moves[i].0
	}
}