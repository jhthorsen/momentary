use actix_web::web;

mod autocomplete_tags;
mod create_moment;
mod feed;
mod feed_by_tag;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(autocomplete_tags::handler);
    cfg.service(create_moment::handler);
    cfg.service(feed::handler);
    cfg.service(feed_by_tag::handler);
}
