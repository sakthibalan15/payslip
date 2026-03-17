// frontend/src/api.rs
// All API calls to the Axum backend.
// gloo-net 0.6 uses RequestBuilder — no breaking changes from 0.5 for basic usage.

use gloo_net::http::Request;
use shared::*;

const BASE: &str = "http://localhost:3001";

// ── Auth ──────────────────────────────────────────────────────────────────────

pub async fn send_otp(email: &str) -> Result<String, String> {
    let res = Request::post(&format!("{BASE}/api/auth/send-otp"))
        .json(&SendOtpRequest { email: email.to_string() })
        .map_err(|e| e.to_string())?
        .send().await
        .map_err(|e| e.to_string())?;

    if res.ok() {
        let v: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        Ok(v["message"].as_str().unwrap_or("OTP sent").to_string())
    } else {
        Err(res.json::<ApiError>().await.map(|e| e.message).unwrap_or_else(|_| "Request failed".into()))
    }
}

pub async fn verify_otp(email: &str, otp: &str) -> Result<AuthResponse, String> {
    let res = Request::post(&format!("{BASE}/api/auth/verify-otp"))
        .json(&VerifyOtpRequest { email: email.to_string(), otp: otp.to_string() })
        .map_err(|e| e.to_string())?
        .send().await
        .map_err(|e| e.to_string())?;

    if res.ok() {
        res.json::<AuthResponse>().await.map_err(|e| e.to_string())
    } else {
        Err(res.json::<ApiError>().await.map(|e| e.message).unwrap_or_else(|_| "Request failed".into()))
    }
}

// ── Admin CSV ─────────────────────────────────────────────────────────────────

pub async fn csv_preview(token: &str, file: web_sys::File) -> Result<CsvPreviewResponse, String> {
    let form = web_sys::FormData::new().unwrap();
    form.append_with_blob("file", &file).unwrap();

    let res = Request::post(&format!("{BASE}/api/admin/csv/preview"))
        .header("Authorization", &format!("Bearer {token}"))
        .body(form).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if res.ok() {
        res.json::<CsvPreviewResponse>().await.map_err(|e| e.to_string())
    } else {
        Err(res.json::<ApiError>().await.map(|e| e.message).unwrap_or_else(|_| "Preview failed".into()))
    }
}

pub async fn csv_upload(token: &str, file: web_sys::File) -> Result<UploadResponse, String> {
    let form = web_sys::FormData::new().unwrap();
    form.append_with_blob("file", &file).unwrap();

    let res = Request::post(&format!("{BASE}/api/admin/csv/upload"))
        .header("Authorization", &format!("Bearer {token}"))
        .body(form).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if res.ok() {
        res.json::<UploadResponse>().await.map_err(|e| e.to_string())
    } else {
        Err(res.json::<ApiError>().await.map(|e| e.message).unwrap_or_else(|_| "Upload failed".into()))
    }
}

// ── PDF URL builders ──────────────────────────────────────────────────────────
// Token is passed as a query param so an <iframe src=url> works without custom headers.
// The backend reads it from query: ?token=...
// (You can also add a dedicated /pdf/token endpoint if you prefer.)

pub fn admin_preview_url(token: &str, year: i32, month: i32, emp_id: &str) -> String {
    format!("{BASE}/api/admin/payslip/preview?year={year}&month={month}&employee_id={emp_id}&token={token}")
}
pub fn admin_download_url(token: &str, year: i32, month: i32, emp_id: &str) -> String {
    format!("{BASE}/api/admin/payslip/download?year={year}&month={month}&employee_id={emp_id}&token={token}")
}
pub fn emp_preview_url(token: &str, year: i32, month: i32) -> String {
    format!("{BASE}/api/employee/payslip/preview?year={year}&month={month}&token={token}")
}
pub fn emp_download_url(token: &str, year: i32, month: i32) -> String {
    format!("{BASE}/api/employee/payslip/download?year={year}&month={month}&token={token}")
}
