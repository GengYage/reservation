use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("sql error")]
    SqlError(#[from] sqlx::Error),
    #[error("invalid start or end time for the reservation")]
    InvalidTime,
    #[error("unknown error")]
    Unknown,
}