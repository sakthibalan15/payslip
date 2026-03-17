# Payslip App

Full-stack Rust payslip management system.

**Stack:** Leptos 0.8 (CSR/WASM) · Axum 0.8 · PostgreSQL · sqlx 0.8 · printpdf · lettre

---

## Key Leptos 0.8 changes applied

| API | 0.6 (old) | 0.8 (this project) |
|-----|-----------|-------------------|
| Signals | `create_signal(x)` | `signal(x)` or `RwSignal::new(x)` |
| Effects | `create_effect(\|_\| ...)` | `Effect::new(\|_\| ...)` |
| Spawn | `spawn_local(...)` | `leptos::task::spawn_local(...)` |
| Router imports | `leptos_router::*` | `leptos_router::components::*` + `leptos_router::hooks::*` |
| Route macro | `<Route path="/" .../>` | `<Route path=path!("/") .../>` |
| Navigation | `use_navigate()` from prelude | `leptos_router::hooks::use_navigate()` |
| Mount | `mount_to_body(App)` | same |

---

## Project layout

```
psapp/
├── Cargo.toml                   ← workspace
├── .env.example
├── sample_data.csv
│
├── shared/src/lib.rs            ← shared DTOs (UserRole, PayslipRecord, etc.)
│
├── backend/
│   ├── Cargo.toml
│   ├── migrations/001_initial.sql
│   └── src/
│       ├── main.rs              ← Axum router, CORS
│       ├── config.rs            ← env config
│       ├── state.rs             ← AppState (PgPool + Config)
│       ├── db.rs                ← PgPoolOptions
│       ├── auth.rs              ← JWT create/verify + OTP gen
│       ├── error.rs             ← AppError → HTTP response
│       ├── pdf.rs               ← printpdf payslip generator
│       ├── middleware/auth.rs   ← JWT guard (axum middleware)
│       └── handlers/
│           ├── auth.rs          ← POST send-otp / verify-otp
│           ├── admin.rs         ← CSV preview/upload, payslip PDF
│           └── employee.rs      ← own payslip preview/download
│
└── frontend/
    ├── Cargo.toml               ← leptos 0.8, leptos_router 0.8
    ├── Trunk.toml               ← WASM bundler + dev proxy
    ├── index.html               ← HTML shell + all CSS
    └── src/
        ├── lib.rs               ← App root, Router, Protected guard
        ├── store.rs             ← AuthCtx (signal-based, localStorage)
        ├── api.rs               ← gloo-net HTTP wrappers
        ├── components/
        │   └── header.rs        ← AppHeader with logout
        └── pages/
            ├── auth.rs          ← LoginPage, OtpPage
            ├── admin.rs         ← AdminDashboard (upload + payslip panels)
            ├── employee.rs      ← EmployeeDashboard
            └── not_found.rs     ← 404 fallback
```

---

## Database schema

```sql
users       — id, name, email, role (admin|employee)
otp_tokens  — user_id (PK/FK), otp, expires_at, used
payslips    — employee_id+year+month (UNIQUE), all salary columns
```

---

## Setup

### Prerequisites

```bash
# Rust + WASM target
rustup target add wasm32-unknown-unknown

# Trunk (WASM bundler for Leptos CSR)
cargo install trunk

# sqlx CLI
cargo install sqlx-cli --no-default-features --features postgres
```

### 1. Database

```bash
createdb payslip_app
cp .env.example .env       # fill in DATABASE_URL, JWT_SECRET, SMTP_*
cd backend
sqlx migrate run
```

### 2. Backend

```bash
cd backend
cargo run
# → http://localhost:3001
```

### 3. Frontend

```bash
cd frontend
trunk serve
# → http://localhost:8080
# Trunk proxies /api/* → localhost:3001 (see Trunk.toml)
```

---

## API reference

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/auth/send-otp` | — | Send 6-digit OTP to email |
| POST | `/api/auth/verify-otp` | — | Verify OTP → JWT |
| POST | `/api/admin/csv/preview` | Admin JWT | Preview first 5 rows |
| POST | `/api/admin/csv/upload` | Admin JWT | Persist all rows |
| GET | `/api/admin/payslip/preview` | Admin JWT | PDF stream (any employee) |
| GET | `/api/admin/payslip/download` | Admin JWT | PDF attachment |
| GET | `/api/employee/payslip/preview` | Employee JWT | Own PDF stream |
| GET | `/api/employee/payslip/download` | Employee JWT | Own PDF attachment |

### Query params for payslip endpoints
- `year=2024&month=3`
- Admin only: `employee_id=<uuid>`
- Frontend appends `&token=<jwt>` so `<iframe src=...>` works without custom headers

---

## CSV format

See `sample_data.csv`. Columns must be in this exact order:

```
employee_email, employee_name, employee_id, department, designation,
pay_period_year, pay_period_month,
basic, hra, conveyance, other_allowance,
pf_deduction, tax_deduction, other_deduction, net_pay
```

New employees are **auto-created** in `users` on first upload.

---

## Security notes

- OTPs expire after `OTP_TTL_SECS` (default 5 min) and are single-use
- JWTs are HS256 / 24 h — switch to RS256 for production
- Employees can only access their own payslips (enforced via `claims.sub`)
- Use HTTPS + secure cookies in production
# payslip
