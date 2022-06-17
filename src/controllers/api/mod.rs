use paperclip::actix::web;
mod instances;
mod users;

pub fn configure_api(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/instances").configure(instances::configure_instances));
    cfg.service(web::scope("/users").configure(users::configure_users));
}
