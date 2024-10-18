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

/// Convert an [Epoch] from one [Scale] to another, using an Earth
/// orientation [Provider].
///
/// The resulting [Epoch] will represent the same moment in time as
/// the original [Epoch].
pub trait ToScaleWith<Other> {
    /// Convert the given [Epoch] into a new [Scale].
    fn to_scale_with<P>(&self, provider: &P) -> Option<Epoch<Other>>
    where
        P: Provider;
}

/// Convert an [Epoch] from one [Scale] to another, using an Earth
/// orientation [Provider].
///
/// The resulting [Epoch] will represent the same moment in time as
/// the original [Epoch].
///
/// Implementing this is a promise that [ToScaleWith] does not use the
/// provider, and always returns [Some].
pub trait ToScale<Other>: ToScaleWith<Other> {
    /// Convert the given [Epoch] into a new [Scale].
    fn to_scale(&self) -> Epoch<Other> {
        self.to_scale_with(&EmptyProvider).unwrap()
    }
}

// identity
impl<S> ToScaleWith<S> for Epoch<S> {
    fn to_scale_with<P>(&self, _provider: &P) -> Option<Epoch<S>> {
        Some(*self)
    }
}

impl<S> ToScale<S> for Epoch<S> {}

// helper for conversion via intermediate scale
macro_rules! impl_to_via {
    (ToScaleWith, $Start:tt, $Middle:tt, $End:tt) => {
        static_cond::static_cond_item! {
            if $Start != $End {
                impl ToScaleWith<$End> for Epoch<$Start> {
                    fn to_scale_with<P>(&self, provider: &P) -> Option<Epoch<$End>>
                    where
                        P: Provider,
                    {
                        let middle: Epoch<$Middle> = self.to_scale_with(provider)?;
                        middle.to_scale_with(provider)
                    }
                }
            }
        }
    };

    (ToScale, $Start:tt, $Middle:tt, $End:tt) => {
        static_cond::static_cond_item! {
            if $Start != $End {
                impl_to_via!(ToScaleWith, $Start, $Middle, $End);

                impl ToScale<$End> for Epoch<$Start> {
                    fn to_scale(&self) -> Epoch<$End> {
                        let middle: Epoch<$Middle> = self.to_scale();
                        middle.to_scale()
                    }
                }
            }
        }
    };
}

// given ToScaleWith<TAI>, implement ToScaleWith<TaiLike>
macro_rules! impl_to_tai_family {
    ($Trait:tt, $Scale:tt) => {
        impl_to_via!($Trait, $Scale, TAI, TT);
        impl_to_via!($Trait, $Scale, TAI, GPS);
    };
}

/// International Atomic Time (*temps atomique international*).
pub struct TAI;

impl Scale for TAI {
    const NAME: &'static str = "TAI";
}

/// Terrestrial Time.
pub struct TT;

impl Scale for TT {
    const NAME: &'static str = "TT";
}

const TT_TAI_OFFSET: TimeDelta<TT> = time_delta(32, 184 * NANOS_PER_MILLI);

impl ToScaleWith<TT> for Epoch<TAI> {
    fn to_scale_with<P>(&self, _provider: &P) -> Option<Epoch<TT>>
    where
        P: Provider,
    {
        Some(self.transmute() + TT_TAI_OFFSET)
    }
}

impl ToScale<TT> for Epoch<TAI> {}

impl ToScaleWith<TAI> for Epoch<TT> {
    fn to_scale_with<P>(&self, _provider: &P) -> Option<Epoch<TAI>>
    where
        P: Provider,
    {
        Some((*self - TT_TAI_OFFSET).transmute())
    }
}

impl ToScale<TAI> for Epoch<TT> {}

impl_to_tai_family!(ToScale, TT);
impl_to_via!(ToScaleWith, TT, TAI, UTC);

/// GPS Time.
pub struct GPS;

impl Scale for GPS {
    const NAME: &'static str = "GPS";
}

const GPS_TAI_OFFSET: TimeDelta<GPS> = time_delta(-19, 0);

impl ToScaleWith<GPS> for Epoch<TAI> {
    fn to_scale_with<P>(&self, _provider: &P) -> Option<Epoch<GPS>>
    where
        P: Provider,
    {
        Some(self.transmute() + GPS_TAI_OFFSET)
    }
}

impl ToScale<GPS> for Epoch<TAI> {}

impl ToScaleWith<TAI> for Epoch<GPS> {
    fn to_scale_with<P>(&self, _provider: &P) -> Option<Epoch<TAI>>
    where
        P: Provider,
    {
        Some((*self - GPS_TAI_OFFSET).transmute())
    }
}

impl ToScale<TAI> for Epoch<GPS> {}

impl_to_tai_family!(ToScale, GPS);
impl_to_via!(ToScaleWith, GPS, TAI, UTC);

/// Coordinated Universal Time.
pub struct UTC;

impl Scale for UTC {
    const NAME: &'static str = "UTC";
}

impl ToScaleWith<UTC> for Epoch<TAI> {
    fn to_scale_with<P>(&self, provider: &P) -> Option<Epoch<UTC>>
    where
        P: Provider,
    {
        let tai_utc = provider.tai_utc_for_tai(self)?;
        Some((*self - tai_utc).transmute())
    }
}

impl ToScaleWith<TAI> for Epoch<UTC> {
    fn to_scale_with<P>(&self, provider: &P) -> Option<Epoch<TAI>>
    where
        P: Provider,
    {
        let tai_utc = provider.tai_utc_for_utc(self)?;
        Some(self.transmute() + tai_utc)
    }
}

impl_to_tai_family!(ToScaleWith, UTC);
