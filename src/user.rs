use crate::errors::AuthAppError;
use crate::model::{CreateUserRequest, UserListItem};
use crate::{password, AppConfig};
use actix_web::{get, post, web, Error, HttpResponse};
use sqlx::{Pool, Postgres};

#[get("/")]
async fn list_users(pool: web::Data<Pool<Postgres>>) -> Result<HttpResponse, Error> {
    let users = sqlx::query_as!(UserListItem, "SELECT email, created_at FROM users")
        .fetch_all(pool.as_ref())
        .await
        .map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Ok().json(users))
}

#[post("/")]
async fn create_user(
    create_user_request: web::Json<CreateUserRequest>,
    pool: web::Data<Pool<Postgres>>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let email = create_user_request.email.to_lowercase();
    let salt = &config.secret;
    let password = passwords::PasswordGenerator::new()
        .length(20)
        .numbers(true)
        .exclude_similar_characters(true)
        .spaces(false)
        .generate_one()
        .unwrap();
    let hash = password::hash_password(salt.clone(), password);
    sqlx::query!(
        r#"
        INSERT INTO users(email, password_hash)
        VALUES ($1, $2)
    "#,
        email,
        hash
    )
    .execute(pool.as_ref())
    .await
    .map_err(AuthAppError::SqlError)?;
    Ok(HttpResponse::Created().finish())
}

pub fn configure_user_svc(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users).service(create_user);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{
        http::{self, header::ContentType},
        test, web, App,
    };
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgPool;
    use std::future::Future;
    use testcontainers::clients::Cli;
    use testcontainers::core::WaitFor;
    use testcontainers::{clients, images::generic, images::postgres};

    async fn migrate_db(docker: &Cli) -> Pool<Postgres> {
        let postgres = docker.run(
            generic::GenericImage::new("postgres", "14-alpine")
                .with_wait_for(WaitFor::message_on_stderr(
                    "database system is ready to accept connections",
                ))
                .with_env_var("POSTGRES_DB", "authapp")
                .with_env_var("POSTGRES_PASSWORD", "example")
                .with_env_var("POSTGRES_HOST_AUTH_METHOD", "trust")
                .with_env_var("POSTGRES_USER", "test"),
        );
        let url = format!(
            "postgres://test:example@localhost:{}/authapp",
            postgres.get_host_port(5432)
        );
        let pg = PgPoolOptions::new()
            .max_connections(1)
            .connect(url.clone().as_str())
            .await
            .expect("Should be able to connect to test database");
        sqlx::migrate!()
            .run(&pg)
            .await
            .expect("Should be able to perform migration");
        pg
    }

    #[actix_web::test]
    async fn list_users_ok() {
        let client = clients::Cli::default();
        let pg = migrate_db(&client).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pg.clone()))
                .app_data(web::Data::new(AppConfig {
                    port: 0,
                    database_url: String::from(""),
                    database_max_connections: 1,
                    run_mode: "test".to_string(),
                    secret: "secret123".to_string(),
                }))
                .service(create_user)
                .service(list_users),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp: Vec<UserListItem> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 0);
    }
    #[actix_web::test]
    async fn can_add_user() {
        let client = clients::Cli::default();
        let pg = migrate_db(&client).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pg.clone()))
                .app_data(web::Data::new(AppConfig {
                    port: 0,
                    database_url: String::from(""),
                    database_max_connections: 1,
                    run_mode: "test".to_string(),
                    secret: "secret123".to_string(),
                }))
                .service(create_user)
                .service(list_users),
        )
        .await;
        let create_user_req = test::TestRequest::post()
            .uri("/")
            .set_json(CreateUserRequest {
                email: String::from("test@test.com"),
            })
            .to_request();
        let create_user_res = test::call_service(&app, create_user_req).await;
        assert_eq!(create_user_res.status(), StatusCode::CREATED);

        let req = test::TestRequest::get().uri("/").to_request();
        let resp: Vec<UserListItem> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 1);
    }

    #[actix_web::test]
    async fn adding_two_users_with_same_email_yields_conflict() {
        let client = clients::Cli::default();
        let pg = migrate_db(&client).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pg.clone()))
                .app_data(web::Data::new(AppConfig {
                    port: 0,
                    database_url: String::from(""),
                    database_max_connections: 1,
                    run_mode: "test".to_string(),
                    secret: "secret123".to_string(),
                }))
                .service(create_user)
                .service(list_users),
        )
        .await;
        let create_user_req = test::TestRequest::post()
            .uri("/")
            .set_json(CreateUserRequest {
                email: String::from("test@test.com"),
            })
            .to_request();
        let create_user_res = test::call_service(&app, create_user_req).await;
        assert_eq!(create_user_res.status(), StatusCode::CREATED);

        let duplicate_user_req = test::TestRequest::post()
            .uri("/")
            .set_json(CreateUserRequest {
                email: String::from("test@test.com"),
            })
            .to_request();
        let create_user_res = test::call_service(&app, duplicate_user_req).await;
        assert_eq!(create_user_res.status(), StatusCode::CONFLICT);

        let req = test::TestRequest::get().uri("/").to_request();
        let resp: Vec<UserListItem> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 1);
    }
}
