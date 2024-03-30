use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono;

use crate::momentary::{tag_re, AppState, Moment};

#[post("/moment")]
pub async fn handler(state: web::Data<AppState>, req: HttpRequest, form: web::Form<Moment>) -> impl Responder {
    let mut ctx = crate::template::template_context(req);

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
