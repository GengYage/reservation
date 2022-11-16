use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;

use abi::{Error, Reservation, ReservationQuery, ReservationStatus};

use crate::{ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, Error> {
        // 参数校验
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan()?.into();

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

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    async fn change_status(&self, id: ReservationId) -> Result<Reservation, Error> {
        let id = Uuid::parse_str(&id)
            .map_err(|_| abi::Error::InvalidResourceId(id))?;

        let rsvp: Reservation = sqlx::query_as(r#"UPDATe rsvp.reservations
        SET status = 'confirmed'
        where id = $1
        AND status = 'pending' RETURNING *"#)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(rsvp)
    }

    async fn update_note(&self, _id: ReservationId, _note: String) -> Result<Reservation, Error> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), Error> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<Reservation, Error> {
        todo!()
    }

    async fn query(&self, _query: ReservationQuery) -> Result<Vec<Reservation>, Error> {
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
    use abi::ReservationConflictInfo;
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


    #[sqlx_database_tester::test(
    pool(variable = "migrated_pool", migrations = "../migrations")
    )]
    async fn reserve_should_error_with_conflict() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp_first = Reservation::new_pending("Geng",
                                                  "ocean-view-room-714",
                                                  "2022-12-25T15:00:00-0700".parse().unwrap(),
                                                  "2022-12-28T12:00:00-0700".parse().unwrap(),
                                                  "ok");

        let rsvp_sec = Reservation::new_pending("yage",
                                                "ocean-view-room-714",
                                                "2022-12-26T15:00:00-0700".parse().unwrap(),
                                                "2022-12-28T12:00:00-0700".parse().unwrap(),
                                                "error");

        let _ = manager.reserve(rsvp_first).await.unwrap();

        // should be error
        let error = manager.reserve(rsvp_sec).await.unwrap_err();

        if let Error::ConflictError(ReservationConflictInfo::Parsed(info)) = error {
            assert_eq!(info.old.rid, "ocean-view-room-714");
            assert_eq!(info.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
            assert_eq!(info.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
        } else {
            panic!("should be panic")
        }
    }

    #[sqlx_database_tester::test(
    pool(variable = "migrated_pool", migrations = "../migrations")
    )]
    async fn reserve_change_status_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending("Geng",
                                                  "ocean-view-room-714",
                                                  "2022-12-25T15:00:00-0700".parse().unwrap(),
                                                  "2022-12-28T12:00:00-0700".parse().unwrap(),
                                                  "ok");
        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, ReservationStatus::Confirmed as i32)
    }

    #[sqlx_database_tester::test(
    pool(variable = "migrated_pool", migrations = "../migrations")
    )]
    async fn reserve_change_status_not_pending_should_do_nothing() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = Reservation::new_pending("Geng",
                                            "ocean-view-room-714",
                                            "2022-12-25T15:00:00-0700".parse().unwrap(),
                                            "2022-12-28T12:00:00-0700".parse().unwrap(),
                                            "ok");
        let rsvp = manager.reserve(rsvp).await.unwrap();

        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        let ret = manager.change_status(rsvp.id).await.unwrap_err();
        println!("{:?}", ret);
    }
}