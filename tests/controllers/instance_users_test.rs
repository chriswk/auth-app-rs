use std::borrow::Borrow;
use std::future::Future;

use actix_web::http::StatusCode;
use actix_web::{
    http::{self, header::ContentType},
    test, web, App,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use testcontainers::clients::Cli;
use testcontainers::core::WaitFor;
use testcontainers::{clients, images::generic, images::postgres};

use auth_app_rs::controllers::instance::{create_instance_db, NewInstanceBody};

use super::*;

async fn migrate_db(docker: &Cli) -> Pool<Postgres> {
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
        postgres.get_host_port(5432)
    );
    let pg = PgPoolOptions::new()
        .max_connections(1)
        .connect(url.clone().as_str())
        .await
        .expect("Should be able to connect to tests database");
    sqlx::migrate!()
        .run(&pg)
        .await
        .expect("Should be able to perform migration");
    pg
}

#[actix_web::test]
async fn can_add_single_user() {
    let client = clients::Cli::default();
    let pg: Pool<Postgres> = migrate_db(&client).await;
    let app =
        test::init_service(App::new().app_data(web::Data::new(pg.clone())).service(
            web::scope("/").configure(
                auth_app_rs::controllers::instance_users::configure_instance_user_service,
            ),
        ))
        .await;
    let req = test::TestRequest::post()
        .uri("/client1/add")
        .set_json(auth_app_rs::controllers::instance_users::NewUserBody {
            email: "test@test.com".to_string(),
            role: "admin".to_string(),
        })
        .to_request();
    let add_user_res = test::call_service(&app, req).await;
    assert_eq!(add_user_res.status(), StatusCode::CREATED);
}

#[actix_web::test]
async fn adding_same_user_to_same_client_yields_conflict() {
    let client = clients::Cli::default();
    let pg: Pool<Postgres> = migrate_db(&client).await;
    let app =
        test::init_service(App::new().app_data(web::Data::new(pg.clone())).service(
            web::scope("/").configure(
                auth_app_rs::controllers::instance_users::configure_instance_user_service,
            ),
        ))
        .await;
    let req = test::TestRequest::post()
        .uri("/client1/add")
        .set_json(auth_app_rs::controllers::instance_users::NewUserBody {
            email: "test@test.com".to_string(),
            role: "admin".to_string(),
        })
        .to_request();
    let add_user_res = test::call_service(&app, req).await;
    assert_eq!(add_user_res.status(), StatusCode::CREATED);
    let repeated_req = test::TestRequest::post()
        .uri("/client1/add")
        .set_json(auth_app_rs::controllers::instance_users::NewUserBody {
            email: "test@test.com".to_string(),
            role: "admin".to_string(),
        })
        .to_request();
    let add_user_again = test::call_service(&app, repeated_req).await;
    assert_eq!(add_user_again.status(), StatusCode::CONFLICT);
}

#[actix_web::test]
async fn adding_same_user_to_different_client_is_ok() {
    let client = clients::Cli::default();
    let pg: Pool<Postgres> = migrate_db(&client).await;
    create_instance_db(
        NewInstanceBody {
            client_id: "client1".to_string(),
            region: "eu".to_string(),
            plan: "pro".to_string(),
            display_name: None,
            email_domain: None,
        },
        pg.borrow(),
    )
    .await
    .unwrap();
    create_instance_db(
        NewInstanceBody {
            client_id: "client5".to_string(),
            region: "eu".to_string(),
            plan: "pro".to_string(),
            display_name: None,
            email_domain: None,
        },
        pg.borrow(),
    )
    .await
    .unwrap();
    let app =
        test::init_service(App::new().app_data(web::Data::new(pg.clone())).service(
            web::scope("/").configure(
                auth_app_rs::controllers::instance_users::configure_instance_user_service,
            ),
        ))
        .await;
    let req = test::TestRequest::post()
        .uri("/client1/add")
        .set_json(auth_app_rs::controllers::instance_users::NewUserBody {
            email: "test@test.com".to_string(),
            role: "admin".to_string(),
        })
        .to_request();
    let add_user_res = test::call_service(&app, req).await;
    assert_eq!(add_user_res.status(), StatusCode::CREATED);
    let repeated_req = test::TestRequest::post()
        .uri("/client5/add")
        .set_json(auth_app_rs::controllers::instance_users::NewUserBody {
            email: "test@test.com".to_string(),
            role: "admin".to_string(),
        })
        .to_request();
    let add_user_again = test::call_service(&app, repeated_req).await;
    assert_eq!(add_user_again.status(), StatusCode::CONFLICT);
}
