use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;
use sqlx::Row;
use abi::{Reservation, ReservationQuery};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: Reservation) -> Result<Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let mut result = rsvp.clone();

        let start = abi::into_date_time(rsvp.start.unwrap());
        let end = abi::into_date_time(rsvp.end.unwrap());

        if start <= end {
            return Err(ReservationError::InvalidTime);
        }

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        // execute sql
        let id: i64 = sqlx::query(r#"INSERT INTO reservation (user_id, resource_id, timespan, note, status)
         VALUES ($1, $2, $3, $4, $5) RETURNING id"#)
            .bind(rsvp.user_id)
            .bind(rsvp.resource_id)
            .bind(timespan)
            .bind(rsvp.note)
            .bind(rsvp.status)
            .fetch_one(&self.pool)
            .await?
            .get(0);

        result.id = id;

        Ok(result)
    }

    async fn change_status(&self, _id: ReservationId) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(&self, _id: ReservationId, _note: String) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<Reservation, ReservationError> {
        todo!()
    }

    async fn query(&self, _query: ReservationQuery) -> Result<Vec<Reservation>, ReservationError> {
        todo!()
    }
}