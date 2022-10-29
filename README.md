## 预定系统

### database

```postgresql
create schema rsvp;

create type rsvp.reservation_status as enum ('unknown', 'pending','confirmed','blocked');
create type rsvp.reservation_update_type as enum ('unknown', 'create','update','delete');


-- reservation
create table rsvp.reservations {
	id uuid not null default uuid_generate_v4() comment 'id',
	user_id varchar(64) not null comment '用户id',
	status rsvp.reservation_status not null default 'pending' comment '订单状态',

	resource_id varchar(64) not null comment '资源id',
	timespan tstzrange not null comment '预定的时间段',

	note null,

	constraint reservations_pkey primary key (id),
	constraint reservations_conflict exclude using gist (resource_id with =, timespan with &&)
}

create index reservations_resource_id_idx on rsvp.reservations(resource_id);
create index reservations_user_id_idx on rsvp.reservations(user_id);


-- reservation change queue
create table rsvp.reservation_changes {
	id serial not null,
	reservation_id uuid not null,
	op rsvp.reservation_update_type not null
}


create or replace function rsvp.query(uid text, rid text, during: tstzrange) returns table rsvp.reservations as $$ $$ language plpgsql;

create or replace function rsvp.reservation_trigger() returns trigger as 
$$
begin
	if TG_OP = 'insert' then
		insert into rsvp.reservation_changes(reservation_id, op) values(NEW.id, 'create');
	elsif TG_OP = 'update' then
		if OLD.status <> NEW.status then
			insert into rsvp.reservation_changes(reservation_id, op) values(NEW.id, 'update');
		end if;
	elsif TG_OP = 'delete' then
		insert into rsvp.reservation_changes(reservation_id, op) values(OLD.id, 'delete');
	end if;
	notify reservation_update, NEW;
	return NULL;
end;
$$ language plpgsql;

create trigger reservation_trigger after insert or update or delete on rsvp.reservations
for each row execute procedure rsvp.reservation_trigger();


```

