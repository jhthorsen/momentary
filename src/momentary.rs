use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use tera::Tera;

pub struct AppState {
    pub db: SqlitePool,
    pub tera: Tera,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Moment {
    pub user_id: i32,
    pub content: String,
    pub created_at: Option<String>,
}

pub fn tag_re() -> regex::Regex {
    Regex::new(r"(\#|@)([a-zA-Z][0-9a-zA-Z_]+)").unwrap()
}
