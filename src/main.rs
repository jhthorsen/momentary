#[macro_use] extern crate rocket;

use rocket::{Build, Rocket};
use rocket::fairing::{self, AdHoc};
use rocket::form::Form;
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_db_pools::sqlx::Row;
use rocket_dyn_templates::{Template, context};
use rocket_dyn_templates::tera::{Error, Value};

use regex::{Captures, Regex};
use serde::{Serialize, Deserialize};
use serde_json::value::to_value;
use std::collections::{HashMap, HashSet};

#[derive(Database)]
#[database("moments_db")]
struct Db(sqlx::SqlitePool);

#[derive(FromForm, Serialize, Deserialize, Debug)]
struct Moment {
    user_id: i32,
    content: String,
}

#[post("/moment", data = "<moment>")]
async fn route_moment_create(mut db: Connection<Db>, moment: Form<Moment>) -> Template {
    let hashtags = extract_hashtags(&moment.content);

    let res = sqlx::query("insert into moments (user_id, content) values (?, ?)")
        .bind(moment.user_id)
        .bind(moment.content.clone())
        .execute(&mut **db).await;

    let moment_id = match res {
        Ok(r) => r.last_insert_rowid(),
        Err(e) => return Template::render("error", context! {error: e.to_string()}),
    };

    for tag in hashtags {
        let _ = sqlx::query("insert into moment_tags (moment_id, tag) values (?, ?)")
            .bind(moment_id)
            .bind(tag)
            .execute(&mut **db).await;
    }

    Template::render("moment/create", context! {
        moment: moment.into_inner(),
    })
}

#[get("/")]
async fn route_index(mut db: Connection<Db>) -> Template {
    let moments = sqlx::query("select user_id, content from moments order by created_at desc")
        .map(|row: sqlx::sqlite::SqliteRow| Moment {
            user_id: row.get::<i32, _>("user_id"),
            content: row.get::<String, _>("content"),
        })
        .fetch_all(&mut **db).await;

    if let Err(e) = moments {
        return Template::render("error", context! {error: e.to_string()});
    }

    Template::render("index", context! {
        moments: moments.unwrap(),
        moment_user_id: 1,
        moment_content: "",
    })
}

fn extract_hashtags(text: &str) -> HashSet<&str> {
    let re = Regex::new(r"(\#|@)([a-zA-Z][0-9a-zA-Z_]+)").unwrap();
    re.find_iter(text).map(|m| m.as_str()).collect()
}

fn tags_to_links(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let text = serde_json::from_value::<String>(value.clone()).unwrap();
    let re = Regex::new(r"(\#|@)([a-zA-Z][0-9a-zA-Z_]+)").unwrap();
    Ok(to_value(re.replace_all(&text, |m: &Captures| {
        format!("<a href=\"/topic/{}\">{}</a>", m[0].to_string(), m[0].to_string())
    })).unwrap())
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
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
            engines.tera.register_filter("tags_to_links", tags_to_links);
        }))
        .mount("/", routes![route_index, route_moment_create])
}
