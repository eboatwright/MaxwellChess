pub const CASTLE_KINGSIDE_MASK: [u64; 2] = [
	(1 << 5) | (1 << 6),
	(1 << 61) | (1 << 62),
];

pub const CASTLE_QUEENSIDE_MASK: [u64; 2] = [
	(1 << 1) | (1 << 2) | (1 << 3),
	(1 << 57) | (1 << 58) | (1 << 59),
];

// This is the order they appear in a FEN string
const WHITE_KINGSIDE:  u8 = 0b1000;
const WHITE_QUEENSIDE: u8 = 0b0100;
const BLACK_KINGSIDE:  u8 = 0b0010;
const BLACK_QUEENSIDE: u8 = 0b0001;

const WHITE_BITS: u8 = 0b1100;
const BLACK_BITS: u8 = 0b0011;

#[derive(Copy, Clone)]
pub struct CastlingRights(pub u8);

impl CastlingRights {
	pub fn from_str(s: &str) -> Self {
		let mut value = 0;

		if s.contains('K') { value |= WHITE_KINGSIDE; }
		if s.contains('Q') { value |= WHITE_QUEENSIDE; }
		if s.contains('k') { value |= BLACK_KINGSIDE; }
		if s.contains('q') { value |= BLACK_QUEENSIDE; }

		Self(value)
	}

	pub fn to_str(&self) -> String {
		let mut result = String::new();

		if self.0 & WHITE_KINGSIDE != 0 { result += "K"; }
		if self.0 & WHITE_QUEENSIDE != 0 { result += "Q"; }
		if self.0 & BLACK_KINGSIDE != 0 { result += "k"; }
		if self.0 & BLACK_QUEENSIDE != 0 { result += "q"; }

		if result.is_empty() {
			String::from("-")
		} else {
			result
		}
	}

	pub fn remove_both(&mut self, white: bool) {
		// Doing it this way instead of:
		// self.0 &= !WHITE_BITS >> (white as usize * 2)
		// Avoids a '!' at runtime
		self.0 &= WHITE_BITS >> (white as usize * 2)
	}

	pub fn remove_one(&mut self, square: u8) {
		if square == 0 {
			self.0 &= !BLACK_QUEENSIDE;
		} else if square == 7 {
			self.0 &= !BLACK_KINGSIDE;
		} else if square == 56 {
			self.0 &= !WHITE_QUEENSIDE;
		} else if square == 63 {
			self.0 &= !WHITE_KINGSIDE;
		}
	}

	pub fn kingside(&self, white: bool) -> bool {
		self.0 & (BLACK_KINGSIDE << (white as usize * 2)) != 0
	}

	pub fn queenside(&self, white: bool) -> bool {
		self.0 & (BLACK_QUEENSIDE << (white as usize * 2)) != 0
	}
}