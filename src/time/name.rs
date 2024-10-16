//! Constants that refer to common reference names.

use chrono::NaiveDateTime;

// helper functions, mostly because const unwrap() is not stable

pub(super) const fn name_gregorian(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> NaiveDateTime {
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
) -> NaiveDateTime {
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

/// The reference date for this crate.
///
/// January 1, 1900 CE at 00:00 (proleptic Gregorian).
pub const FRAMESHIFT_0: NaiveDateTime = name_gregorian(1900, 1, 1, 0, 0, 0);

/// Julian day 0.
///
/// January 1, 4713 BCE at 12:00 (proleptic Julian).
pub const JULIAN_DAY_0: NaiveDateTime = name_julian(-4712, 1, 1, 12, 0, 0);

/// Modified Julian day 0.
///
/// November 17, 1858 CE at 00:00 (proleptic Gregorian).
pub const MODIFIED_JULIAN_DAY_0: NaiveDateTime = name_gregorian(1858, 11, 17, 0, 0, 0);
