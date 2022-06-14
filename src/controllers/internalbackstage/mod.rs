use paperclip::actix::web;

mod health;
mod version;

pub fn configure_internal_backstage(cfg: &mut web::ServiceConfig) {
    cfg.service(health::healthy).service(version::version);
}
