create table if not exists users (
    id integer primary key,
    email text not null unique
);
create table if not exists user_friends (
    user_id unsigned integer not null,
    friend_id unsigned integer not null,
    tag varchar not null,
    constraint unique_momentary_tag unique (user_id, friend_id, tag)
);
create table if not exists moment_tags (
    moment_id unsigned integer not null,
    tag varchar not null,
    constraint unique_momentary_tag unique (moment_id, tag)
);
create table if not exists moments (
    id integer primary key,
    user_id unsigned integer not null,
    content text not null,
    created_at timestamp not null default current_timestamp
);
