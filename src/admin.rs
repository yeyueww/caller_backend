use sea_orm::{ColumnTrait, Set};
use std::env;
use crate::{db, AppState};
use actix_web::{HttpResponse, Responder, get, post, web, cookie, HttpRequest};
use actix_web::cookie::SameSite;
use actix_web::cookie::time::Duration;
use chrono::{Datelike, NaiveDate, Weekday};
use sea_orm::{EntityTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    account: String,
    password: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SetUnableDateRequest {
    dates: Vec<NaiveDate>,
}

#[utoipa::path(
    post,
    tag = "Admin",
    summary = "管理員登入",
    responses(
        (status = 200, description = "User login successfully"),
        (status = 400, description = "Not logged in"),
    )
)]
#[post("/user/session")]
pub async fn login(
    app_state: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> impl Responder {
    if request.account.is_empty() || request.password.is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    if request.account == app_state.account && request.password == app_state.password {
        let cache = app_state.cache.clone();
        let new_session = uuid::Uuid::new_v4().to_string();

        cache.insert(new_session.clone(),1).await;

        let cookie = cookie::Cookie::build(("session", new_session))
            .partitioned(true)
            .max_age(Duration::days(49))
            .path("/")
            .domain(env::var("DOMAIN").unwrap_or("localhost".to_string()))
            .http_only(true)
            .same_site(SameSite::None)
            .secure(true)
            .build();

        return HttpResponse::Ok().cookie(cookie).finish();
    }
    HttpResponse::Unauthorized().finish()
}

#[utoipa::path(
    get,
    tag = "Admin",
    summary = "確定使用者登入狀態",
    responses(
        (status = 200, description = "User login successfully"),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/user/session")]
pub async fn check_login_state(
    app_state: web::Data<AppState>,
    web_data: HttpRequest,
) -> impl Responder {
    let cookie = web_data.cookie("session");

    if cookie.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let cookie = cookie.unwrap();

    if app_state.cache.get(&cookie.to_string()).await.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::Ok().finish()
}

#[utoipa::path(
    post,
    tag = "Admin",
    summary = "set no service date",
    responses(
        (status = 200, description = "successfully"),
        (status = 400, description = "Not logged in"),
        (status = 500, description = "failed")
    )
)]
#[post("/date")]
pub async fn set_no_service_date(
    app_state: web::Data<AppState>,
    web_data: HttpRequest,
    request: web::Json<SetUnableDateRequest>,
) -> impl Responder {
    let cookie = web_data.cookie("session");

    if cookie.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let cookie = cookie.unwrap();

    if app_state.cache.get(&cookie.to_string()).await.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    for i in 0..request.dates.len() {
        if request.dates[i].weekday().eq(&Weekday::Sat) {
            match db::unable_date::Entity::find()
                .filter(db::unable_date::Column::Date.eq(request.dates[i].clone()))
                .one(&app_state.db_conn)
                .await {
                Ok(date) => {
                    if date.is_some() {
                        match db::unable_date::Entity::delete_by_id(date.unwrap().id).exec(&app_state.db_conn).await {
                            Ok(_) => (),
                            Err(_) => return HttpResponse::InternalServerError().finish(),
                        }
                    } else {
                        let date = db::unable_date::ActiveModel {
                            date: Set(request.dates[i].clone()),
                            ..Default::default()
                        };
                        match db::unable_date::Entity::insert(date).exec(&app_state.db_conn).await {
                            Ok(_) => (),
                            Err(_) => return HttpResponse::InternalServerError().finish(),
                        }
                    }
                },
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }
        }
    }
    HttpResponse::Ok().finish()
}