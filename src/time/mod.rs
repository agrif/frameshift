pub mod epoch;
mod epoch_type;
pub mod name;
mod scale;
mod time_delta;

pub use epoch_type::*;
pub use scale::*;
pub use time_delta::*;

/// 1 us = 1,000 ns
pub const NANOS_PER_MICRO: u32 = 1_000;

/// 1 ms = 1,000,000 ns
pub const NANOS_PER_MILLI: u32 = 1_000_000;

/// 1 s = 1,000,000,000 ns
pub const NANOS_PER_SEC: u32 = 1_000_000_000;
