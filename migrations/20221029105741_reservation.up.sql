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
