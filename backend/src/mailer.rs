// backend/src/mailer.rs
use anyhow::Context;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use crate::config::Config;

/// Build a reusable async SMTP transport from config.
/// Call once at startup and store in AppState.
pub fn build_transport(cfg: &Config) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(cfg.smtp_user.clone(), cfg.smtp_password.clone());

    let transport = if cfg.smtp_use_ssl {
        // Port 465 — implicit TLS
        AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)
            .context("Failed to build SMTP relay")?
            .credentials(creds)
            .port(cfg.smtp_port)
            .build()
    } else {
        // Port 587 — STARTTLS
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)
            .context("Failed to build STARTTLS relay")?
            .credentials(creds)
            .port(cfg.smtp_port)
            .build()
    };

    Ok(transport)
}

/// Send a plain-text email.
pub async fn send_email(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    to: &str,
    subject: &str,
    body: String,
) -> anyhow::Result<()> {
    let from_addr: Mailbox = from.parse().context("Invalid from address")?;
    let to_addr: Mailbox   = to.parse().context("Invalid to address")?;

    let email = Message::builder()
        .from(from_addr)
        .to(to_addr)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .context("Failed to build email message")?;

    transport.send(email).await.context("Failed to send email")?;
    Ok(())
}

/// Send an HTML email (e.g. for payslip notifications).
pub async fn send_html_email(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    to: &str,
    subject: &str,
    html_body: String,
) -> anyhow::Result<()> {
    let from_addr: Mailbox = from.parse().context("Invalid from address")?;
    let to_addr: Mailbox   = to.parse().context("Invalid to address")?;

    let email = Message::builder()
        .from(from_addr)
        .to(to_addr)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .context("Failed to build HTML email message")?;

    transport.send(email).await.context("Failed to send email")?;
    Ok(())
}

/// Send an email with a PDF attachment (for payslip download).
pub async fn send_pdf_attachment(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    to: &str,
    subject: &str,
    text_body: &str,
    pdf_bytes: Vec<u8>,
    filename: &str,
) -> anyhow::Result<()> {
    use lettre::message::{MultiPart, SinglePart, Attachment};

    let from_addr: Mailbox = from.parse().context("Invalid from address")?;
    let to_addr: Mailbox   = to.parse().context("Invalid to address")?;

    let pdf_content_type = "application/pdf".parse().unwrap();
    let attachment = Attachment::new(filename.to_string())
        .body(pdf_bytes, pdf_content_type);

    let email = Message::builder()
        .from(from_addr)
        .to(to_addr)
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text_body.to_string())
                )
                .singlepart(attachment),
        )
        .context("Failed to build email with attachment")?;

    transport.send(email).await.context("Failed to send email with attachment")?;
    Ok(())
}