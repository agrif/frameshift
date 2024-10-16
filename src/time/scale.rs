use super::{TimeDelta, NANOS_PER_MILLI};

// augh const fn
const fn time_delta(secs: i64, nanos: u32) -> chrono::TimeDelta {
    match chrono::TimeDelta::new(secs, nanos) {
        Some(delta) => delta,
        None => panic!("bad TimeDelta"),
    }
}

/// A timescale in which to measure [Epoch][super::Epoch]s and [TimeDelta]s.
pub trait Scale {
    /// The name of this timescale, used in [core::fmt::Display].
    const NAME: &'static str;
}

/// Convert from one timescale to another.
pub trait FromScale<Other>: Sized {
    /// Convert the given [TimeDelta] from [FRAMESHIFT_0][super::name::FRAMESHIFT_0].
    fn from_frameshift(other: &TimeDelta<Other>) -> TimeDelta<Self>;
}

/// Convert from one timescale to another.
pub trait ToScale<Other>: Sized {
    /// Convert the given [TimeDelta] from [FRAMESHIFT_0][super::name::FRAMESHIFT_0].
    fn to_frameshift(other: &TimeDelta<Self>) -> TimeDelta<Other>;
}

impl<A, B> ToScale<B> for A
where
    B: FromScale<A>,
{
    fn to_frameshift(other: &TimeDelta<Self>) -> TimeDelta<B> {
        B::from_frameshift(other)
    }
}

/// International Atomic Time (*temps atomique international*).
#[derive(Debug, Clone, Copy, Hash)]
pub struct TAI;

impl Scale for TAI {
    const NAME: &'static str = "TAI";
}

impl FromScale<TAI> for TAI {
    fn from_frameshift(other: &TimeDelta<Self>) -> TimeDelta<Self> {
        *other
    }
}

impl FromScale<TT> for TAI {
    fn from_frameshift(other: &TimeDelta<TT>) -> TimeDelta<Self> {
        TimeDelta::from_chrono(other.to_chrono() - TT_TAI_OFFSET)
    }
}

/// Terrestrial Time.
#[derive(Debug, Clone, Copy, Hash)]
pub struct TT;

const TT_TAI_OFFSET: chrono::TimeDelta = time_delta(32, 184 * NANOS_PER_MILLI);

impl Scale for TT {
    const NAME: &'static str = "TT";
}

impl FromScale<TT> for TT {
    fn from_frameshift(other: &TimeDelta<Self>) -> TimeDelta<Self> {
        *other
    }
}

impl FromScale<TAI> for TT {
    fn from_frameshift(other: &TimeDelta<TAI>) -> TimeDelta<Self> {
        TimeDelta::from_chrono(other.to_chrono() + TT_TAI_OFFSET)
    }
}
