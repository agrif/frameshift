use super::{Epoch, TimeDelta, NANOS_PER_MILLI};
use crate::provider::{EmptyProvider, Provider};

// augh const fn
const fn time_delta<S>(secs: i64, nanos: u32) -> TimeDelta<S> {
    match TimeDelta::new(secs, nanos) {
        Some(delta) => delta,
        None => panic!("bad TimeDelta"),
    }
}

/// A timescale in which to measure [Epoch][super::Epoch]s and [TimeDelta]s.
pub trait Scale {
    /// The name of this timescale, used in [core::fmt::Display].
    const NAME: &'static str;
}

/// Convert from one timescale to another, using orientation data.
pub trait FromScaleWith<Other>: Sized {
    /// Convert the given [TimeDelta] from [FRAMESHIFT_0][super::name::FRAMESHIFT_0].
    fn from_frameshift_with<P>(provider: &P, other: &TimeDelta<Other>) -> Option<TimeDelta<Self>>
    where
        P: Provider;
}

/// Convert to one timescale from another, using orientation data.
pub trait ToScaleWith<Other>: Sized {
    /// Convert the given [TimeDelta] from [FRAMESHIFT_0][super::name::FRAMESHIFT_0].
    fn to_frameshift_with<P>(provider: &P, delta: &TimeDelta<Self>) -> Option<TimeDelta<Other>>
    where
        P: Provider;
}

/// Convert from one timescale to another, statelessly.
///
/// Implementing this is a promise that
/// [FromScaleWith::from_frameshift_with] does not use the provider,
/// and always returns [Some].
pub trait FromScale<Other>: FromScaleWith<Other> {
    /// Convert the given [TimeDelta] from [FRAMESHIFT_0][super::name::FRAMESHIFT_0].
    fn from_frameshift(other: &TimeDelta<Other>) -> TimeDelta<Self> {
        Self::from_frameshift_with(&EmptyProvider, other).unwrap()
    }
}

/// Convert to one timescale from another, statelessly.
///
/// Implementing this is a promise that
/// [ToScaleWith::to_frameshift_with] does not use the provider,
/// and always returns [Some].
pub trait ToScale<Other>: ToScaleWith<Other> {
    /// Convert the given [TimeDelta] from [FRAMESHIFT_0][super::name::FRAMESHIFT_0].
    fn to_frameshift(delta: &TimeDelta<Self>) -> TimeDelta<Other> {
        Self::to_frameshift_with(&EmptyProvider, delta).unwrap()
    }
}

impl<A, B> ToScaleWith<B> for A
where
    B: FromScaleWith<A>,
{
    fn to_frameshift_with<P>(provider: &P, other: &TimeDelta<Self>) -> Option<TimeDelta<B>>
    where
        P: Provider,
    {
        B::from_frameshift_with(provider, other)
    }
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
pub struct TAI;

impl Scale for TAI {
    const NAME: &'static str = "TAI";
}

impl FromScaleWith<TAI> for TAI {
    fn from_frameshift_with<P>(_provider: &P, other: &TimeDelta<Self>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        Some(*other)
    }
}

impl FromScale<TAI> for TAI {}

impl FromScaleWith<TT> for TAI {
    fn from_frameshift_with<P>(_provider: &P, other: &TimeDelta<TT>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        Some((*other - TT_TAI_OFFSET).transmute())
    }
}

impl FromScale<TT> for TAI {}

/// Terrestrial Time.
pub struct TT;

const TT_TAI_OFFSET: TimeDelta<TT> = time_delta(32, 184 * NANOS_PER_MILLI);

impl Scale for TT {
    const NAME: &'static str = "TT";
}

impl FromScaleWith<TT> for TT {
    fn from_frameshift_with<P>(_provider: &P, other: &TimeDelta<Self>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        Some(*other)
    }
}

impl FromScale<TT> for TT {}

impl FromScaleWith<TAI> for TT {
    fn from_frameshift_with<P>(_provider: &P, other: &TimeDelta<TAI>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        Some(other.transmute() + TT_TAI_OFFSET)
    }
}

impl FromScale<TAI> for TT {}

/// Coordinated Universal Time.
pub struct UTC;

impl Scale for UTC {
    const NAME: &'static str = "UTC";
}

impl FromScaleWith<UTC> for UTC {
    fn from_frameshift_with<P>(_provider: &P, other: &TimeDelta<Self>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        Some(*other)
    }
}

impl FromScale<UTC> for UTC {}

impl FromScaleWith<TAI> for UTC {
    fn from_frameshift_with<P>(provider: &P, other: &TimeDelta<TAI>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        let tai_utc = provider.tai_utc_for_tai(&Epoch::from_frameshift(*other))?;
        Some((*other - tai_utc).transmute())
    }
}

impl FromScaleWith<UTC> for TAI {
    fn from_frameshift_with<P>(provider: &P, other: &TimeDelta<UTC>) -> Option<TimeDelta<Self>>
    where
        P: Provider,
    {
        let tai_utc = provider.tai_utc_for_utc(&Epoch::from_frameshift(*other))?;
        Some(other.transmute() + tai_utc)
    }
}
