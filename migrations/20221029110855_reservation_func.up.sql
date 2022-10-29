create or replace function rsvp.query(uid text, rid text, during tstzrange)
    returns table("like" rsvp.reservations)
as
$$
BEGIN
    if uid is null and uid is null then
        return query select * from rsvp.reservations where timespan && during;
    elsif uid is null then
        return query select *
                     from rsvp.reservations
                     where resource_id = rid
                       and during @> timespan;
    elsif rid is null then
        return query select *
                     from rsvp.reservations
                     where user_id = uid
                       and during @> timespan;
    else
        return query select *
                     from rsvp.reservations
                     where resource_id = uid
                       and user_id = uid
                       and during @> timespan;
    end if;
END;

$$ language plpgsql;