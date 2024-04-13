use crate::utils::get_sign;
use crate::constants::is_checkmate;
use std::mem::size_of;
use crate::move_data::MoveData;

pub const MEGABYTE: usize = 1024 * 1024;
pub const TT_ENTRY_SIZE: usize = size_of::<TTEntry>();

#[derive(Copy, Clone, PartialEq)]
pub enum EvalBound {
	Alpha,
	Beta,
	Exact,
}

#[derive(Copy, Clone)]
pub struct TTEntry {
	pub key: u64,
	pub best_move: MoveData, // TODO maybe pack this into a u16
	pub eval_bound: EvalBound,
	pub eval: i16,
	pub depth: u8,
}

#[derive(Clone)]
pub struct TranspositionTable {
	pub mbs: usize,
	pub table: Vec<Option<TTEntry>>,
	pub entries: usize,
}

impl TranspositionTable {
	pub fn new(mbs: usize) -> Self {
		let length = mbs * MEGABYTE / TT_ENTRY_SIZE;
		Self {
			mbs,
			table: vec![None; length],
			entries: 0,
		}
	}

	pub fn resize(&mut self, mbs: usize) {
		let length = mbs * MEGABYTE / TT_ENTRY_SIZE;
		self.mbs = mbs;
		self.table = vec![None; length];
		self.entries = 0;
	}

	pub fn print(&self) {
		let size = (self.entries * TT_ENTRY_SIZE) as f32 / MEGABYTE as f32;
		println!("tt size: {} MB / {} MB", size, self.mbs);
	}

	pub fn get_index(&self, key: u64) -> usize { (key as usize) % self.table.len() }

	pub fn store(&mut self, mut new_entry: TTEntry, ply: u8) {
		if self.table.is_empty() {
			return;
		}

		let index = self.get_index(new_entry.key);

		if let Some(entry) = &self.table[index] {
			// Prefer deeper depth
			if entry.depth > new_entry.depth {
				return;
			}

			// Prefer exact eval bounds
			if entry.depth == new_entry.depth
			&& (entry.eval_bound == EvalBound::Exact
			|| new_entry.eval_bound != EvalBound::Exact) {
				return;
			}
		} else {
			self.entries += 1;
		}

		if is_checkmate(new_entry.eval) {
			let sign = get_sign(new_entry.eval);
			new_entry.eval = (new_entry.eval * sign + ply as i16) * sign;
		}

		self.table[index] = Some(new_entry);
	}

	pub fn get(&self, key: u64) -> Option<TTEntry> {
		if self.table.is_empty() {
			return None;
		}

		if let Some(Some(entry)) = self.table.get(self.get_index(key)) {
			if entry.key == key {
				return Some(*entry);
			}
		}

		None
	}

	pub fn lookup(&mut self, key: u64, depth: u8, ply: u8, alpha: i16, beta: i16) -> (Option<i16>, Option<MoveData>) {
		if let Some(entry) = self.get(key) {
			let mut return_eval = None;

			if entry.depth >= depth {
				match entry.eval_bound {
					EvalBound::Alpha => if entry.eval <= alpha { return_eval = Some(alpha); }
					EvalBound::Beta => if entry.eval >= beta { return_eval = Some(beta); }
					EvalBound::Exact => {
						let mut eval = entry.eval;

						if is_checkmate(eval) {
							let sign = get_sign(eval);
							eval = (eval * sign - ply as i16) * sign;
						}

						return_eval = Some(eval);
					}
				}
			}

			return (return_eval, Some(entry.best_move));
		}

		(None, None)
	}
}