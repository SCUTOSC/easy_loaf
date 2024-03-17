use std::time::Duration;

use actix_web::{cookie::time::error, web};
use serde::Serialize;

use crate::model::{mood_response::MoodSubmitTooFrequentErrorResponse, systeminfo::TempSystemInfo};

pub struct AppState {
    /// 数据库连接
    db: sea_orm::DatabaseConnection,
    /// 数据提交的间隔时间
    interval_seconds:u64,
    /// 消息更新间隔
    msg_update_interval_seconds:u64,
    /// 暂存在内存中未更新的系统信息
    temp_system_info:TempSystemInfo,

}

impl AppState {
    ///创建新的状态
    pub async fn new() -> Result<web::Data<Self>,sea_orm::DbErr> {
            let mut dbopts = sea_orm::ConnectOptions::new(dburl);

        dbopts
            .max_connections(100)
            .min_connections(2)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(30))
            .max_lifetime(Duration::from_secs(60))
            .sqlx_logging(false)
            .sqlx_logging_level(log::LevelFilter::Info);

        let db = sea_orm::Database::connect(dbopts)
            .await
            .expect("Failed to connect to database");
        let interval_seconds = 60
            * std::env::var("SUBMIT_INTERVAL")
                .expect("SUBMIT_INTERVAL not set")
                .parse::<u64>()
                .expect("SUBMIT_INTERVAL parse error");
        let msg_update_interval_seconds = std::env::var("MSG_UPDATE_INTERVAL_SECONDS")
            .expect("MSG_UPDATE_INTERVAL_SECONDS not set")
            .parse::<u64>()
            .expect("MSG_UPDATE_INTERVAL_SECONDS parse error");

        let temp_system_info = TempSystemInfo::new();
        let result: web::Data<AppState> = web::Data::new(Self {
            db,
            interval_seconds,
            msg_update_interval_seconds,
            temp_system_info,
        });

        // 从数据库加载数据
        let r = Self::load_from_db(&result).await;
        if let Err(e) = r {
            log::error!("Failed to load data from db: {:?}", e);
            panic!("Failed to load data from db: {:?}", e);
        }

        return Ok(result);
    }
    
    async fn load_from_db(app_data: &web::Data<AppState>) -> Result<(), SystemError> {
        app_data
            .temp_system_info
            .load_visit_count_from_db(app_data)
            .await?;

        return Ok(());
    }

    /// 获取数据库连接
    pub fn db_conn(&self) -> &sea_orm::DatabaseConnection {
        &self.db
    }
    pub fn interval_seconds(&self) -> u64 {
        self.interval_seconds
    }

    pub fn msg_update_interval_seconds(&self) -> u64 {
        self.msg_update_interval_seconds
    }

    pub fn temp_system_info(&self) -> &TempSystemInfo {
        &self.temp_system_info
    }
}

#[derive(Debug)]
pub enum SystemError {
    DbErr(sea_orm::DbErr),
    /// 请在系统内部重试
    InnerRetry,
    Busy(String),
    /// 提交过于频繁
    MoodSubmitTooFrequent(i64),
    /// AI错误
    AIError(String),
}

impl SystemError {
    pub fn to_public_error(&self) -> PublicError {
        log::warn!("SystemError to_public_error: {:?}", self);
        match self {
            SystemError::DbErr(e) => {
                PublicError::new(PublicErrorCode::InternalServerError, format!("数据库错误"))
            }
            SystemError::Busy(s) => {
                PublicError::new(PublicErrorCode::InternalServerError, format!("系统繁忙"))
            }
            SystemError::MoodSubmitTooFrequent(sec) => PublicError::new(
                PublicErrorCode::MoodSubmitTooFrequent,
                MoodSubmitTooFrequentErrorResponse::new(*sec).to_msg(),
            ),
            SystemError::AIError(s) => {
                PublicError::new(PublicErrorCode::InternalServerError, format!("AI错误"))
            }
            SystemError::InnerRetry => PublicError::new(
                PublicErrorCode::InternalServerError,
                format!("系统内部错误"),
            ),
        }
    }
}


impl From<sea_orm::DbErr> for SystemError {
    fn from(e: sea_orm::DbErr) -> Self {
        SystemError::DbErr(e)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PublicErrorCode {
    /// 未找到
    NotFound = 40404,
    /// 未授权
    Unauthorized = 40401,
    /// 无权限
    Forbidden = 40403,
    /// 心情提交过于频繁
    MoodSubmitTooFrequent = 40060,
    /// 参数错误
    InvalidParameter = 40000,
    /// 服务器错误
    InternalServerError = 50000,
}

impl Serialize for PublicErrorCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(*self as u16)
    }
}

/// 暴露給外部的公共错误
#[derive(Debug, Serialize)]
pub struct PublicError {
    status_code: PublicErrorCode,
    message: String,
}

impl PublicError {
    pub fn new(status_code: PublicErrorCode, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }
    pub fn to_json_response(&self) -> actix_web::HttpResponse {
        let code = self.status_code as u16;
        let st = format!("{code}");
        let s = st.as_str();
        if s.bytes().nth(0).unwrap() == b'4' {
            return actix_web::HttpResponse::BadRequest().json(self);
        } else if s.bytes().nth(0).unwrap() == b'5' {
            return actix_web::HttpResponse::InternalServerError().json(self);
        } else {
            return actix_web::HttpResponse::Ok().json(self);
        }
    }
}
