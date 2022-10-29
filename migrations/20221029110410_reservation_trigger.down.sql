DROP TRIGGER reservation_trigger ON rsvp.reservation;
DROP FUNCTION rsvp.reservation_trigger();
DROP TABLE rsvp.reservation_changes CASCADE;