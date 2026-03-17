// frontend/src/pages/auth.rs
// Leptos 0.8:
//   - signal()          instead of create_signal()
//   - Effect::new()     instead of create_effect()
//   - task::spawn_local instead of spawn_local (top-level)
//   - event_target_value(&e) unchanged

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::{api, components::header::AppHeader, store::use_auth};

// ── Login Page ────────────────────────────────────────────────────────────────

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    {
        let navigate = navigate.clone();
        let auth = auth.clone();

        Effect::new(move |_| {
            if let Some(s) = auth.session.get() {
                let dest = match s.role {
                    shared::UserRole::Admin => "/admin",
                    shared::UserRole::Employee => "/employee",
                };

                navigate(dest, Default::default());
            }
        });
    }

    let email = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let on_send = {
        let auth = auth.clone();
        let navigate = navigate.clone();

        move |_| {
            let v = email.get();

            if v.trim().is_empty() {
                error.set(Some("Please enter your company email".into()));
                return;
            }

            loading.set(true);
            error.set(None);

            let auth = auth.clone();
            let navigate = navigate.clone();
            let error = error;
            let loading = loading;

            spawn_local(async move {
                match api::send_otp(&v).await {
                    Ok(_) => {
                        auth.set_pending_email.set(Some(v));
                        navigate("/otp", Default::default());
                    }

                    Err(e) => {
                        error.set(Some(e));
                    }
                }

                loading.set(false);
            });
        }
    };

    view! {
        <AppHeader />

        <div class="card">
            <div class="field">
                <label>"Email id"</label>

                <input
                    type="email"
                    placeholder="you@company.com"
                    prop:value=move || email.get()
                    on:input=move |e| email.set(event_target_value(&e))
                />
            </div>

            {
                move || error.get().map(|msg| {
                    view! {
                        <div class="msg-err">{msg}</div>
                    }
                })
            }

            <div class="btn-center">
                <button
                    class="btn btn-primary"
                    on:click=on_send
                    prop:disabled=move || loading.get()
                >
                    {
                        move || {
                            if loading.get() {
                                view! {
                                    <>
                                        <span class="spinner"></span>
                                        " Sending..."
                                    </>
                                }.into_any()
                            } else {
                                view! {
                                    "Send OTP"
                                }.into_any()
                            }
                        }
                    }
                </button>
            </div>
        </div>
    }
}

// ── OTP Page ──────────────────────────────────────────────────────────────────

#[component]
pub fn OtpPage() -> impl IntoView {
    let auth     = use_auth();
    let navigate = use_navigate();
    let nav_for_effect = navigate.clone();

    // Guard: if no pending email, go back to login
    // Effect::new(move |_| {
    //     if auth.pending_email.get().is_none() {
    //         navigate("/login", Default::default());
    //     }
    // });

    Effect::new(move |_| {
        if auth.pending_email.get().is_none() {
            nav_for_effect("/login", Default::default());
        }
    });

    let email_val = move || auth.pending_email.get().unwrap_or_default();

    // Six individual digit signals — Leptos 0.8 RwSignal::new()
    let digits: Vec<RwSignal<String>> = (0..6).map(|_| RwSignal::new(String::new())).collect();

    let loading = RwSignal::new(false);
    let error   = RwSignal::new(Option::<String>::None);

    let digits_c = digits.clone();
    // let on_verify = {
    //     let value = navigate.clone();
    //     move |_| {
    //         let otp: String = digits_c.iter().map(|d| d.get()).collect();
    //         if otp.chars().count() < 6 {
    //             error.set(Some("Please enter all 6 digits".into()));
    //             return;
    //         }
    //         let em = email_val();
    //         loading.set(true);
    //         error.set(None);

    //         spawn_local(async move {
    //             match api::verify_otp(&em, &otp).await {
    //                 Ok(resp) => {
    //                     auth.set_session.set(Some(crate::store::Session {
    //                         token: resp.token,
    //                         role:  resp.role.clone(),
    //                         name:  resp.name,
    //                         email: em,
    //                     }));
    //                     auth.set_pending_email.set(None);
    //                     navigate(
    //                         match resp.role {
    //                             shared::UserRole::Admin    => "/admin",
    //                             shared::UserRole::Employee => "/employee",
    //                         },
    //                         Default::default(),
    //                     );
    //                 }
    //                 Err(e) => error.set(Some(e)),
    //             }
    //             loading.set(false);
    //         });
    //     }
    // };

    let on_verify = {
        let auth = auth.clone();
        let value = navigate.clone();

        move |_| {
            let otp: String = digits_c.iter().map(|d| d.get()).collect();

            if otp.chars().count() < 6 {
                error.set(Some("Please enter all 6 digits".into()));
                return;
            }

            let em = email_val();

            loading.set(true);
            error.set(None);

            let auth = auth.clone();
            let nav = value.clone();
            let error = error;
            let loading = loading;

            spawn_local(async move {
                match api::verify_otp(&em, &otp).await {
                    Ok(resp) => {
                        auth.set_session.set(Some(crate::store::Session {
                            token: resp.token,
                            role: resp.role.clone(),
                            name: resp.name,
                            email: em,
                        }));

                        auth.set_pending_email.set(None);

                        let path = match resp.role {
                            shared::UserRole::Admin => "/admin",
                            shared::UserRole::Employee => "/employee",
                        };

                        nav(path, Default::default());
                    }

                    Err(e) => {
                        error.set(Some(e));
                    }
                }

                loading.set(false);
            });
        }
    };


    view! {
        <AppHeader />
        <div class="card">
            <div class="field">
                <label>"Email id"</label>
                <input type="email" prop:value=email_val disabled=true />
            </div>

            <div class="field">
                <label>"Enter OTP"</label>
                <div class="otp-row">
                    {digits.iter().enumerate().map(|(i, sig)| {
                        let sig = *sig;
                        view! {
                            <input
                                id=format!("otp-{i}")
                                class="otp-box"
                                type="text"
                                inputmode="numeric"
                                maxlength="1"
                                prop:value=move || sig.get()
                                on:input=move |e: web_sys::Event| {
                                    let ch = event_target_value(&e)
                                        .chars().last()
                                        .map(|c| c.to_string())
                                        .unwrap_or_default();
                                    sig.set(ch);
                                    // auto-advance to next box
                                    if let Some(win) = web_sys::window() {
                                        if let Some(doc) = win.document() {
                                            let next = format!("otp-{}", i + 1);
                                            if let Some(el) = doc.get_element_by_id(&next) {
                                                let _ = wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(el)
                                                    .map(|inp| inp.focus());
                                            }
                                        }
                                    }
                                }
                                on:keydown=move |e: web_sys::KeyboardEvent| {
                                    // backspace → move to previous box
                                    if e.key() == "Backspace" && sig.get().is_empty() {
                                        if let Some(win) = web_sys::window() {
                                            if let Some(doc) = win.document() {
                                                let prev = format!("otp-{}", i.saturating_sub(1));
                                                if let Some(el) = doc.get_element_by_id(&prev) {
                                                    let _ = wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(el)
                                                        .map(|inp| inp.focus());
                                                }
                                            }
                                        }
                                    }
                                }
                            />
                        }
                    }).collect_view()}
                </div>
            </div>

            {move || error.get().map(|msg| view! { <div class="msg-err">{msg}</div> })}

            <div class="btn-center">
                <button class="btn btn-primary" on:click=on_verify prop:disabled=move || loading.get()>
                    {move || if loading.get() {
                        view! { <><span class="spinner"></span>" Verifying..."</> }.into_any()
                    } else {
                        view! { "Verify OTP" }.into_any()
                    }}
                </button>
            </div>
        </div>
    }
}
