pub const PAWN:     u8 = 0;
pub const KNIGHT:   u8 = 1;
pub const BISHOP:   u8 = 2;
pub const ROOK:     u8 = 3;
pub const QUEEN:    u8 = 4;
pub const KING:     u8 = 5;

pub const BLACK: usize = 0;
pub const WHITE: usize = 1;

pub const BLACK_PAWN:     u8 = PAWN;
pub const BLACK_KNIGHT:   u8 = KNIGHT;
pub const BLACK_BISHOP:   u8 = BISHOP;
pub const BLACK_ROOK:     u8 = ROOK;
pub const BLACK_QUEEN:    u8 = QUEEN;
pub const BLACK_KING:     u8 = KING;

pub const WHITE_PAWN:     u8 = PAWN + 6;
pub const WHITE_KNIGHT:   u8 = KNIGHT + 6;
pub const WHITE_BISHOP:   u8 = BISHOP + 6;
pub const WHITE_ROOK:     u8 = ROOK + 6;
pub const WHITE_QUEEN:    u8 = QUEEN + 6;
pub const WHITE_KING:     u8 = KING + 6;

pub const NONE: u8 = 12;
pub const COUNT: u8 = 12;

pub fn build_piece(is_white: bool, piece_type: u8) -> u8 {
	piece_type + is_white as u8 * 6
}

pub fn is_white(piece: u8) -> bool {
	piece > BLACK_KING
}

pub fn get_type(piece: u8) -> u8 {
	piece % 6
}

pub fn get_color_index(piece: u8) -> usize {
	is_white(piece) as usize
}

pub fn get_color_offset(piece: u8) -> u8 {
	if is_white(piece) {
		6
	} else {
		0
	}
}

pub fn can_pawn_capture(piece: u8, capture: u8) -> bool {
	   capture != NONE
	&& is_white(piece) != is_white(capture)
}

pub fn to_char(piece: u8) -> char {
	match piece {
		BLACK_PAWN => 'p',
		BLACK_KNIGHT => 'n',
		BLACK_BISHOP => 'b',
		BLACK_ROOK => 'r',
		BLACK_QUEEN => 'q',
		BLACK_KING => 'k',

		WHITE_PAWN => 'P',
		WHITE_KNIGHT => 'N',
		WHITE_BISHOP => 'B',
		WHITE_ROOK => 'R',
		WHITE_QUEEN => 'Q',
		WHITE_KING => 'K',

		_ => ' ',
	}
}

pub fn to_str(piece: u8) -> &'static str {
	match piece {
		BLACK_PAWN => "p",
		BLACK_KNIGHT => "n",
		BLACK_BISHOP => "b",
		BLACK_ROOK => "r",
		BLACK_QUEEN => "q",
		BLACK_KING => "k",

		WHITE_PAWN => "P",
		WHITE_KNIGHT => "N",
		WHITE_BISHOP => "B",
		WHITE_ROOK => "R",
		WHITE_QUEEN => "Q",
		WHITE_KING => "K",

		_ => " ",
	}
}

pub fn from_char(c: char) -> u8 {
	match c {
		'p' => BLACK_PAWN,
		'n' => BLACK_KNIGHT,
		'b' => BLACK_BISHOP,
		'r' => BLACK_ROOK,
		'q' => BLACK_QUEEN,
		'k' => BLACK_KING,

		'P' => WHITE_PAWN,
		'N' => WHITE_KNIGHT,
		'B' => WHITE_BISHOP,
		'R' => WHITE_ROOK,
		'Q' => WHITE_QUEEN,
		'K' => WHITE_KING,

		_ => NONE,
	}
}