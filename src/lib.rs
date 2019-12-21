//! This library's functions are used to retrieve time changes and date/time characteristics for a given TZ.
//! Based on data provided by system timezone files and [low-level parsing library](https://crates.io/crates/libtzfile).
//! System TZfiles default location can be overriden with the TZFILES_DIR environment variable.
//!
//! There are two functions, one using the other's result:
//!
//! `get_timechanges` obtains time changes for specified year, or all time changes recorded in the TZfile if no year is specified.
//!
//! `get_zoneinfo` further parses the data to provide useful and human-readable output.
//!
//! Example with get_zoneinfo:
//! ```
//! extern crate tzparse;
//!
//! fn main() {
//!     match tzparse::get_timechanges("Europe/Paris", Some(2019)) {
//!         Some(tz) => println!("{:?}", tzparse::get_zoneinfo(&tz).unwrap()),
//!         None => println!("Timezone not found")
//!     };
//! }
//! ```
//!
//! Outputs:
//! ```text
//! { utc_datetime: 2019-09-27T07:04:09.366157Z, datetime: 2019-09-27T09:04:09.366157+02:00,
//! dst_from: Some(2019-03-31T01:00:00Z), dst_until: Some(2019-10-27T01:00:00Z),
//! raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST" }
//! ```
//! The get_timechanges used alone ouputs:
//! ```text
//! [Timechange { time: 2019-03-31T01:00:00Z, gmtoff: 7200, isdst: true, abbreviation: "CEST" },
//! Timechange { time: 2019-10-27T01:00:00Z, gmtoff: 3600, isdst: false, abbreviation: "CET" }]
//! ```

extern crate libtzfile;
use chrono::prelude::*;
use std::convert::TryInto;

/// Convenient and human-readable informations about a timezone.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tzinfo {
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
}

/// The Timechange struct contains one timechange from the parsed TZfile.
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// Returns year's timechanges for a timezone.
/// If there's no timechange for selected year, returns the last occured timechange to see selected year's zone current parameters.
/// If no year is specified, returns all time changes recorded in the TZfile .
pub fn get_timechanges(requested_timezone: &str, y: Option<i32>) -> Option<Vec<Timechange>> {
    // low-level parse of tzfile
    let timezone = match libtzfile::parse(requested_timezone) {
        Ok(tz) => tz,
        Err(_) => return None,
    };

    // used to store timechange indices
    let mut timechanges = Vec::new();
    let mut nearest_timechange: usize = 0;

    // Used to store parsed timechanges
    let mut parsedtimechanges = Vec::new();

    // Get and store the timechange indices for requested year
    if y.is_some() {
        // for year comparison
        let yearbeg = Utc.ymd(y.unwrap(), 1, 1).and_hms(0, 0, 0);
        let yearend = Utc.ymd(y.unwrap(), 12, 31).and_hms(0, 0, 0);
        for t in 0..timezone.tzh_timecnt_data.len() {
            if timezone.tzh_timecnt_data[t] > yearbeg && timezone.tzh_timecnt_data[t] < yearend {
                timechanges.push(t);
            }
            if timezone.tzh_timecnt_data[t] < yearbeg {
                nearest_timechange = t.try_into().unwrap();
            };
        }
    } else {
        // No year requested ? stores all timechanges
        for t in 0..timezone.tzh_timecnt_data.len() {
            timechanges.push(t);
        }
    }

    if timechanges.len() != 0 {
        for t in 0..timechanges.len() {
            let tc = Timechange {
                time: timezone.tzh_timecnt_data[timechanges[t]],
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
            time: timezone.tzh_timecnt_data[nearest_timechange],
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
    Some(parsedtimechanges)
}

/// Returns convenient data about a timezone. Used for example in my [world time API](https://github.com/nicolasbauw/world-time-api).
pub fn get_zoneinfo(parsedtimechanges: &Vec<Timechange>) -> Option<Tzinfo> {
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
        Some(Tzinfo {
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
        Some(Tzinfo {
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
        None
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
                abbreviation: "CEST".to_string(),
            },
            Timechange {
                time: Utc.ymd(2019, 10, 27).and_hms(1, 0, 0),
                gmtoff: 3600,
                isdst: false,
                abbreviation: "CET".to_string(),
            },
        ];
        assert_eq!(get_timechanges("Europe/Paris", Some(2019)).unwrap(), tz);
    }
}
