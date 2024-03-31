use actix_web::HttpRequest;
use actix_web::http::header::HeaderValue;
use regex::{Captures, Regex};
use serde::{Serialize, Deserialize};
use serde_json::value::to_value;
use std::collections::HashMap;
use tera::{Context, Error, Tera, Value};

#[derive(Serialize, Deserialize, Debug)]
struct HtmxHeaders {
    boosted: bool,
    history_restore_request: bool,
    request: bool,
    target: String,
    trigger: String,
    trigger_name: String,
}

pub fn build_tera() -> Tera {
    let mut tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    tera.register_filter("tags_to_links", template_filter_tags_to_links);
    return tera;
}

fn header_value_to_string(value: Option<&HeaderValue>) -> String {
    value.unwrap_or(&HeaderValue::from_static("")).to_str().unwrap_or_default().to_string()
}

fn tag_re() -> regex::Regex {
    Regex::new(r"(\#|@)([a-zA-Z][0-9a-zA-Z_]+)").unwrap()
}

pub fn template_context(req: HttpRequest) -> Context {
    let mut ctx = Context::new();
    let h = req.headers();
    ctx.insert("htmx", &HtmxHeaders {
        boosted: h.get("HX-Boosted").is_some(),
        history_restore_request: h.get("HX-History-Restore-Request").is_some(),
        request: h.get("HX-Request").is_some(),
        target: header_value_to_string(h.get("HX-Target")),
        trigger: header_value_to_string(h.get("HX-Trigger")),
        trigger_name: header_value_to_string(h.get("HX-Trigger-Name")),
    });

    ctx
}

fn template_filter_tags_to_links(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let text = serde_json::from_value::<String>(value.clone()).unwrap();
    Ok(to_value(tag_re().replace_all(&text, |m: &Captures| {
        format!("<a href=\"/{}\">{}</a>",
            Regex::new(r"#").unwrap().replace(&mut m[0].to_string(), "%23"),
            m[0].to_string())
    })).unwrap())
}
