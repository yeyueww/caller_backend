use sea_orm::ColumnTrait;
use crate::{AppState, db};
use actix_web::{HttpResponse, Responder, get, post, web};
use chrono::NaiveDate;
use migration::Order;
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema)]
pub struct AddCustomerRequest {
    how_much_customer: u64,
    name: Vec<String>,
    date: NaiveDate,
}

#[derive(Deserialize, IntoParams)]
pub struct GetCustomersRequest {
    date: NaiveDate,
}

#[derive(Serialize, ToSchema)]
pub struct PreCustomer {
    id: i32,
    name: String,
}

#[utoipa::path(
    post,
    tag = "Customer",
    summary = "新增顧客到特定日期",
    responses(
        (status = 201, body=[PreCustomer]),
        (status = 500, description = "Internal server error"),
    ),
)]
#[post("/customer")]
pub async fn add_customer(
    app_state: web::Data<AppState>,
    request: web::Json<AddCustomerRequest>,
) -> impl Responder {
    let date = request.date.clone();
    let mut customs = Vec::new();
    let latest_custom = db::customer::Entity::find()
        .column(db::customer::Column::Number)
        .filter(db::customer::Column::Date.eq(date.clone()))
        .order_by(db::customer::Column::Number, Order::Desc)
        .one(&app_state.db_conn)
        .await;
    let mut latest_number = 0;
    match latest_custom {
        Ok(c) => match c {
            Some(c) => {
                latest_number = c.number;
            }
            None => {}
        },
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }
    latest_number = latest_number + 1;
    for i in 0..(request.how_much_customer as usize) {
        if latest_number.to_string().contains('4') || latest_number.to_string().contains('0') {
            latest_number = latest_number + 1
        };
        let custom = db::customer::ActiveModel {
            name: Set(request.name[i].clone()),
            date: Set(date.clone()),
            number: Set(latest_number),
            ..Default::default()
        };
        customs.push(custom);
        latest_number = latest_number + 1;
    }
    let res = db::customer::Entity::insert_many(customs)
        .exec(&app_state.db_conn)
        .await;
    match res {
        Ok(_) => {
            let inserted = db::customer::Entity::find()
                .filter(db::customer::Column::Date.eq(date.clone()))
                .order_by(db::customer::Column::Number, Order::Desc)
                .limit(request.how_much_customer)
                .all(&app_state.db_conn)
                .await;

            match inserted {
                Ok(customers) => {
                    let mut pre_customers = Vec::new();
                    for c in customers {
                        pre_customers.push(PreCustomer {
                            id: c.number,
                            name: c.name,
                        });
                    }
                    HttpResponse::Created().json(pre_customers)
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[utoipa::path(
    get,
    tag = "Customer",
    summary = "取得特定日期的顧客列表",
    responses(
        (status = 200, body=[PreCustomer]),
        (status = 500, description = "Internal server error"),
    ),
)]
#[get("/customer/{date}")]
pub async fn get_customers(
    app_state: web::Data<AppState>,
    date: web::Path<NaiveDate>,
) -> impl Responder {
    let customers = db::customer::Entity::find()
        .filter(db::customer::Column::Date.eq(*date))
        .order_by(db::customer::Column::Number, Order::Asc)
        .all(&app_state.db_conn)
        .await;

    match customers {
        Ok(customers) => {
            let pre_customers: Vec<PreCustomer> = customers.into_iter().map(|c| PreCustomer {
                id: c.number,
                name: c.name,
            }).collect();
            HttpResponse::Ok().json(pre_customers)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[utoipa::path(
    get,
    tag = "Date",
    summary = "取得無法提供服務日期",
    responses(
        (status = 200, body=[String]),
        (status = 500, description = "Internal server error"),
    ),
)]
#[get("/date")]
pub async fn get_no_service_date(
    app_state: web::Data<AppState>,
) -> impl Responder {
    let dates = db::unable_date::Entity::find()
        .filter(db::unable_date::Column::Date.gte(chrono::Utc::now().date_naive()))
        .all(&app_state.db_conn)
        .await;

    match dates {
        Ok(dates) => {
            let res: Vec<chrono::NaiveDate> = dates.into_iter().map(|d| d.date).collect();
            HttpResponse::Ok().json(res)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
