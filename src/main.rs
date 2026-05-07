pub mod admin;
pub mod counter;
pub mod user;
mod db;

use std::env;
use std::time::Duration;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use moka::future::Cache;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use utoipa::openapi::Contact;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;
use migration::{Migrator, MigratorTrait};
use crate::admin::{check_login_state, login, set_no_service_date};
use crate::user::{add_customer, get_customers, get_no_service_date};
use crate::counter::{get_now_counter, next_number, previous_number, set_number};

pub struct AppState {
    account: String,
    password: String,
    cache: Cache<String, u32>,
    db_conn: DatabaseConnection,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let account = env::var("ACCOUNT").expect("ACCOUNT environment variable not set");
    let password = env::var("PASSWORD").expect("PASSWORD environment variable not set");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let mut db_connect_option = ConnectOptions::new(db_url);
    db_connect_option
        .max_connections(10)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false) // disable SQLx logging
        .sqlx_logging_level(log::LevelFilter::Info);

    let database_conn = Database::connect(db_connect_option).await.unwrap();
    Migrator::up(&database_conn, None).await.unwrap();

    let cache:Cache<String,u32> = Cache::builder()
        .max_capacity(128*1024) // 128 KB
        .time_to_idle(Duration::from_hours(7*24)) //7 Days
        .build();



    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_methods("GET,POST,PUT,DELETE,OPTIONS".split(',').into_iter())
            .allowed_origin_fn(|_, _| true)
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(AppState {
                account: account.clone(),
                password: password.clone(),
                cache: cache.clone(),
                db_conn: database_conn.clone(),
            }))
            .wrap(cors)
            .into_utoipa_app()
            .service(utoipa_actix_web::scope("/api/v1").configure(config))
            .openapi_service(|mut api| {
                api.info.title = "PDS API".to_string();
                api.info.version = "1.0.0".to_string();
                api.info.description = Some("A Data Storage Service.".to_string());
                api.info.contact = Some(
                    Contact::builder()
                        .name(Some("Yeyue"))
                        .email(Some("support@yeyue.org"))
                        .build(),
                );
                api.tags = Some(vec![
                    utoipa::openapi::Tag::new("User"),
                    utoipa::openapi::Tag::new("Counter"),
                    utoipa::openapi::Tag::new("Admin"),
                ]);
                SwaggerUi::new("/api/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .into_app()
    })
        .bind(("0.0.0.0", 9650))?
        .run()
        .await
}

fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(get_customers)
        .service(add_customer)
        .service(get_now_counter)
        .service(next_number)
        .service(previous_number)
        .service(set_number)
        .service(login)
        .service(check_login_state)
        .service(set_no_service_date)
        .service(get_no_service_date)
    ;
}