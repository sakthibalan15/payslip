// shared/src/lib.rs
// All DTOs shared between frontend (Leptos 0.8) and backend (Axum)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;

// ── Auth ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub role:  UserRole,
    pub name:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Employee,
}

// ── Payslip ──────────────────────────────────────────────────────────────────

/// One row from the uploaded CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayslipRecord {
    pub employee_email:   String,
    pub employee_name:    String,
    pub employee_id:      i32,
    pub department:       String,
    pub designation:      String,
    pub pay_period_year:  i32,
    pub pay_period_month: i32,
    pub basic:            BigDecimal,
    pub hra:              BigDecimal,
    pub conveyance:       BigDecimal,
    pub other_allowance:  BigDecimal,
    pub pf_deduction:     BigDecimal,
    pub tax_deduction:    BigDecimal,
    pub other_deduction:  BigDecimal,
    pub net_pay:          BigDecimal,
}


// ── CSV preview ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvPreviewResponse {
    pub headers: Vec<String>,
    pub rows:    Vec<Vec<String>>,
    pub total:   usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub uploaded: usize,
    pub message:  String,
}

// ── Generic API error ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

// ── Payslip query params ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayslipQuery {
    pub year:  i32,
    pub month: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPayslipQuery {
    pub year:        i32,
    pub month:       i32,
    pub employee_id: Uuid,
}
