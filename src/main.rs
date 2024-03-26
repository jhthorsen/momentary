#[macro_use] extern crate rocket;

use rocket::{Build, Rocket};
use rocket::fairing::{self, AdHoc};
use rocket_db_pools::{sqlx, Database};
use rocket_dyn_templates::{Template, context};

#[derive(Database)]
#[database("moments_db")]
struct Db(sqlx::SqlitePool);

#[get("/")]
fn route_index() -> Template {
    Template::render("index", context! {
        foo: 123,
    })
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
        .mount("/", routes![route_index])
        .attach(Template::fairing())
}
