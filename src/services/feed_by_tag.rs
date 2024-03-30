use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::Row;

use crate::momentary::{AppState, Moment};

#[get("/{tag}")]
pub async fn handler(state: web::Data<AppState>, req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let mut ctx = crate::template::template_context(req);

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
