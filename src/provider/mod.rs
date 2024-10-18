//! Providers for Earth orientation data and leap seconds.

use crate::time::{Epoch, TimeDelta, TAI, UTC};

pub mod celestrak;

pub use celestrak::CelestrakProvider;

/// An Earth orientation provider.
///
/// Some reference systems are referenced to the position of the
/// Earth, which varies over time and is hard to predict. Many
/// reference organizations publish files containing Earth Orientation
/// Parameters, or EOPs. This trait provides an interface to access
/// them.
///
/// This crate also lumps leap second info into this, since they are
/// related concepts and many data sources provide leap second info
/// alongside EOPs.
///
/// Look at the documentation for implementors of this trait to learn
/// how to load this data for use.
pub trait Provider {
    /// Return TAI - UTC for the given UTC [Epoch].
    ///
    /// Returns [None] if data for this epoch is not available.
    fn tai_utc_for_utc(&self, epoch: &Epoch<UTC>) -> Option<TimeDelta<TAI>>;
    /// Return TAI - UTC for the given TAI [Epoch].
    ///
    /// Returns [None] if data for this epoch is not available.
    fn tai_utc_for_tai(&self, epoch: &Epoch<TAI>) -> Option<TimeDelta<TAI>>;
}

/// An empty Earth orientation provider.
///
/// This provider always returns [None] for all info. It is mostly
/// used as a dummy argument to
/// [ToScaleWith][crate::time::ToScaleWith] for conversions that do
/// not need orientation data.
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
