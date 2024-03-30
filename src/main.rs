use actix_web::{web, App, HttpServer};

mod momentary;
mod db;
mod template;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string());
    let pool = match db::build_db(&database_url).await {
        Ok(pool) => { println!("✅ Connected to migrated database {}", database_url); pool },
        Err(e) => { println!("🔥 Failed to connect to the database {}: {:?}", database_url, e); std::process::exit(1); },
    };

    let port :u16 = std::env::var("PORT").unwrap_or("8000".to_string()).parse().unwrap();
    println!("🚀 Listening to http://127.0.0.1:{}/ ...", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(crate::momentary::AppState {
                db: pool.clone(),
                tera: crate::template::build_tera().clone(),
            }))
            .configure(services::configure)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
