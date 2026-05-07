use sea_orm::ColumnTrait;
use crate::{db, AppState};
use actix_web::{Responder, get, post, web, HttpResponse, HttpRequest};
use sea_orm::{EntityTrait, QueryFilter, ActiveModelTrait, Set};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SetCounterRequest {
    number: i32,
}

#[utoipa::path(
    get,
    tag = "Counter",
    summary = "Get now count number",
    responses(
        (status=200, description="Ok", body=i32),
        (status=500, description="Internal server error")
    )
)]
#[get("/counter")]
pub async fn get_now_counter(
    app_state: web::Data<AppState>,
) -> impl Responder {
    let date = chrono::Utc::now().date_naive();
    let count = db::counter::Entity::find()
        .filter(db::counter::Column::Date.eq(date))
        .one(&app_state.db_conn)
        .await;
    match count {
        Ok(Some(counter)) => HttpResponse::Ok().json(counter.number),
        Ok(None) => HttpResponse::Ok().json(0),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[utoipa::path(
    post,
    tag = "Admin",
    summary = "Next number",
    responses(
        (status=200, description="Ok", body=i32),
        (status = 400, description = "Not logged in"),
        (status=500, description="Internal server error")
    )
)]
#[post("/counter/next")]
pub async fn next_number(
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

    let date = chrono::Utc::now().date_naive();
    let count = db::counter::Entity::find()
        .filter(db::counter::Column::Date.eq(date))
        .one(&app_state.db_conn)
        .await;

    match count {
        Ok(Some(counter)) => {
            let mut active_model: db::counter::ActiveModel = counter.into();
            let new_number = active_model.number.as_ref() + 1;
            active_model.number = Set(new_number);
            match active_model.update(&app_state.db_conn).await {
                Ok(m) => HttpResponse::Ok().json(m.number),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        Ok(None) => {
            let active_model = db::counter::ActiveModel {
                date: Set(date),
                number: Set(1),
                ..Default::default()
            };
            match active_model.insert(&app_state.db_conn).await {
                Ok(m) => HttpResponse::Ok().json(m.number),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[utoipa::path(
    post,
    tag = "Admin",
    summary = "Previous number",
    responses(
        (status=200, description="Ok", body=i32),
        (status = 400, description = "Not logged in"),
        (status=500, description="Internal server error")
    )
)]
#[post("/counter/previous")]
pub async fn previous_number(
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

    let date = chrono::Utc::now().date_naive();
    let count = db::counter::Entity::find()
        .filter(db::counter::Column::Date.eq(date))
        .one(&app_state.db_conn)
        .await;

    match count {
        Ok(Some(counter)) => {
            let mut active_model: db::counter::ActiveModel = counter.into();
            let mut new_number = *active_model.number.as_ref() - 1;
            if new_number < 0 {
                new_number = 0;
            }
            active_model.number = Set(new_number);
            match active_model.update(&app_state.db_conn).await {
                Ok(m) => HttpResponse::Ok().json(m.number),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        Ok(None) => {
            HttpResponse::Ok().json(0)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[utoipa::path(
    post,
    tag = "Admin",
    summary = "Set number",
    request_body = SetCounterRequest,
    responses(
        (status=200, description="Ok", body=i32),
        (status = 400, description = "Not logged in"),
        (status=500, description="Internal server error")
    )
)]
#[post("/counter")]
pub async fn set_number(
    app_state: web::Data<AppState>,
    request: web::Json<SetCounterRequest>,
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

    let date = chrono::Utc::now().date_naive();
    let count = db::counter::Entity::find()
        .filter(db::counter::Column::Date.eq(date))
        .one(&app_state.db_conn)
        .await;

    let target = request.number;
    match count {
        Ok(Some(counter)) => {
            let mut active_model: db::counter::ActiveModel = counter.into();
            active_model.number = Set(target);
            match active_model.update(&app_state.db_conn).await {
                Ok(m) => HttpResponse::Ok().json(m.number),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        Ok(None) => {
            let active_model = db::counter::ActiveModel {
                date: Set(date),
                number: Set(target),
                ..Default::default()
            };
            match active_model.insert(&app_state.db_conn).await {
                Ok(m) => HttpResponse::Ok().json(m.number),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
