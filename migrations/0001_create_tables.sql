create table users (
    id integer primary key,
    email text not null,
    name text not null
);

create unique index users___unique_idx on users (lower(email));

create table moment_tags (
    moment_id unsigned integer not null,
    kind char(1) not null,
    name varchar not null
);

create unique index moment_tags__unique_idx on moment_tags (moment_id, kind, lower(name));

create table moments (
    id integer primary key,
    user_id unsigned integer not null,
    content text not null,
    created_at timestamp not null default current_timestamp
);

create unique index moments__unique_idx on moments (user_id, lower(content));
