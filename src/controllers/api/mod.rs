use paperclip::actix::web;
mod google_auth;
mod instances;
mod users;

pub fn configure_api(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth/google").configure(google_auth::configure_google_auth));
    cfg.service(web::scope("/instances").configure(instances::configure_instances));
    cfg.service(web::scope("/users").configure(users::configure_users));
}
