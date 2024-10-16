//! Constants that refer to specific epochs.

use super::name::name_gregorian;
use super::{Epoch, TT};

/// J2000.0
///
/// January 1, 2000 CE at 12:00 TT (proleptic Gregorian).
pub const J2000: Epoch<TT> = Epoch::from_name(name_gregorian(2000, 1, 1, 12, 0, 0));
