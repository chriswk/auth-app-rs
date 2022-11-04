use actix_web::test::read_body_json;
use actix_web::{test, App};
use auth_app_rs::model::instance::{CreateInstanceBody, InstanceRow, InstanceState};
use awc::http::StatusCode;
use paperclip::actix::web;
use sqlx::postgres::PgPoolOptions;
use testcontainers::clients;
use testcontainers::core::WaitFor;
use testcontainers::images::generic;

#[cfg(test)]
#[actix_web::test]
async fn can_add_instance() {
    let docker = clients::Cli::default();
    let postgres = docker.run(
        generic::GenericImage::new("postgres", "14-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", "authapp")
            .with_env_var("POSTGRES_PASSWORD", "example")
            .with_env_var("POSTGRES_HOST_AUTH_METHOD", "trust")
            .with_env_var("POSTGRES_USER", "tests"),
    );
    let url = format!(
        "postgres://tests:example@localhost:{}/authapp",
        postgres.get_host_port_ipv4(5432)
    );
    let migration_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Couldn't connect to database");
    auth_app_rs::migrate_db(migration_pool).await;
    let test_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Couldn't connect to database");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_pool))
            .service(web::scope("/api").configure(auth_app_rs::controllers::api::configure_api)),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/api/instances")
        .set_json(CreateInstanceBody {
            client_id: "test_instance".to_string(),
            billing_center: "eu".to_string(),
            display_name: None,
            email_domain: None,
            region: "eu".to_string(),
            plan: InstanceState::Unassigned.to_string(),
            stripe_customer_id: None,
        })
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::CREATED);
}
#[actix_web::test]
async fn can_list_added_instances() {
    let docker = clients::Cli::default();
    let postgres = docker.run(
        generic::GenericImage::new("postgres", "14-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", "authapp")
            .with_env_var("POSTGRES_PASSWORD", "example")
            .with_env_var("POSTGRES_HOST_AUTH_METHOD", "trust")
            .with_env_var("POSTGRES_USER", "tests"),
    );
    let url = format!(
        "postgres://tests:example@localhost:{}/authapp",
        postgres.get_host_port_ipv4(5432)
    );
    let migration_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Couldn't connect to database");
    auth_app_rs::migrate_db(migration_pool).await;
    let test_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Couldn't connect to database");
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_pool))
            .service(web::scope("/api").configure(auth_app_rs::controllers::api::configure_api)),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/api/instances")
        .set_json(CreateInstanceBody {
            client_id: "test_instance".to_string(),
            billing_center: "eu".to_string(),
            display_name: None,
            email_domain: None,
            region: "eu".to_string(),
            plan: "pro".to_string(),
            stripe_customer_id: None,
        })
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let req = test::TestRequest::get().uri("/api/instances").to_request();
    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success());
    let created_instance: Vec<InstanceRow> = read_body_json(res).await;
    assert_eq!(
        created_instance[0].instance_state,
        InstanceState::Unassigned.to_string()
    );
    assert_eq!(created_instance[0].plan, "pro".to_string());
}

#[actix_web::test]
async fn adding_existing_client_id_yields_conflict() {
    let docker = clients::Cli::default();
    let postgres = docker.run(
        generic::GenericImage::new("postgres", "14-alpine")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", "authapp")
            .with_env_var("POSTGRES_PASSWORD", "example")
            .with_env_var("POSTGRES_HOST_AUTH_METHOD", "trust")
            .with_env_var("POSTGRES_USER", "tests"),
    );
    let url = format!(
        "postgres://tests:example@localhost:{}/authapp",
        postgres.get_host_port_ipv4(5432)
    );
    let migration_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Couldn't connect to database");
    auth_app_rs::migrate_db(migration_pool).await;
    let test_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Couldn't connect to database");
    let req = test::TestRequest::post()
        .uri("/api/instances")
        .set_json(CreateInstanceBody {
            client_id: "test_instance".to_string(),
            billing_center: "eu".to_string(),
            display_name: None,
            email_domain: None,
            region: "eu".to_string(),
            plan: "pro".to_string(),
            stripe_customer_id: None,
        })
        .to_request();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_pool))
            .service(web::scope("/api").configure(auth_app_rs::controllers::api::configure_api)),
    )
    .await;
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let req = test::TestRequest::post()
        .uri("/api/instances")
        .set_json(CreateInstanceBody {
            client_id: "test_instance".to_string(),
            billing_center: "eu".to_string(),
            display_name: None,
            email_domain: None,
            region: "eu".to_string(),
            plan: "pro".to_string(),
            stripe_customer_id: None,
        })
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}
