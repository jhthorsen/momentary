use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::Row;

use crate::momentary::AppState;

#[derive(Deserialize)]
struct TagsQuery {
    q: String,
}

#[get("/-/tags")]
pub async fn handler(state: web::Data<AppState>, req: HttpRequest, query_params: web::Query<TagsQuery>) -> impl Responder {
    let mut ctx = crate::template::template_context(req);
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
