pub fn pop_lsb(value: &mut u64) -> u8 {
	let i = value.trailing_zeros();
	*value &= *value - 1;
	i as u8
}

pub fn get_lsb(value: u64) -> u8 {
	value.trailing_zeros() as u8
}

pub fn print_bitboard(bitboard: u64) {
	for row in 0..8 {
		for col in 0..8 {
			let i = row * 8 + col;
			if bitboard & (1 << i) != 0 {
				print!("1 ");
			} else {
				print!(". ");
			}
		}
		println!();
	}
	println!();
}

pub fn get_sign(x: i16) -> i16 {
	if x > 0 {
		1
	} else {
		-1
	}
}