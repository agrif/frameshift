//! Providers for Earth orientation data and leap seconds.

use crate::time::{Epoch, TimeDelta, TAI, UTC};

pub mod celestrak;

pub use celestrak::CelestrakProvider;

pub trait Provider {
    fn tai_utc_for_utc(&self, epoch: &Epoch<UTC>) -> Option<TimeDelta<TAI>>;
    fn tai_utc_for_tai(&self, epoch: &Epoch<TAI>) -> Option<TimeDelta<TAI>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmptyProvider;

impl Provider for EmptyProvider {
    fn tai_utc_for_utc(&self, _epoch: &Epoch<UTC>) -> Option<TimeDelta<TAI>> {
        None
    }

    fn tai_utc_for_tai(&self, _epoch: &Epoch<TAI>) -> Option<TimeDelta<TAI>> {
        None
    }
}
