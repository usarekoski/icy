-- Measurements table.
-- Missing measurements values are null.

create table measurements(
    id serial primary key,
    timestamp timestamp with time zone not null,
    sensor_id integer references sensors(id) not null,
    temperature float,
    pressure float
);
