use paperclip_actix::web;

mod instances_views;

pub fn configure_admin_web_page(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/instances").configure(instances_views::configure_admin_instance_views),
    );
}
