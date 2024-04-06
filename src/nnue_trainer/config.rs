pub const GAMES_PER_BATCH: usize = 10_000;
pub const MINIBATCH_SIZE: usize = 8_192;
pub const LEARNING_RATE: f32 = 0.0004;

pub const DEPTH_PER_MOVE: u8 = 14;
pub const PERC_CHANCE_FOR_RANDOM_MOVE: u8 = 5;
pub const CONCURRENT_GAMES: usize = 4;
pub const MAX_PLY: usize = 300;

pub const INPUT_NODES: usize = 768;
pub const HIDDEN_NODES: usize = 256;

pub const HIDDEN_BUCKETS: usize = 1;
pub const OUTPUT_BUCKETS: usize = 2;