use paperclip::actix::web;
mod instances;

pub fn configure_api(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/instances").configure(instances::configure_instances));
}
