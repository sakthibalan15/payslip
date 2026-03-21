// backend/src/handlers/admin.rs
use axum::{
    body::Body,
    extract::{Extension, Multipart, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bigdecimal::ToPrimitive;
use shared::{AdminPayslipQuery, CsvPreviewResponse, PayslipRecord, UploadResponse, UserRole};
use uuid::Uuid;
use crate::{auth::Claims, error::{AppError, Result}, pdf, state::AppState};

/// POST /api/admin/csv/preview
pub async fn preview_csv(
    State(_s): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut mp: Multipart,
) -> Result<Json<CsvPreviewResponse>> {
    require_admin(&claims)?;
    let bytes = read_file_bytes(&mut mp).await?;
    Ok(Json(csv_preview(&bytes)?))
}

/// POST /api/admin/csv/upload
pub async fn upload_csv(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut mp: Multipart,
) -> Result<Json<UploadResponse>> {
    require_admin(&claims)?;
    let bytes = read_file_bytes(&mut mp).await?;
    let records = csv_records(&bytes)?;
    let count = records.len();

    for rec in records {
        let user = sqlx::query!(
            "SELECT id FROM users WHERE email = $1", rec.employee_email
        ).fetch_optional(&state.pool).await?;

        let emp_id = match user {
            Some(u) => u.id,
            None => {
                let id = Uuid::new_v4();
                sqlx::query!(
                    "INSERT INTO users (id, name, email, role) VALUES ($1,$2,$3,'employee')",
                    id, rec.employee_name, rec.employee_email
                ).execute(&state.pool).await?;
                id
            }
        };

        sqlx::query!(
            r#"INSERT INTO payslips
               (id, employee_id, employee_name, employee_ext_id, department, designation,
                year, month, basic, hra, conveyance, other_allowance,
                pf_deduction, tax_deduction, other_deduction, net_pay)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,
                       $9::float8::numeric,$10::float8::numeric,$11::float8::numeric,$12::float8::numeric,
                       $13::float8::numeric,$14::float8::numeric,$15::float8::numeric,$16::float8::numeric)
               ON CONFLICT (employee_id, year, month) DO UPDATE SET
                 basic=EXCLUDED.basic, hra=EXCLUDED.hra, conveyance=EXCLUDED.conveyance,
                 other_allowance=EXCLUDED.other_allowance, pf_deduction=EXCLUDED.pf_deduction,
                 tax_deduction=EXCLUDED.tax_deduction, other_deduction=EXCLUDED.other_deduction,
                 net_pay=EXCLUDED.net_pay"#,
            Uuid::new_v4(),               // $1  id
            emp_id,                        // $2  employee_id (Uuid)
            rec.employee_name,             // $3  employee_name
            rec.employee_id.to_string(),   // $4  employee_ext_id — i32 converted to String
            rec.department,                // $5  department
            rec.designation,               // $6  designation
            rec.pay_period_year,           // $7  year
            rec.pay_period_month,          // $8  month
            rec.basic
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: basic must be numeric".into()))?,
            rec.hra
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: hra must be numeric".into()))?,
            rec.conveyance
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: conveyance must be numeric".into()))?,
            rec.other_allowance
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: other_allowance must be numeric".into()))?,
            rec.pf_deduction
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: pf_deduction must be numeric".into()))?,
            rec.tax_deduction
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: tax_deduction must be numeric".into()))?,
            rec.other_deduction
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: other_deduction must be numeric".into()))?,
            rec.net_pay
                .to_f64()
                .ok_or_else(|| AppError::BadRequest("CSV: net_pay must be numeric".into()))?,
        ).execute(&state.pool).await?;
    }

    Ok(Json(UploadResponse {
        uploaded: count,
        message:  format!("Successfully uploaded {count} records"),
    }))
}

/// GET /api/admin/payslip/preview?year=&month=&employee_id=
pub async fn preview_payslip(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AdminPayslipQuery>,
) -> Result<Response> {
    require_admin(&claims)?;
    let data = fetch_payslip(&state, q.employee_id, q.year, q.month).await?;
    Ok(inline_pdf(pdf::generate(&data)))
}

/// GET /api/admin/payslip/download?year=&month=&employee_id=
pub async fn download_payslip(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<AdminPayslipQuery>,
) -> Result<Response> {
    require_admin(&claims)?;
    let data = fetch_payslip(&state, q.employee_id, q.year, q.month).await?;
    Ok(attachment_pdf(pdf::generate(&data), q.year, q.month))
}

// ── shared helpers ─────────────────────────────────────────────────────────────

pub async fn fetch_payslip(
    state: &AppState,
    employee_id: Uuid,
    year: i32,
    month: i32,
) -> Result<pdf::PayslipData> {
    struct Row {
        employee_name:   String,
        employee_ext_id: String,
        employee_email:  String,
        department:      String,
        designation:     String,
        year:            i32,
        month:           i32,
        basic:           f64,
        hra:             f64,
        conveyance:      f64,
        other_allowance: f64,
        pf_deduction:    f64,
        tax_deduction:   f64,
        other_deduction: f64,
        net_pay:         f64,
    }

    let r = sqlx::query_as!(
        Row,
        r#"SELECT p.employee_name, p.employee_ext_id, u.email as employee_email,
                  p.department, p.designation, p.year, p.month,
                  p.basic::float8 as "basic!",
                  p.hra::float8 as "hra!",
                  p.conveyance::float8 as "conveyance!",
                  p.other_allowance::float8 as "other_allowance!",
                  p.pf_deduction::float8 as "pf_deduction!",
                  p.tax_deduction::float8 as "tax_deduction!",
                  p.other_deduction::float8 as "other_deduction!",
                  p.net_pay::float8 as "net_pay!"
           FROM payslips p JOIN users u ON u.id = p.employee_id
           WHERE p.employee_id = $1 AND p.year = $2 AND p.month = $3"#,
        employee_id, year, month
    ).fetch_optional(&state.pool).await?.ok_or(AppError::NotFound)?;

    Ok(pdf::PayslipData {
        employee_name:   r.employee_name,
        employee_id:     r.employee_ext_id,
        employee_email:  r.employee_email,
        department:      r.department,
        designation:     r.designation,
        year:            r.year,
        month:           r.month,
        basic:           r.basic,
        hra:             r.hra,
        conveyance:      r.conveyance,
        other_allowance: r.other_allowance,
        pf_deduction:    r.pf_deduction,
        tax_deduction:   r.tax_deduction,
        other_deduction: r.other_deduction,
        net_pay:         r.net_pay,
    })
}

pub fn inline_pdf(bytes: Vec<u8>) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, "application/pdf")], Body::from(bytes)).into_response()
}

pub fn attachment_pdf(bytes: Vec<u8>, year: i32, month: i32) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"payslip_{year}_{month:02}.pdf\"")),
        ],
        Body::from(bytes),
    ).into_response()
}

fn require_admin(c: &Claims) -> Result<()> {
    if c.role != UserRole::Admin { Err(AppError::Forbidden) } else { Ok(()) }
}

async fn read_file_bytes(mp: &mut Multipart) -> Result<Vec<u8>> {
    while let Some(f) = mp.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if f.name() == Some("file") {
            return Ok(f.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?.to_vec());
        }
    }
    Err(AppError::BadRequest("No 'file' field".into()))
}

fn csv_preview(bytes: &[u8]) -> Result<CsvPreviewResponse> {
    let mut rdr = csv::Reader::from_reader(bytes);
    let headers: Vec<String> = rdr.headers()
        .map_err(|_| AppError::BadRequest("Bad CSV headers".into()))?
        .iter().map(String::from).collect();
    let mut rows = Vec::new();
    let mut total = 0usize;
    for r in rdr.records() {
        let rec = r.map_err(|_| AppError::BadRequest("CSV parse error".into()))?;
        total += 1;
        if total <= 5 { rows.push(rec.iter().map(String::from).collect()); }
    }
    Ok(CsvPreviewResponse { headers, rows, total })
}

fn csv_records(bytes: &[u8]) -> Result<Vec<PayslipRecord>> {
    let mut rdr = csv::Reader::from_reader(bytes);
    rdr.deserialize()
        .map(|r| r.map_err(|e| AppError::BadRequest(format!("CSV row: {e}"))))
        .collect()
}