//! This library's functions are used to retrieve time changes and date/time characteristics for a given TZ.
//! Based on IANA database, system timezone files and [low-level parsing library](https://crates.io/crates/libtzfile).
//!
//! There are two functions:
//!
//! `get_zoneinfo` parses the tzfile and returns a Tzinfo struct which provides useful and human-readable data about the timezone
//! and can be converted to a json string with an optional feature.
//!
//! `get_timechanges` obtains time changes for specified year, or all time changes recorded in the TZfile if no year is specified.
//!
//! Example with get_zoneinfo:
//! ```text
//! [dependencies]
//! tzparse = { version = "1.1", features=["json"] }
//! ```
//! 
//! ```text
//! fn main() {
//!     println!("{}", tzparse::get_zoneinfo("/usr/share/zoneinfo/Europe/Paris").unwrap().to_json().unwrap());
//! }
//! ```
//!
//! Outputs:
//! ```text
//! {"timezone":"Europe/Paris","utc_datetime":"2020-01-22T14:12:36.792898Z","datetime":"2020-01-22T15:12:36.792898+01:00",
//! "dst_from":"2020-03-29T01:00:00Z","dst_until":"2020-10-25T01:00:00Z","dst_period":false,"raw_offset":3600,
//! "dst_offset":7200,"utc_offset":"+01:00","abbreviation":"CET","week_number":4}
//! ```
//! The get_timechanges function for Europe/Paris in 2019 returns:
//! ```text
//! [Timechange { time: 2019-03-31T01:00:00Z, gmtoff: 7200, isdst: true, abbreviation: "CEST" },
//! Timechange { time: 2019-10-27T01:00:00Z, gmtoff: 3600, isdst: false, abbreviation: "CET" }]
//! ```
//!

use chrono::prelude::*;
pub use libtzfile::TzError;
#[cfg(feature = "json")]
use serde::Serialize;

#[cfg(feature = "json")]
mod offset_serializer {
    use serde::Serialize;
    fn offset_to_json(t: chrono::FixedOffset) -> String {
        format!("{:?}", t)
    }

    pub fn serialize<S: serde::Serializer>(
        time: &chrono::FixedOffset,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        offset_to_json(time.clone()).serialize(serializer)
    }
}

/// Convenient and human-readable informations about a timezone.
#[cfg(feature = "json")]
#[derive(Debug, Serialize)]
pub struct Tzinfo {
    /// Timezone name
    pub timezone: String,
    /// UTC time
    pub utc_datetime: DateTime<Utc>,
    /// Local time
    pub datetime: DateTime<FixedOffset>,
    /// Start of DST period
    pub dst_from: Option<DateTime<Utc>>,
    /// End of DST period
    pub dst_until: Option<DateTime<Utc>>,
    /// Are we in DST period ?
    pub dst_period: bool,
    /// Normal offset to GMT, in seconds
    pub raw_offset: isize,
    /// DST offset to GMT, in seconds
    pub dst_offset: isize,
    /// current offset to GMT, in +/-HH:MM
    #[serde(with = "offset_serializer")]
    pub utc_offset: FixedOffset,
    /// Timezone abbreviation
    pub abbreviation: String,
    /// Week number
    pub week_number: i32,
}

#[cfg(not(feature = "json"))]
#[derive(Debug)]
pub struct Tzinfo {
    /// Timezone name
    pub timezone: String,
    /// UTC time
    pub utc_datetime: DateTime<Utc>,
    /// Local time
    pub datetime: DateTime<FixedOffset>,
    /// Start of DST period
    pub dst_from: Option<DateTime<Utc>>,
    /// End of DST period
    pub dst_until: Option<DateTime<Utc>>,
    /// Are we in DST period ?
    pub dst_period: bool,
    /// Normal offset to GMT, in seconds
    pub raw_offset: isize,
    /// DST offset to GMT, in seconds
    pub dst_offset: isize,
    /// current offset to GMT, in +/-HH:MM
    pub utc_offset: FixedOffset,
    /// Timezone abbreviation
    pub abbreviation: String,
    /// Week number
    pub week_number: i32,
}

/// The Timechange struct contains one timechange from the parsed TZfile.
#[derive(Debug, PartialEq)]
pub struct Timechange {
    /// The UTC time and date of the timechange, BEFORE new parameters apply
    pub time: DateTime<Utc>,
    /// The UPCOMING offset to GMT
    pub gmtoff: isize,
    /// Is upcoming change dst ?
    pub isdst: bool,
    /// TZ abbreviation of upcoming change
    pub abbreviation: String,
}

/// Transforms the Tzinfo struct to a JSON string
#[cfg(feature = "json")]
impl Tzinfo {
    pub fn to_json(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Returns year's timechanges for a timezone.
/// If year is Some(0), returns current year's timechanges.
/// If there's no timechange for selected year, returns the last occured timechange to see selected zone's applying parameters.
/// If no year (None) is specified, returns all time changes recorded in the TZfile .
pub fn get_timechanges(
    requested_timezone: &str,
    y: Option<i32>,
) -> Result<Vec<Timechange>, TzError> {
    // low-level parse of tzfile
    let timezone = libtzfile::parse(requested_timezone)?;

    // used to store timechange indices
    let mut timechanges = Vec::new();
    let mut nearest_timechange: usize = 0;

    // Used to store parsed timechanges
    let mut parsedtimechanges = Vec::new();

    // Get and store the timechange indices for requested year
    if y.is_some() {
        let d = Utc::now();
        let y = y.unwrap();
        // year = 0 ? current year is requested
        let y = if y == 0 {
            d.format("%Y").to_string().parse()?
        } else {
            y
        };
        // for year comparison
        let yearbeg = Utc.ymd(y, 1, 1).and_hms(0, 0, 0).timestamp();
        let yearend = Utc.ymd(y, 12, 31).and_hms(0, 0, 0).timestamp();
        for t in 0..timezone.tzh_timecnt_data.len() {
            if timezone.tzh_timecnt_data[t] > yearbeg && timezone.tzh_timecnt_data[t] < yearend {
                timechanges.push(t);
            }
            if timezone.tzh_timecnt_data[t] < yearbeg {
                nearest_timechange = t;
            };
        }
    } else {
        // No year requested ? stores all timechanges
        for t in 0..timezone.tzh_timecnt_data.len() {
            /* patch : chrono panics on an overflowing timestamp, and a 0xF800000000000000 timestamp is present in some Debian 10 TZfiles.*/
            if timezone.tzh_timecnt_data[t] != -576460752303423488 { timechanges.push(t) };
        }
    }

    // Populating returned Vec<Timechange>
    if timechanges.len() != 0 {
        for t in 0..timechanges.len() {
            let tc = Timechange {
                time: Utc.timestamp(timezone.tzh_timecnt_data[timechanges[t]], 0),
                gmtoff: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                    .tt_gmtoff,
                isdst: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                    .tt_isdst
                    == 1,
                abbreviation: timezone.tz_abbr[timezone.tzh_typecnt
                    [timezone.tzh_timecnt_indices[timechanges[t]] as usize]
                    .tt_abbrind as usize]
                    .to_string(),
            };
            parsedtimechanges.push(tc);
        }
    } else {
        let tc = Timechange {
            time: Utc.timestamp(timezone.tzh_timecnt_data[nearest_timechange], 0),
            gmtoff: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                .tt_gmtoff,
            isdst: timezone.tzh_typecnt[timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                .tt_isdst
                == 1,
            abbreviation: timezone.tz_abbr[timezone.tzh_typecnt
                [timezone.tzh_timecnt_indices[nearest_timechange] as usize]
                .tt_abbrind as usize]
                .to_string(),
        };
        parsedtimechanges.push(tc);
    }
    Ok(parsedtimechanges)
}

/// Returns convenient data about a timezone for current date and time.
pub fn get_zoneinfo(requested_timezone: &str) -> Result<Tzinfo, TzError> {
    let mut timezone = String::new();
    #[cfg(not(windows))]
    let mut tz: Vec<&str> = requested_timezone.split("/").collect();
    #[cfg(windows)]
    let mut tz: Vec<&str> = requested_timezone.split("\\").collect();
    // To prevent crash (case of requested directory separator unmatching OS separator)
    if tz.len() < 3 { return Err(TzError::InvalidTimezone)}
    for _ in 0..(tz.len()) - 2 {
        tz.remove(0);
    }
    if tz[0] != "zoneinfo" {
        timezone.push_str(tz[0]);
        timezone.push_str("/");
    }
    timezone.push_str(tz[1]);
    let parsedtimechanges = get_timechanges(requested_timezone, Some(0))?;
    let d = Utc::now();
    if parsedtimechanges.len() == 2 {
        // 2 times changes the same year ? DST observed
        // Are we in a dst period ? true / false
        let dst = d > parsedtimechanges[0].time && d < parsedtimechanges[1].time;
        let utc_offset = if dst == true {
            FixedOffset::east(parsedtimechanges[0].gmtoff as i32)
        } else {
            FixedOffset::east(parsedtimechanges[1].gmtoff as i32)
        };
        Ok(Tzinfo {
            timezone: timezone,
            week_number: d
                .with_timezone(&utc_offset)
                .format("%V")
                .to_string()
                .parse()?,
            utc_datetime: d,
            datetime: d.with_timezone(&utc_offset),
            dst_from: Some(parsedtimechanges[0].time),
            dst_until: Some(parsedtimechanges[1].time),
            dst_period: dst,
            raw_offset: parsedtimechanges[1].gmtoff,
            dst_offset: parsedtimechanges[0].gmtoff,
            utc_offset: utc_offset,
            abbreviation: if dst == true {
                parsedtimechanges[0].abbreviation.clone()
            } else {
                parsedtimechanges[1].abbreviation.clone()
            },
        })
    } else if parsedtimechanges.len() == 1 {
        let utc_offset = FixedOffset::east(parsedtimechanges[0].gmtoff as i32);
        Ok(Tzinfo {
            timezone: timezone,
            week_number: d
                .with_timezone(&utc_offset)
                .format("%V")
                .to_string()
                .parse()?,
            utc_datetime: d,
            datetime: d.with_timezone(&utc_offset),
            dst_from: None,
            dst_until: None,
            dst_period: false,
            raw_offset: parsedtimechanges[0].gmtoff,
            dst_offset: 0,
            utc_offset: utc_offset,
            abbreviation: parsedtimechanges[0].abbreviation.clone(),
        })
    } else {
        Err(TzError::NoData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoneinfo() {
        let tz = vec![
            Timechange {
                time: Utc.ymd(2019, 3, 31).and_hms(1, 0, 0),
                gmtoff: 7200,
                isdst: true,
                abbreviation: String::from("CEST"),
            },
            Timechange {
                time: Utc.ymd(2019, 10, 27).and_hms(1, 0, 0),
                gmtoff: 3600,
                isdst: false,
                abbreviation: String::from("CET"),
            },
        ];
        assert_eq!(
            get_timechanges("/usr/share/zoneinfo/Europe/Paris", Some(2019)).unwrap(),
            tz
        );
    }
}
