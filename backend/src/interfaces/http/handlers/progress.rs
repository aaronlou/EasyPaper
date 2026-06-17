use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::interfaces::http::AppState;
use crate::models::api::ProgressInfo;

/// GET /api/papers/:id/progress  —— 获取论文解读进度
///
/// 如果内存中没有该论文的进度记录，返回一个默认的“处理中”状态，
/// 这样前端在上传后能立即看到步骤条，而不必等待第一次 emit。
pub async fn get_progress(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ProgressInfo>> {
    let info = state.get_progress(id).await.unwrap_or_else(|| {
        ProgressInfo::new("interpreting", "准备中", "正在排队，即将开始 AI 解读...", 5)
    });

    Ok(Json(info))
}
