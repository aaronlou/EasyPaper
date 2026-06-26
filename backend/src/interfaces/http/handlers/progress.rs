use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::interfaces::http::AppState;
use crate::interfaces::http::device::DeviceId;
use crate::models::api::ProgressInfo;

/// GET /api/papers/:id/progress  —— 获取论文解读进度
///
/// 如果内存中没有该论文的进度记录，会从数据库状态推导一个兜底进度，
/// 避免服务重启后遗留的 processing 论文在前端永久转圈。
pub async fn get_progress(
    State(state): State<AppState>,
    device_id: DeviceId,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ProgressInfo>> {
    let info = state
        .get_progress(device_id.as_str(), id)
        .await
        .unwrap_or_else(|| {
            ProgressInfo::new("interpreting", "准备中", "正在排队，即将开始 AI 解读...", 5)
        });

    Ok(Json(info))
}
