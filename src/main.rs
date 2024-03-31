use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono;
use regex::Regex;
use serde::{Serialize, Deserialize};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tera::Tera;

mod template;
use crate::template::{build_tera, template_context};

pub struct AppState {
    db: SqlitePool,
    tera: Tera,
}

#[derive(Deserialize)]
struct TagsQuery {
    q: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Moment {
    user_id: i32,
    content: String,
    created_at: Option<String>,
}

#[post("/moment")]
async fn route_create(state: web::Data<AppState>, req: HttpRequest, form: web::Form<Moment>) -> impl Responder {
    let mut ctx = template_context(req);

    let res = sqlx::query("insert into moments (user_id, content) values (?, ?)")
        .bind(form.user_id).bind(form.content.clone())
        .execute(&state.db).await;

    if let Err(e) = res {
        ctx.insert("error", &e.to_string());
        return HttpResponse::Ok().body(state.tera.render("error.html", &ctx).unwrap());
    }

    let moment_id = res.unwrap().last_insert_rowid();

    for tag_match in tag_re().captures_iter(&form.content) {
        let res = sqlx::query("insert into moment_tags (moment_id, kind, name) values (?, ?, ?)")
            .bind(moment_id).bind(tag_match[1].to_string()).bind(tag_match[2].to_string())
            .execute(&state.db).await;

        if let Err(e) = res {
            ctx.insert("error", &e.to_string());
            return HttpResponse::Ok().body(state.tera.render("error.html", &ctx).unwrap());
        }
    }

    let mut moment = form.into_inner();
    moment.created_at = Some(chrono::Local::now().to_rfc3339());
    ctx.insert("moment", &moment);
    return HttpResponse::Ok().body(state.tera.render("moment/create.html", &ctx).unwrap());
}

#[get("/{tag}")]
async fn route_tag(state: web::Data<AppState>, req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let mut ctx = template_context(req);

    let tag = path.into_inner();
    let kind = tag[0..1].to_string();
    let name = tag[1..].to_string();
    let res = sqlx::query("select user_id, content, strftime('%FT%T', created_at) as created_at
            from moments
            join moment_tags on moment_tags.moment_id = moments.id
            where moment_tags.kind = ?
              and moment_tags.name = ?
            order by created_at desc"
        )
        .bind(kind).bind(name)
        .map(|row: sqlx::sqlite::SqliteRow| Moment {
            user_id: row.get::<i32, _>("user_id"),
            content: row.get::<String, _>("content"),
            created_at: row.get::<Option<String>, _>("created_at"),
        })
        .fetch_all(&state.db).await;

    if let Err(e) = res {
        ctx.insert("error", &e.to_string());
        return HttpResponse::Ok().body(state.tera.render("error.html", &ctx).unwrap());
    }

    ctx.insert("moments", &res.unwrap());
    return HttpResponse::Ok().body(state.tera.render("index.html", &ctx).unwrap());
}

#[get("/-/tags")]
async fn route_autocomplete_tags(state: web::Data<AppState>, req: HttpRequest, query_params: web::Query<TagsQuery>) -> impl Responder {
    let mut ctx = template_context(req);
    let kind = &query_params.q[0..1].to_string();
    let name = format!("{}%", &query_params.q[1..].to_string());
    let res = sqlx::query("select kind, name
            from moment_tags
            where kind = ? and name like ?
            group by kind, name
            limit 10"
        )
        .bind(kind).bind(name)
        .map(|row: sqlx::sqlite::SqliteRow| format!("{}{}", row.get::<String, _>("kind"), row.get::<String, _>("name")))
        .fetch_all(&state.db).await;

    if let Err(e) = res {
        ctx.insert("error", &e.to_string());
        return HttpResponse::Ok().body(state.tera.render("error.html", &ctx).unwrap());
    }

    ctx.insert("tags", &res.unwrap());
    return HttpResponse::Ok().body(state.tera.render("autocomplete/tags.html", &ctx).unwrap());
}


#[get("/")]
async fn route_feed(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let mut ctx = template_context(req);

    let res = sqlx::query("select user_id, content, strftime('%FT%T', created_at) as created_at from moments order by created_at desc")
        .map(|row: sqlx::sqlite::SqliteRow| Moment {
            user_id: row.get::<i32, _>("user_id"),
            content: row.get::<String, _>("content"),
            created_at: row.get::<Option<String>, _>("created_at"),
        })
        .fetch_all(&state.db).await;

    if let Err(e) = res {
        ctx.insert("error", &e.to_string());
        return HttpResponse::Ok().body(state.tera.render("error.html", &ctx).unwrap());
    }

    ctx.insert("moments", &res.unwrap());
    HttpResponse::Ok().body(state.tera.render("index.html", &ctx).unwrap())
}

fn tag_re() -> regex::Regex {
    Regex::new(r"(\#|@)([a-zA-Z][0-9a-zA-Z_]+)").unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string());

    let pool = match SqlitePoolOptions::new().connect(&database_url).await {
        Ok(pool) => { println!("✅ Connected to the database {}.", database_url); pool }
        Err(err) => { println!("🔥 Failed to connect to the database {}: {:?}", database_url, err); std::process::exit(1); }
    };

    let port :u16 = std::env::var("PORT").unwrap_or("8000".to_string()).parse().unwrap();
    println!("🚀 Listening to http://127.0.0.1:{}/ ...", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {db: pool.clone(), tera: build_tera().clone()}))
            .service(route_autocomplete_tags)
            .service(route_create)
            .service(route_feed)
            .service(route_tag)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
