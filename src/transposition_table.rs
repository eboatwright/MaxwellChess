use std::mem::size_of;
use crate::move_data::MoveData;

pub const MEGABYTE: usize = 1024 * 1024;
pub const TT_ENTRY_SIZE: usize = size_of::<TTEntry>();

#[derive(Copy, Clone)]
pub enum EvalBound {
	Alpha,
	Beta,
	Exact,
}

#[derive(Clone)]
pub struct TTEntry {
	pub key: u64,
	pub best_move: MoveData, // TODO: maybe pack this into a u16
	pub eval_bound: EvalBound,
	pub eval: i32,
	pub depth: u8,
}

#[derive(Clone)]
pub struct TranspositionTable {
	pub mbs: usize,
	pub entries: Vec<Option<TTEntry>>,
}

impl TranspositionTable {
	pub fn new(mbs: usize) -> Self {
		let length = mbs * MEGABYTE / TT_ENTRY_SIZE;
		Self {
			mbs,
			entries: vec![None; length],
		}
	}

	pub fn get(&mut self) -> Option<TTEntry> {
		None
	}
}