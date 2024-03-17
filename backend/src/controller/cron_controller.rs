use actix_web::{get, web, HttpResponse, Scope};
use serde::Deserialize;

use crate::service::cron_service::CronService;

use super::{AppState, PublicError, PublicErrorCode};

#[derive(Debug)]
pub struct CronController;

impl CronController {
    pub fn new() -> Scope {
        let x: Scope = web::scope("/cron").service(cron_job_trigger);
        x
    }
}

#[derive(Deserialize)]
struct TriggerQuery {
    key: String,
}
#[get("/trigger")]
async fn cron_job_trigger(
    app_data: web::Data<AppState>,
    query: web::Query<TriggerQuery>,
) -> HttpResponse {
    if query.key.as_str() != std::env::var("CRON_JOB_KEY").unwrap() {
        return PublicError::new(PublicErrorCode::Forbidden, format!("无效的key！"))
            .to_json_response();
    }

    let res = CronService::cron_job_trigger(&app_data).await;
    if let Err(e) = res {
        return e.to_public_error().to_json_response();
    }

    return HttpResponse::Ok().json(res.unwrap());
}
