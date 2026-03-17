-- backend/migrations/001_initial.sql

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    role       TEXT NOT NULL CHECK (role IN ('admin','employee')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS otp_tokens (
    user_id    UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    otp        TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used       BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS payslips (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    employee_name    TEXT NOT NULL,
    employee_ext_id  TEXT NOT NULL,
    department       TEXT NOT NULL,
    designation      TEXT NOT NULL,
    year             INTEGER NOT NULL,
    month            INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    basic            NUMERIC(14,2) NOT NULL DEFAULT 0,
    hra              NUMERIC(14,2) NOT NULL DEFAULT 0,
    conveyance       NUMERIC(14,2) NOT NULL DEFAULT 0,
    other_allowance  NUMERIC(14,2) NOT NULL DEFAULT 0,
    pf_deduction     NUMERIC(14,2) NOT NULL DEFAULT 0,
    tax_deduction    NUMERIC(14,2) NOT NULL DEFAULT 0,
    other_deduction  NUMERIC(14,2) NOT NULL DEFAULT 0,
    net_pay          NUMERIC(14,2) NOT NULL DEFAULT 0,
    uploaded_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (employee_id, year, month)
);

CREATE INDEX IF NOT EXISTS idx_payslips_emp ON payslips(employee_id, year, month);

-- Seed admin (update email after deploy)
INSERT INTO users (id, name, email, role)
VALUES (gen_random_uuid(), 'Admin User', 'admin@company.com', 'admin')
ON CONFLICT (email) DO NOTHING;
