## 预定系统

## run the test

```shell
cargo install cargo-nextest

cargo nextest run

cargo nextest run --nocapture
```

### database

```postgresql
create schema rsvp;

-- type
create type rsvp.reservation_status as enum ('unknown', 'pending','confirmed','blocked');
create type rsvp.reservation_update_type as enum ('unknown', 'create','update','delete');
-- reservation
create table rsvp.reservations
(
    id          uuid                    not null default gen_random_uuid(),
    user_id     varchar(64)             not null,
    status      rsvp.reservation_status not null default 'pending',
    resource_id varchar(64)             not null,
    timespan    tstzrange               not null,
    note        text                    null,
    constraint reservations_pkey primary key (id),
    constraint reservations_conflict exclude using gist(resource_id with =,timespan with &&)
);

create index reservations_resource_id_idx on rsvp.reservations (resource_id);
create index reservations_user_id_idx on rsvp.reservations (user_id);

create or replace function rsvp.query(uid text, rid text, during tstzrange)
    returns table
            (
                "like" rsvp.reservations
            )
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
```

