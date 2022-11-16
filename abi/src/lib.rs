pub use pb::*;

mod error;
mod pb;
mod types;
mod utils;

pub use error::{Error, ReservationConflictInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}