use rusqlite::{Connection, Result};
use std::env;

fn setup_database() -> Result<Connection, rusqlite::Error> {
    let database_location = match env::var_os("MOMENTARY_DB") {
        Some(v) => v.into_string().unwrap(),
        None => "local/momentary.db".to_string(),
    };

    let conn = Connection::open(database_location)?;

    conn.execute("create table if not exists users (
        id integer primary key,
        email text not null unique
    )", [])?;
    conn.execute("create table if not exists user_friends (
        user_id unsigned integer not null,
        friend_id unsigned integer not null,
        tag varchar not null,
        constraint unique_momentary_tag unique (user_id, friend_id, tag)
    )", [])?;
    conn.execute("create table if not exists momentary_tags (
        moment_id unsigned integer not null,
        tag varchar not null,
        constraint unique_momentary_tag unique (moment_id, tag)
    )", [])?;
    conn.execute("create table if not exists moments (
        id integer primary key,
        user_id unsigned integer not null,
        content text not null,
        created_at timestamp not null default current_timestamp
    )", [])?;

    Ok(conn)
}

fn main() -> Result<()> {
    let conn = setup_database()?;
    let n_users: i32 = conn.query_row("select count(*) from users", [], |row| row.get(0))?;
    println!("Number of users: {}", n_users);
    Ok(())
}
