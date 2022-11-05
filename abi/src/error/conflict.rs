use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    UnParsed(String),
}

#[derive(Debug, Clone)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

#[derive(Debug, Clone)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse::<ReservationConflict>() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::UnParsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan")
            .ok_or(())?.replace('"', "");

        let mut split = timespan_str.splitn(2, ',');

        let start = parse_time(split.next().ok_or(())?)?;
        let end = parse_time(split.next().ok_or(())?)?;

        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

#[derive(Debug)]
struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regular expression to parse the string
        let re = Regex::new(r#"\((?P<k1>[a-zA-Z0-9-_]+),\s*(?P<k2>[a-zA-Z0-9-_]+)\)=\((?P<v1>[a-zA-Z0-9-_]+),\s*\[(?P<v2>[^\]\)]+)\)"#).unwrap();
        let mut maps = vec![];

        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            maps.push(Some(map));
        }

        if maps.len() != 2 {
            return Err(());
        }

        // 从vec中通过Some获取元素所有权
        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

fn parse_time(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc)
    )
}

#[cfg(test)]
mod test {
    use super::*;

    const ERROR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-714, [\"2022-12-26 22:00:00+00\",\"2022-12-28 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-714, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).";

    #[test]
    fn parse_info_should_work() {
        let info: ParsedInfo = ERROR_MSG.parse().unwrap();
        assert_eq!(info.new["timespan"], "\"2022-12-26 22:00:00+00\",\"2022-12-28 19:00:00+00\"");
        assert_eq!(info.new["resource_id"], "ocean-view-room-714");
        assert_eq!(info.old["timespan"], "\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\"");
        assert_eq!(info.old["resource_id"], "ocean-view-room-714");
    }

    #[test]
    fn conflict_error_msg_should_parse() {
        let info: ReservationConflictInfo = ERROR_MSG.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-714");
                assert_eq!(conflict.new.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
                assert_eq!(conflict.old.rid, "ocean-view-room-714");
                assert_eq!(conflict.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
            }
            ReservationConflictInfo::UnParsed(_) => { panic!("should be parsed") }
        }
    }

    #[test]
    fn parse_time() {
        let time = DateTime::parse_from_str("2022-12-26 22:00:00+00", "%Y-%m-%d %H:%M:%S%#z").unwrap().with_timezone(&Utc);
        assert_eq!(time.to_rfc3339(), "2022-12-26T22:00:00+00:00")
    }
}