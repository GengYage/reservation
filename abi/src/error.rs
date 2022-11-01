use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("sql error")]
    SqlError(sqlx::Error),
    #[error("{0}")]
    ConflictError(String),
    #[error("invalid start or end time of the reservation")]
    InvalidTime,
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
    #[error("invalid resource id: {0}")]
    InvalidResourceId(String),
    #[error("unknown error")]
    Unknown,
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(err_dyn) => {
                let pg_error: &PgDatabaseError = err_dyn.downcast_ref();
                match (pg_error.code(), pg_error.schema(), pg_error.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictError(pg_error.detail().unwrap().to_string())
                    }

                    _ => Error::SqlError(sqlx::Error::Database(err_dyn))
                }
            }
            _ => Error::SqlError(err),
        }
    }
}