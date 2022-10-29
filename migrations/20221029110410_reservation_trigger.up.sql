-- reservation change queue
create table rsvp.reservation_changes
(
    id             serial                       not null,
    reservation_id uuid                         not null,
    op             rsvp.reservation_update_type not null
);

create or replace function rsvp.reservation_trigger() returns trigger as
$$
begin
    if TG_OP = 'insert' then
        insert into rsvp.reservation_changes(reservation_id, op) values (NEW.id, 'create');
    elsif TG_OP = 'update' then
        if OLD.status <> NEW.status then
            insert into rsvp.reservation_changes(reservation_id, op) values (NEW.id, 'update');
        end if;
    elsif TG_OP = 'delete' then
        insert into rsvp.reservation_changes(reservation_id, op) values (OLD.id, 'delete');
    end if;
    notify reservation_update;
    return NULL;
end;
$$ language plpgsql;

create trigger reservation_trigger
    after insert or update or delete
    on rsvp.reservations
    for each row
execute procedure rsvp.reservation_trigger();

