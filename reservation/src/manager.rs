use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;
use abi::{Reservation, ReservationQuery, ReservationStatus};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let start = abi::convert_to_datatime(rsvp.start.clone().unwrap());
        let end = abi::convert_to_datatime(rsvp.end.clone().unwrap());

        if start >= end {
            return Err(ReservationError::InvalidTime);
        }

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let status = ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(ReservationStatus::Pending);


        // execute sql
        let id: Uuid = sqlx::query(r#"INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status)
         VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"#)
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(status.to_string())
            .fetch_one(&self.pool)
            .await?
            .get(0);

        println!("{}", id);
        rsvp.id = id.to_string();

        Ok(rsvp)
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

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod test {
    use chrono::FixedOffset;
    use super::*;

    #[sqlx_database_tester::test(
    pool(variable = "migrated_pool", migrations = "../migrations")
    )]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-25T15:00:00-0700".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-28T12:00:00-0700".parse().unwrap();

        let rsvp = Reservation::new_pending("gyg", "ocean-view-room-713", start, end, "I'll arrive at 3pm. Please help to upgrade to executive room if possible.");

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert_ne!(rsvp.id, "")
    }
}