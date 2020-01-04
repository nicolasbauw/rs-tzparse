# tzparse

[![Current Crates.io Version](https://img.shields.io/crates/v/tzparse.svg)](https://crates.io/crates/tzparse)
[![Downloads badge](https://img.shields.io/crates/d/tzparse.svg)](https://crates.io/crates/tzparse)

This library's functions are used to retrieve time changes and date/time characteristics for a given TZ.
Based on data provided by system timezone files and [low-level parsing library](https://crates.io/crates/libtzfile).
System TZfiles default location can be overriden with the TZFILES_DIR environment variable.

There are two functions:

`get_zoneinfo` parses the tzfile to provide useful and human-readable data about the timezone.

`get_timechanges` obtains time changes for specified year, or all time changes recorded in the TZfile if no year is specified.

Example with get_zoneinfo:
```rust
fn main() {
   println!("{:?}", tzparse::get_zoneinfo("Europe/Paris").unwrap());
}
```

Outputs:
```
{ utc_datetime: 2019-09-27T07:04:09.366157Z, datetime: 2019-09-27T09:04:09.366157+02:00,
dst_from: Some(2019-03-31T01:00:00Z), dst_until: Some(2019-10-27T01:00:00Z),
raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST" }
```
The get_timechanges function for Europe/Paris in 2019 returns:
```
[Timechange { time: 2019-03-31T01:00:00Z, gmtoff: 7200, isdst: true, abbreviation: "CEST" },
Timechange { time: 2019-10-27T01:00:00Z, gmtoff: 3600, isdst: false, abbreviation: "CET" }]
```

License: GPL-3.0
