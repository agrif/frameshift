use super::{Scale, NANOS_PER_SEC, SECS_PER_DAY};

/// A signed difference between two [Epoch][super::Epoch]s in the same [Scale].
pub struct TimeDelta<Scale> {
    delta: chrono::TimeDelta,
    _marker: std::marker::PhantomData<Scale>,
}

impl<S> TimeDelta<S> {
    pub const fn from_chrono(delta: chrono::TimeDelta) -> Self {
        Self {
            delta,
            _marker: std::marker::PhantomData,
        }
    }

    pub const fn to_chrono(&self) -> chrono::TimeDelta {
        self.delta
    }

    pub const fn new(secs: i64, nanos: u32) -> Option<Self> {
        match chrono::TimeDelta::new(secs, nanos) {
            Some(delta) => Some(Self::from_chrono(delta)),
            None => None,
        }
    }

    pub const fn to_raw(&self) -> (i64, u32) {
        // extract original construction arguments from timedelta
        let mut secs = self.delta.num_seconds();
        let mut nanos = self.delta.subsec_nanos();
        if nanos < 0 {
            secs -= 1;
            nanos += NANOS_PER_SEC as i32;
        }

        (secs, nanos as u32)
    }

    pub fn from_seconds(seconds: f64) -> Self {
        let secs = seconds.floor();
        let nanos = (seconds - secs) * NANOS_PER_SEC as f64;
        match Self::new(secs as i64, nanos.floor() as u32) {
            Some(delta) => delta,
            None => unreachable!("calculated nanos out of range"),
        }
    }

    pub fn to_seconds(&self) -> f64 {
        let (secs, nanos) = self.to_raw();
        secs as f64 + (nanos as f64 / NANOS_PER_SEC as f64)
    }

    pub fn from_days(days: f64) -> Self {
        Self::from_seconds(days * SECS_PER_DAY as f64)
    }

    pub fn to_days(&self) -> f64 {
        self.to_seconds() / SECS_PER_DAY as f64
    }
}

impl<S> std::clone::Clone for TimeDelta<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> std::marker::Copy for TimeDelta<S> {}

impl<S> std::fmt::Debug for TimeDelta<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (secs, nanos) = self.to_raw();

        f.debug_struct("TimeDelta")
            .field("secs", &secs)
            .field("nanos", &nanos)
            .field("scale", &std::any::type_name::<S>())
            .finish()
    }
}

impl<S> std::fmt::Display for TimeDelta<S>
where
    S: Scale,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.delta, S::NAME))
    }
}

impl<S> Default for TimeDelta<S> {
    fn default() -> Self {
        Self::from_chrono(Default::default())
    }
}

impl<S> std::convert::From<chrono::TimeDelta> for TimeDelta<S> {
    fn from(value: chrono::TimeDelta) -> Self {
        Self::from_chrono(value)
    }
}

impl<S> std::convert::Into<chrono::TimeDelta> for TimeDelta<S> {
    fn into(self) -> chrono::TimeDelta {
        self.to_chrono()
    }
}

impl<S> std::cmp::PartialEq for TimeDelta<S> {
    fn eq(&self, other: &Self) -> bool {
        self.delta.eq(&other.delta)
    }
}

impl<S> std::cmp::Eq for TimeDelta<S> {}

impl<S> std::cmp::PartialOrd for TimeDelta<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.delta.partial_cmp(&other.delta)
    }
}

impl<S> std::cmp::Ord for TimeDelta<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.delta.cmp(&other.delta)
    }
}

impl<S> std::hash::Hash for TimeDelta<S> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.delta.hash(state)
    }
}

impl<S> std::ops::Add for TimeDelta<S> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_chrono(self.delta + rhs.delta)
    }
}

impl<S> std::ops::AddAssign for TimeDelta<S> {
    fn add_assign(&mut self, rhs: Self) {
        self.delta += rhs.delta;
    }
}

impl<S> std::ops::Sub for TimeDelta<S> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_chrono(self.delta - rhs.delta)
    }
}

impl<S> std::ops::SubAssign for TimeDelta<S> {
    fn sub_assign(&mut self, rhs: Self) {
        self.delta -= rhs.delta;
    }
}

impl<S> std::ops::Neg for TimeDelta<S> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_chrono(-self.delta)
    }
}
