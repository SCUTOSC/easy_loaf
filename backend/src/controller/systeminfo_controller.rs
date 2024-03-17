use actix_web::{get, web, HttpResponse, Scope};

use crate::model::systeminfo_response::BaseSystemInfoResponse;

use super::AppState;

#[derive(Debug)]
pub struct SystemInfoController;

impl SystemInfoController {
    pub fn new() -> Scope {
        let x: Scope = web::scope("/systeminfo").service(base);
        x
    }
}

#[get("/base")]
async fn base(app_data: web::Data<AppState>) -> HttpResponse {
    let tmpsi = app_data.temp_system_info();
    let mut res = BaseSystemInfoResponse::default();
    res.visitor_count = tmpsi.base_visit_count() + tmpsi.base_visit_count_increase();

    HttpResponse::Ok().json(res)
}
