use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::Row;

use crate::momentary::{AppState, Moment};

#[get("/")]
pub async fn handler(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let mut ctx = crate::template::template_context(req);

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
