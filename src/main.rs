#[macro_use] extern crate rocket;

use rocket_dyn_templates::{Template, context};
use rusqlite::Connection;
use std::env;

#[get("/")]
fn route_index() -> Template {
    Template::render("index", context! {
        foo: 123,
    })
}

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

fn main() -> Result<(), rusqlite::Error> {
    let conn = setup_database()?;
    let n_users: i32 = conn.query_row("select count(*) from users", [], |row| row.get(0))?;
    println!("Number of users: {}", n_users);

    let rt = rocket::tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .unwrap();

    let rocket = rocket::build()
        .mount("/", routes![route_index])
        .attach(Template::fairing());

    if let Err(e) = rt.block_on(rocket.launch()) {
        println!("Launch failed! Error: {}", e);
    }

    Ok(())
}
