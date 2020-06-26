# tzparse

[![Current Crates.io Version](https://img.shields.io/crates/v/tzparse.svg)](https://crates.io/crates/tzparse)
[![Downloads badge](https://img.shields.io/crates/d/tzparse.svg)](https://crates.io/crates/tzparse)

This library's functions are used to retrieve time changes and date/time characteristics for a given TZ.
Based on IANA database, system timezone files and [low-level parsing library](https://crates.io/crates/libtzfile).

There are two functions:

`get_zoneinfo` parses the tzfile and returns a Tzinfo struct which provides useful and human-readable data about the timezone
and can be converted to a json string with an optional feature.

`get_timechanges` obtains time changes for specified year, or all time changes recorded in the TZfile if no year is specified.

Example with get_zoneinfo:
```
[dependencies]
tzparse = { version = "1.1", features=["json"] }

fn main() {
    println!("{}", tzparse::get_zoneinfo("/usr/share/zoneinfo/Europe/Paris").unwrap().to_json().unwrap());
}
```

Outputs:
```
{"timezone":"Europe/Paris","utc_datetime":"2020-01-22T14:12:36.792898Z","datetime":"2020-01-22T15:12:36.792898+01:00",
"dst_from":"2020-03-29T01:00:00Z","dst_until":"2020-10-25T01:00:00Z","dst_period":false,"raw_offset":3600,
"dst_offset":7200,"utc_offset":"+01:00","abbreviation":"CET","week_number":4}
```
The get_timechanges function for Europe/Paris in 2019 returns:
```
[Timechange { time: 2019-03-31T01:00:00Z, gmtoff: 7200, isdst: true, abbreviation: "CEST" },
Timechange { time: 2019-10-27T01:00:00Z, gmtoff: 3600, isdst: false, abbreviation: "CET" }]
```

License: GPL-3.0
