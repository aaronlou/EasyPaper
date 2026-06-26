use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, request::Parts},
};

use crate::error::AppError;

const DEVICE_ID_HEADER: &str = "x-easypaper-device-id";
const DEVICE_ID_MAX_LEN: usize = 96;

#[derive(Debug, Clone)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<S> FromRequestParts<S> for DeviceId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        device_id_from_headers(&parts.headers)
    }
}

fn device_id_from_headers(headers: &HeaderMap) -> Result<DeviceId, AppError> {
    let raw = headers
        .get(DEVICE_ID_HEADER)
        .ok_or_else(|| AppError::BadRequest("缺少浏览器设备标识，请刷新页面后重试。".into()))?
        .to_str()
        .map_err(|_| AppError::BadRequest("浏览器设备标识格式无效。".into()))?
        .trim();

    if raw.is_empty() || raw.len() > DEVICE_ID_MAX_LEN {
        return Err(AppError::BadRequest("浏览器设备标识格式无效。".into()));
    }

    if !raw
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Err(AppError::BadRequest("浏览器设备标识格式无效。".into()));
    }

    Ok(DeviceId(raw.to_string()))
}
