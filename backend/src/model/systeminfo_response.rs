use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BaseSystemInfoResponse {
    /// 总点击次数
    pub visitor_count: i64,
}
