use super::name::{FRAMESHIFT_0, JULIAN_DAY_0, MODIFIED_JULIAN_DAY_0};
use super::{FromScale, Scale, TimeDelta, ToScale, TAI, TT};

// FIXME this derives *way* too strict requirements on Scale
/// A specific instant in time, measured in a specific [Scale].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Epoch<Scale> {
    // measures delta from Jan 1, 1900 00:00:00.0 *in this scale*
    // so Jan 1, 1900 00:00 TAI is encoded as TimeDelta<TAI>::new(0, 0)
    // and Jan 1, 1900 00:00 TT is encoded as TimeDelta<TT>::new(0, 0)
    // this date is FRAMESHIFT_0
    delta: TimeDelta<Scale>,
}

impl<S> std::fmt::Display for Epoch<S>
where
    S: Scale,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.to_name(), S::NAME))
    }
}

impl<S> Epoch<S> {
    pub const fn from_frameshift(delta: TimeDelta<S>) -> Self {
        Self { delta }
    }

    pub const fn to_frameshift(&self) -> TimeDelta<S> {
        self.delta
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

    pub fn from_scale<Other>(other: &Epoch<Other>) -> Self
    where
        S: FromScale<Other>,
    {
        Self::from_frameshift(S::from_frameshift(&other.delta))
    }

    pub fn to_scale<Other>(&self) -> Epoch<Other>
    where
        S: ToScale<Other>,
    {
        Epoch::from_frameshift(S::to_frameshift(&self.delta))
    }

    pub fn to_tai(&self) -> Epoch<TAI>
    where
        S: ToScale<TAI>,
    {
        self.to_scale()
    }

    pub fn to_tt(&self) -> Epoch<TT>
    where
        S: ToScale<TT>,
    {
        self.to_scale()
    }
}
