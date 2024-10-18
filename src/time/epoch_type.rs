use super::name::{FRAMESHIFT_0, JULIAN_DAY_0, MODIFIED_JULIAN_DAY_0};
use super::{Scale, TimeDelta, ToScale, ToScaleWith, GPS, TAI, TT, UTC};
use crate::provider::Provider;

/// A specific instant in time, measured in a specific [Scale].
pub struct Epoch<Scale> {
    // measures delta from Jan 1, 1900 00:00:00.0 *in this scale*
    // so Jan 1, 1900 00:00 TAI is encoded as TimeDelta<TAI>::new(0, 0)
    // and Jan 1, 1900 00:00 TT is encoded as TimeDelta<TT>::new(0, 0)
    // this date is FRAMESHIFT_0
    delta: TimeDelta<Scale>,
}

impl<S> Epoch<S> {
    pub const fn from_frameshift(delta: TimeDelta<S>) -> Self {
        Self { delta }
    }

    pub const fn to_frameshift(&self) -> TimeDelta<S> {
        self.delta
    }

    pub const fn transmute<T>(&self) -> Epoch<T> {
        Epoch::from_frameshift(self.to_frameshift().transmute())
    }

    pub fn from_name_delta(name: &chrono::NaiveDateTime, delta: TimeDelta<S>) -> Self {
        Self::from_frameshift(TimeDelta::from_chrono(*name - FRAMESHIFT_0) + delta)
    }

    pub fn to_name_delta(&self, name: &chrono::NaiveDateTime) -> TimeDelta<S> {
        self.delta - TimeDelta::from_chrono(*name - FRAMESHIFT_0)
    }

    pub const fn from_name(name: chrono::NaiveDateTime) -> Self {
        Self::from_frameshift(TimeDelta::from_chrono(
            name.signed_duration_since(FRAMESHIFT_0),
        ))
    }

    pub fn to_name(&self) -> chrono::NaiveDateTime {
        FRAMESHIFT_0 + self.delta.to_chrono()
    }

    pub fn from_julian_day(delta: TimeDelta<S>) -> Self {
        Self::from_name_delta(&JULIAN_DAY_0, delta)
    }

    pub fn to_julian_day(&self) -> TimeDelta<S> {
        self.to_name_delta(&JULIAN_DAY_0)
    }

    pub fn from_modified_julian_day(delta: TimeDelta<S>) -> Self {
        Self::from_name_delta(&MODIFIED_JULIAN_DAY_0, delta)
    }

    pub fn to_modified_julian_day(&self) -> TimeDelta<S> {
        self.to_name_delta(&MODIFIED_JULIAN_DAY_0)
    }
}

macro_rules! to_scale_helpers {
    ($Scale:ty, $to_with:ident, $to:ident) => {
        /// Convert to
        #[doc=stringify!($Scale)]
        /// timescale, using an orientation provider.
        ///
        /// See [ToScaleWith].
        pub fn $to_with<P>(&self, provider: &P) -> Option<Epoch<$Scale>>
        where
            P: Provider,
            Self: ToScaleWith<$Scale>,
        {
            self.to_scale_with(provider)
        }

        /// Convert to
        #[doc=stringify!($Scale)]
        /// timescale, statelessly.
        ///
        /// See [ToScale].
        pub fn $to(&self) -> Epoch<$Scale>
        where
            Self: ToScale<$Scale>,
        {
            self.to_scale()
        }
    };
}

impl<S> Epoch<S> {
    to_scale_helpers!(TAI, to_tai_with, to_tai);
    to_scale_helpers!(TT, to_tt_with, to_tt);
    to_scale_helpers!(GPS, to_gps_with, to_gps);
    to_scale_helpers!(UTC, to_utc_with, to_utc);
}

impl<S> std::clone::Clone for Epoch<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> std::marker::Copy for Epoch<S> {}

impl<S> std::fmt::Debug for Epoch<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Epoch").field("delta", &self.delta).finish()
    }
}

impl<S> std::fmt::Display for Epoch<S>
where
    S: Scale,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.to_name(), S::NAME))
    }
}

impl<S> Default for Epoch<S> {
    fn default() -> Self {
        Self::from_frameshift(Default::default())
    }
}

impl<S> std::cmp::PartialEq for Epoch<S> {
    fn eq(&self, other: &Self) -> bool {
        self.delta.eq(&other.delta)
    }
}

impl<S> std::cmp::Eq for Epoch<S> {}

impl<S> std::cmp::PartialOrd for Epoch<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.delta.partial_cmp(&other.delta)
    }
}

impl<S> std::cmp::Ord for Epoch<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.delta.cmp(&other.delta)
    }
}

impl<S> std::hash::Hash for Epoch<S> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.delta.hash(state)
    }
}

impl<S> std::ops::Add<TimeDelta<S>> for Epoch<S> {
    type Output = Self;

    fn add(self, rhs: TimeDelta<S>) -> Self::Output {
        Self::from_frameshift(self.to_frameshift() + rhs)
    }
}

impl<S> std::ops::AddAssign<TimeDelta<S>> for Epoch<S> {
    fn add_assign(&mut self, rhs: TimeDelta<S>) {
        self.delta += rhs;
    }
}

impl<S> std::ops::Sub<TimeDelta<S>> for Epoch<S> {
    type Output = Self;

    fn sub(self, rhs: TimeDelta<S>) -> Self::Output {
        Self::from_frameshift(self.to_frameshift() - rhs)
    }
}

impl<S> std::ops::SubAssign<TimeDelta<S>> for Epoch<S> {
    fn sub_assign(&mut self, rhs: TimeDelta<S>) {
        self.delta -= rhs;
    }
}

impl<S> std::ops::Sub for Epoch<S> {
    type Output = TimeDelta<S>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.delta - rhs.delta
    }
}
