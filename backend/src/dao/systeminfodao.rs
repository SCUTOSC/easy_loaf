use actix_web::web;
use chrono::{DateTime, Local};
use log::{info, warn};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Statement,
};

use crate::{
    controller::{AppState, SystemError},
    entity::systeminfo,
    model::systeminfo::SystemInfoType,
};

#[derive(Debug)]
pub struct SystemInfoDao;

impl SystemInfoDao {
    pub async fn get_system_info (
        
    )
}