use super::{Scale, NANOS_PER_SEC};

// FIXME this derives *way* too strict requirements on Scale
/// A signed difference between two [Epoch][super::Epoch]s in the same [Scale].
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct TimeDelta<Scale> {
    delta: chrono::TimeDelta,
    _marker: std::marker::PhantomData<Scale>,
}

impl<S> std::clone::Clone for TimeDelta<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> std::marker::Copy for TimeDelta<S> {}

impl<S> std::fmt::Debug for TimeDelta<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // extract original construction arguments from timedelta
        let mut secs = self.delta.num_seconds();
        let mut nanos = self.delta.subsec_nanos();
        if nanos < 0 {
            secs -= 1;
            nanos += NANOS_PER_SEC as i32;
        }

        // use them directly to make Debug simpler to read
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
