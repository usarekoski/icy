-- Sensors table

create table sensors(
    id serial primary key,
    description text not null
);

insert into sensors values
(1, 'sensor 1'),
(2, 'sensor 2')
;
