#[macro_use] extern crate rocket;

use rocket::{Build, Rocket};
use rocket::fairing::{self, AdHoc};
use rocket::form::Form;
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_db_pools::sqlx::Row;
use rocket_dyn_templates::{Template, context};
use rocket_dyn_templates::tera::{Error, Value};

use chrono;
use regex::{Captures, Regex};
use serde::{Serialize, Deserialize};
use serde_json::value::to_value;
use std::collections::HashMap;

#[derive(Database)]
#[database("moments_db")]
struct Db(sqlx::SqlitePool);

#[derive(FromForm, Serialize, Deserialize, Debug)]
struct Moment {
    user_id: i32,
    content: String,
    created_at: Option<String>,
}

fn tag_re() -> regex::Regex {
    Regex::new(r"(\#|@)([a-zA-Z][0-9a-zA-Z_]+)").unwrap()
}

#[post("/moment", data = "<moment>")]
async fn route_create(mut db: Connection<Db>, moment: Form<Moment>) -> Template {
    let res = sqlx::query("insert into moments (user_id, content) values (?, ?)")
        .bind(moment.user_id).bind(moment.content.clone())
        .execute(&mut **db).await;

    if let Err(e) = res {
        return Template::render("error", context! {error: e.to_string()});
    }

    let moment_id = res.unwrap().last_insert_rowid();

    for tag_match in tag_re().captures_iter(&moment.content) {
        let res = sqlx::query("insert into moment_tags (moment_id, kind, name) values (?, ?, ?)")
            .bind(moment_id).bind(tag_match[1].to_string()).bind(tag_match[2].to_string())
            .execute(&mut **db).await;

        if let Err(e) = res {
            return Template::render("error", context! {error: e.to_string()});
        }
    }

    let mut moment = moment.into_inner();
    moment.created_at = Some(chrono::Local::now().to_rfc3339());

    Template::render("moment/create", context! {moment: moment})
}

#[get("/")]
async fn route_feed(mut db: Connection<Db>) -> Template {
    let res = sqlx::query("select user_id, content, strftime('%FT%T', created_at) as created_at from moments order by created_at desc")
        .map(|row: sqlx::sqlite::SqliteRow| Moment {
            user_id: row.get::<i32, _>("user_id"),
            content: row.get::<String, _>("content"),
            created_at: row.get::<Option<String>, _>("created_at"),
        })
        .fetch_all(&mut **db).await;

    if let Err(e) = res {
        return Template::render("error", context! {error: e.to_string()});
    }

    Template::render("index", context! {moments: res.unwrap()})
}

fn template_filter_tags_to_links(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let text = serde_json::from_value::<String>(value.clone()).unwrap();
    Ok(to_value(tag_re().replace_all(&text, |m: &Captures| {
        format!("<a href=\"/tag/{}/{}\">{}{}</a>", m[1].to_string(), m[2].to_string(), m[1].to_string(), m[2].to_string())
    })).unwrap())
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        }
        None => Err(rocket),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .attach(Template::custom(|engines| {
            engines.tera.register_filter("tags_to_links", template_filter_tags_to_links);
        }))
        .mount("/", routes![route_feed, route_create])
}
