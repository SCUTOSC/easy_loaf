use std::sync::atomic::AtomicI64;

use actix_web::web;
use serde::{Deserialize, Serialize};

use crate::{
    controller::{AppState, SystemError},
    dao::systeminfo_dao::SystemInfoDao,
};

#[derive(Debug)]
pub struct TempSystemInfo {
    /// 今日访问次数
    base_visit_count: AtomicI64,
    /// 还没有保存到数据库的访问次数
    visit_count_increase: AtomicI64,
}

impl TempSystemInfo {
    pub fn new() -> Self {
        Self {
            base_visit_count: AtomicI64::new(0),
            visit_count_increase: AtomicI64::new(0),
        }
    }

    pub fn base_visit_count(&self) -> i64 {
        self.base_visit_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn base_visit_count_increase(&self) -> i64 {
        self.visit_count_increase
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn take_visit_count_increase(&self) -> i64 {
        self.visit_count_increase
            .swap(0, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn increase_base_visit_count(&self) {
        self.visit_count_increase
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn increase_base_visit_count_by(&self, count: i64) {
        self.visit_count_increase
            .fetch_add(count, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_base_visit_count(&self, count: i64) {
        self.base_visit_count
            .store(count, std::sync::atomic::Ordering::SeqCst);
    }

    /// 从数据库加载数据
    pub async fn load_visit_count_from_db(
        &self,
        app_data: &web::Data<AppState>,
    ) -> Result<(), SystemError> {
        let mut retrying = false;
        loop {
            let r = SystemInfoDao::get_system_info(app_data, SystemInfoType::VisitCount).await;
            match r {
                Ok(system_info) => {
                    self.set_base_visit_count(system_info.bigint1.unwrap_or(0));
                    return Ok(());
                }
                Err(e) => {
                    if retrying {
                        log::error!("Failed to load system info from db: {:?}", e);
                        return Err(e);
                    } else {
                        log::warn!("Failed to load system info from db: {:?}, retrying...", e);
                        retrying = true;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemInfoType {
    /// 总访问次数
    VisitCount,
}

impl ToString for SystemInfoType {
    fn to_string(&self) -> String {
        match self {
            SystemInfoType::VisitCount => "VisitCount".to_string(),
        }
    }
}
