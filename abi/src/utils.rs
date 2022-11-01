use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use crate::error::Error;

pub fn convert_to_utc(ts: &Option<Timestamp>) -> Result<DateTime<Utc>, Error> {
    if ts.is_none() {
        return Err(Error::InvalidTime);
    }
    let ts = ts.as_ref().unwrap();

    Ok(DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(
            ts.seconds,
            ts.nanos as _),
        Utc,
    ))
}

pub fn convert_to_timestamp(dt: &DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}
