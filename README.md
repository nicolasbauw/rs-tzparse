# TZfile data parse library

[![Current Crates.io Version](https://img.shields.io/crates/v/tzparse.svg)](https://crates.io/crates/tzparse)

This library is using the TZfile parsing library (https://crates.io/crates/libtzfile) and adds further parsing to retrieve time changes and characteristics for a given TZ.
It uses system TZfiles (default location on Linux and Macos /usr/share/zoneinfo). You can override the TZfiles default location with the TZFILES_DIR environment variable.
There are two functions described below.
See also the world-time-api (https://github.com/nicolasbauw/world-time-api)

## get_timechanges

get_timechanges("Europe/Paris", Some(2019))

The get function returns an Option enum of Vec< Timechange > for a given TZ and year (defaults to current year), output sample for Europe/Paris 2019:

[Timechange { time: 2019-03-31T01:00:00Z, gmtoff: 7200, isdst: true, abbreviation: "CEST" },
Timechange { time: 2019-10-27T01:00:00Z, gmtoff: 3600, isdst: false, abbreviation: "CET" }]

## get_zoneinfo

The get_zoneinfo function takes as input the result of the get function, and returns an Option enum of Tzinfo struct, containing convenient and human readable data about a timezone. output sample:

Tzinfo { utc_datetime: 2019-09-27T07:04:09.366157Z, datetime: 2019-09-27T09:04:09.366157+02:00, dst_from: Some(2019-03-31T01:00:00Z), dst_until: Some(2019-10-27T01:00:00Z),
raw_offset: 3600, dst_offset: 7200, utc_offset: +02:00, abbreviation: "CEST" }

Add the lib in dependencies:

````
tzparse = "0.3.3"
````

Code example:

```
extern crate tzparse;

fn main() {
    match tzparse::get_timechanges("Europe/Paris", Some(2019)) {
        Some(tz) => println!("{:?}", tzparse::get_zoneinfo(tz).unwrap()),
        None => println!("Timezone not found")
    };
}
```