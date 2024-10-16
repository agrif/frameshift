pub const NANOS_PER_MICRO: u32 = 1_000;
pub const NANOS_PER_MILLI: u32 = 1_000_000;
pub const NANOS_PER_SEC: u32 = 1_000_000_000;

// Jan 1, 1900 CE, midnight (proleptic Gregorian calendar)
pub const FRAMESHIFT_0: chrono::NaiveDateTime = name_gregorian(1900, 1, 1, 0, 0, 0);

// Jan 1, 4713 BCE, noon (proleptic Julian calendar)
pub const JULIAN_DAY_0: chrono::NaiveDateTime = name_julian(-4712, 1, 1, 12, 0, 0);

// Nov 17, 1858 CE, midnight (proleptic Gregorian calendar)
pub const MODIFIED_JULIAN_DAY_0: chrono::NaiveDateTime = name_gregorian(1858, 11, 17, 0, 0, 0);

pub const TT_TAI_OFFSET: chrono::TimeDelta = time_delta(32, 184 * NANOS_PER_MILLI);

pub const J2000: Epoch<TT> = Epoch::from_name(name_gregorian(2000, 1, 1, 12, 0, 0));

// augh const fn
const fn time_delta(secs: i64, nanos: u32) -> chrono::TimeDelta {
    match chrono::TimeDelta::new(secs, nanos) {
        Some(delta) => delta,
        None => panic!("bad TimeDelta"),
    }
}

const fn name_gregorian(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> chrono::NaiveDateTime {
    match chrono::NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => match date.and_hms_opt(hour, min, sec) {
            Some(datetime) => datetime,
            None => panic!("bad hour, minute, or second"),
        },
        None => panic!("bad year, month, or day"),
    }
}

const fn name_julian(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> chrono::NaiveDateTime {
    use julian::Month::*;
    let month = match month {
        1 => January,
        2 => February,
        3 => March,
        4 => April,
        5 => May,
        6 => June,
        7 => July,
        8 => August,
        9 => September,
        10 => October,
        11 => November,
        12 => December,
        _ => panic!("bad month"),
    };

    match julian::Calendar::JULIAN.at_ymd(year, month, day) {
        Ok(jdate) => {
            let jdate = jdate.convert_to(julian::Calendar::GREGORIAN);
            name_gregorian(
                jdate.year(),
                jdate.month().number(),
                jdate.day(),
                hour,
                min,
                sec,
            )
        }
        Err(_) => panic!("bad year, month, or day"),
    }
}

pub trait Scale {
    const NAME: &'static str;
}

pub trait FromScale<Other>: Sized {
    fn from_frameshift(other: &TimeDelta<Other>) -> TimeDelta<Self>;
}

pub trait ToScale<Other>: Sized {
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
        TimeDelta::from_chrono(other.delta - TT_TAI_OFFSET)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct TT;

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
        TimeDelta::from_chrono(other.delta + TT_TAI_OFFSET)
    }
}

// FIXME this derives *way* too strict requirements on Scale
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

// FIXME this derives *way* too strict requirements on Scale
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
        FRAMESHIFT_0 + self.delta.delta
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
