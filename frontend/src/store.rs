// frontend/src/store.rs
// Leptos 0.8: signal() instead of create_signal(), RwSignal::new() etc.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use shared::UserRole;
use gloo_storage::Storage;

const STORAGE_KEY: &str = "psapp_session";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub token: String,
    pub role:  UserRole,
    pub name:  String,
    pub email: String,
}

/// Everything the app needs from auth
#[derive(Clone, Copy)]
pub struct AuthCtx {
    pub session:           ReadSignal<Option<Session>>,
    pub set_session:       WriteSignal<Option<Session>>,
    pub pending_email:     ReadSignal<Option<String>>,
    pub set_pending_email: WriteSignal<Option<String>>,
    /// Shown on the OTP page after Send OTP succeeds (e.g. "OTP sent to your email").
    pub otp_send_notice:     ReadSignal<Option<String>>,
    pub set_otp_send_notice: WriteSignal<Option<String>>,
}

pub fn provide_auth() {
    // Try to restore previous session from localStorage
    let initial: Option<Session> = gloo_storage::LocalStorage::get(STORAGE_KEY).ok();

    // Leptos 0.8: signal() returns (ReadSignal, WriteSignal)
    let (session, set_session)               = signal(initial);
    let (pending_email, set_pending_email)   = signal::<Option<String>>(None);
    let (otp_send_notice, set_otp_send_notice) = signal::<Option<String>>(None);

    // Persist changes to localStorage
    Effect::new(move |_| {
        match session.get() {
            Some(ref s) => { let _ = gloo_storage::LocalStorage::set(STORAGE_KEY, s); }
            None        => { gloo_storage::LocalStorage::delete(STORAGE_KEY); }
        }
    });

    provide_context(AuthCtx {
        session,
        set_session,
        pending_email,
        set_pending_email,
        otp_send_notice,
        set_otp_send_notice,
    });
}

pub fn use_auth() -> AuthCtx {
    use_context::<AuthCtx>().expect("AuthCtx missing — did you call provide_auth()?")
}
