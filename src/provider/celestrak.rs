//! Celestrak orientation provider.
//!
//! This provider reads the [CSV files produced by
//! Celestrak][celestrak] to provide Earth orientation data and leap
//! second info.
//!
//! [celestrak]: https://celestrak.org/SpaceData/
//!
//! Celestrak produces [a file containing all data so far][long] as
//! well as [one for only the last 5 years][short].
//!
//! [long]: https://celestrak.org/SpaceData/EOP-All.csv
//! [short]: https://celestrak.org/SpaceData/EOP-Last5Years.csv

use std::io::BufRead;

use crate::time::{Epoch, TimeDelta, TAI, UT1, UTC};

#[derive(Debug, Clone)]
pub struct CelestrakProvider {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    /// Measurement time.
    pub time_utc: Epoch<UTC>,

    /// Arc-seconds.
    pub x: f64,

    /// Arc-seconds.
    pub y: f64,

    /// UT1 - UTC, seconds.
    pub ut1_utc: f64,

    /// Length of day, seconds.
    pub lod: f64,

    /// Arc-seconds.
    pub dpsi: f64,

    /// Arc-seconds.
    pub deps: f64,

    /// Arc-seconds.
    pub dx: f64,

    /// Arc-seconds.
    pub dy: f64,

    /// TAI - UTC, seconds.
    pub tai_utc: i64,

    /// Is this entry observed or predicted?
    pub data_type: Type,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Observed,
    Predicted,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error reading CSV data")]
    Read(#[from] std::io::Error),
    #[error("the CSV file does not have a header")]
    MissingHeader,
    #[error("the CSV file does not contain a {0} column")]
    MissingColumn(&'static str),
    #[error("row {0} of the CSV file is missing column {1}")]
    MissingField(usize, &'static str),
    #[error("column {1} of row {0} of the CSV file failed to parse")]
    BadParse(usize, &'static str),
}

macro_rules! find_column {
    ($header:expr, $name:expr) => {
        $header
            .clone()
            .position(|s| s == $name)
            .ok_or_else(|| Error::MissingColumn($name))
    };
}

macro_rules! get_column {
    ($rowi:expr, $row:expr, $i:expr, $name:expr, $parse:expr) => {
        $row.get($i)
            .ok_or_else(|| Error::MissingField($rowi, $name))
            .and_then(|s| ($parse)(s).map_err(|_| Error::BadParse($rowi, $name)))
    };

    ($rowi:expr, $row:expr, $i:expr, $name:expr) => {
        get_column!($rowi, $row, $i, $name, |s: &str| s.parse())
    };
}

impl CelestrakProvider {
    pub fn from_csv<R>(file: R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        let mut lines = std::io::BufReader::new(file).lines();

        let header_line = lines.next().ok_or_else(|| Error::MissingHeader)??;
        let header = header_line.split(',');

        let i_time_utc = find_column!(header, "MJD")?;
        let i_x = find_column!(header, "X")?;
        let i_y = find_column!(header, "Y")?;
        let i_ut1_utc = find_column!(header, "UT1-UTC")?;
        let i_lod = find_column!(header, "LOD")?;
        let i_dpsi = find_column!(header, "DPSI")?;
        let i_deps = find_column!(header, "DEPS")?;
        let i_dx = find_column!(header, "DX")?;
        let i_dy = find_column!(header, "DY")?;
        let i_tai_utc = find_column!(header, "DAT")?;
        let i_data_type = find_column!(header, "DATA_TYPE")?;

        let mut entries = Vec::new();
        for line in lines {
            let line = line?;
            let rowi = entries.len();
            let row: Vec<&str> = line.split(',').collect();

            let entry = Entry {
                time_utc: Epoch::from_modified_julian_day(TimeDelta::from_days(get_column!(
                    rowi, row, i_time_utc, "MJD"
                )?)),
                x: get_column!(rowi, row, i_x, "X")?,
                y: get_column!(rowi, row, i_y, "Y")?,
                ut1_utc: get_column!(rowi, row, i_ut1_utc, "UT1-UTC")?,
                lod: get_column!(rowi, row, i_lod, "LOD")?,
                dpsi: get_column!(rowi, row, i_dpsi, "DPSI")?,
                deps: get_column!(rowi, row, i_deps, "DEPS")?,
                dx: get_column!(rowi, row, i_dx, "DX")?,
                dy: get_column!(rowi, row, i_dy, "DY")?,
                tai_utc: get_column!(rowi, row, i_tai_utc, "DAT")?,

                data_type: get_column!(rowi, row, i_data_type, "DATA_TYPE")?,
            };

            entries.push(entry);
        }

        Ok(Self::from_entries(entries))
    }

    pub fn from_entries(mut entries: Vec<Entry>) -> Self {
        entries.sort_by_key(|e| e.time_utc);
        Self { entries }
    }

    pub fn get_utc(&self, t: &Epoch<UTC>) -> Option<Entry> {
        let idx = self.entries.iter().position(|e| e.time_utc > *t)?;

        if idx == 0 {
            return None;
        }

        Some(self.entries[idx - 1].lerp(&self.entries[idx], *t - self.entries[idx].time_utc))
    }

    pub fn get_tai(&self, t: &Epoch<TAI>) -> Option<Entry> {
        let idx = self.entries.iter().position(|e| e.time_tai() > *t)?;

        if idx == 0 {
            return None;
        }

        // FIXME this lerp is a little suspicious
        Some(self.entries[idx - 1].lerp(&self.entries[idx], *t - self.entries[idx].time_tai()))
    }

    pub fn get_ut1(&self, t: &Epoch<UT1>) -> Option<Entry> {
        let idx = self.entries.iter().position(|e| e.time_ut1() > *t)?;

        if idx == 0 {
            return None;
        }

        // FIXME this lerp is *extremely* suspicious
        Some(self.entries[idx - 1].lerp(&self.entries[idx], *t - self.entries[idx].time_ut1()))
    }
}

impl Entry {
    pub fn time_tai(&self) -> Epoch<TAI> {
        self.time_utc.transmute() + TimeDelta::new(self.tai_utc, 0).unwrap()
    }

    pub fn time_ut1(&self) -> Epoch<UT1> {
        self.time_utc.transmute() + TimeDelta::from_seconds(self.ut1_utc)
    }

    fn lerp<S>(&self, other: &Self, t: TimeDelta<S>) -> Self {
        let g1 = t.to_seconds() / (other.time_utc - self.time_utc).to_seconds();
        let g0 = 1.0 - g1;

        Self {
            // integral, don't even try to interpolate
            // keep time_utc as a note of when tai_utc takes effect
            tai_utc: self.tai_utc,
            time_utc: self.time_utc,

            // interpolate
            x: g0 * self.x + g1 * other.x,
            y: g0 * self.y + g1 * other.y,
            ut1_utc: g0 * self.ut1_utc + g1 * other.ut1_utc,
            lod: g0 * self.lod + g1 * other.lod,
            dpsi: g0 * self.dpsi + g1 * other.dpsi,
            deps: g0 * self.deps + g1 * other.deps,
            dx: g0 * self.dx + g1 * other.dx,
            dy: g0 * self.dy + g1 * other.dy,

            data_type: self.data_type.merge(&other.data_type),
        }
    }
}

impl Type {
    fn merge(&self, other: &Self) -> Self {
        match self {
            Self::Observed => match other {
                Self::Observed => Self::Observed,
                Self::Predicted => Self::Predicted,
            },

            Self::Predicted => Self::Predicted,
        }
    }
}

impl std::str::FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "O" => Ok(Self::Observed),
            "P" => Ok(Self::Predicted),
            _ => Err(()),
        }
    }
}

impl super::Provider for CelestrakProvider {
    fn tai_utc_for_utc(&self, epoch: &Epoch<UTC>) -> Option<TimeDelta<TAI>> {
        TimeDelta::new(self.get_utc(epoch)?.tai_utc, 0)
    }

    fn tai_utc_for_tai(&self, epoch: &Epoch<TAI>) -> Option<TimeDelta<TAI>> {
        TimeDelta::new(self.get_tai(epoch)?.tai_utc, 0)
    }

    fn ut1_utc_for_utc(&self, epoch: &Epoch<UTC>) -> Option<TimeDelta<UT1>> {
        Some(TimeDelta::from_seconds(self.get_utc(epoch)?.ut1_utc))
    }

    fn ut1_utc_for_ut1(&self, epoch: &Epoch<UT1>) -> Option<TimeDelta<UT1>> {
        Some(TimeDelta::from_seconds(self.get_ut1(epoch)?.ut1_utc))
    }
}
