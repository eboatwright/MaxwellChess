pub const CASTLE_KINGSIDE_MASK: [u64; 2] = [
	(1 << 5) | (1 << 6),
	(1 << 61) | (1 << 62),
];

pub const CASTLE_QUEENSIDE_MASK: [u64; 2] = [
	(1 << 1) | (1 << 2) | (1 << 3),
	(1 << 57) | (1 << 58) | (1 << 59),
];

#[derive(Copy, Clone)]
pub struct CastlingRights {
	pub kingside: [bool; 2],
	pub queenside: [bool; 2],
}

impl CastlingRights {
	pub fn from_str(s: &str) -> Self {
		Self {
			kingside: [s.contains('k'), s.contains('K')],
			queenside: [s.contains('q'), s.contains('Q')],
		}
	}

	pub fn to_str(&self) -> String {
		let mut result = String::new();

		if self.kingside[1] { result += "K"; }
		if self.queenside[1] { result += "Q"; }
		if self.kingside[0] { result += "k"; }
		if self.queenside[0] { result += "q"; }

		if result.is_empty() {
			String::from("-")
		} else {
			result
		}
	}

	pub fn remove_both(&mut self, white: bool) {
		if white {
			self.kingside[1] = false;
			self.queenside[1] = false;
		} else {
			self.kingside[0] = false;
			self.queenside[0] = false;
		}
	}

	pub fn remove_one(&mut self, square: u8) {
		if square == 0 {
			self.queenside[0] = false;
		} else if square == 7 {
			self.kingside[0] = false;
		} else if square == 56 {
			self.queenside[1] = false;
		} else if square == 63 {
			self.kingside[1] = false;
		}
	}
}