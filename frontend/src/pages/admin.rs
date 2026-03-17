// frontend/src/pages/admin.rs
// Leptos 0.8: RwSignal::new(), signal(), Effect::new(), task::spawn_local

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::CsvPreviewResponse;
use wasm_bindgen::JsCast;

use crate::{api, components::header::AppHeader, store::use_auth};

// ── Admin Dashboard ───────────────────────────────────────────────────────────

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let auth     = use_auth();
    let name     = move || auth.session.get().map(|s| s.name).unwrap_or_else(|| "Admin".into());
    let active   = RwSignal::new("upload"); // "upload" | "payslip"

    view! {
        <AppHeader show_logout=true />
        <div class="dashboard">
            <div class="welcome">"Hello " {name} "!!"</div>

            <div class="tabs">
                <button
                    class=move || if active.get() == "upload" { "tab active" } else { "tab inactive" }
                    on:click=move |_| active.set("upload")
                >"Upload Data"</button>
                <button
                    class=move || if active.get() == "payslip" { "tab active" } else { "tab inactive" }
                    on:click=move |_| active.set("payslip")
                >"Download Payslip"</button>
            </div>

            {move || match active.get() {
                "upload"  => view! { <UploadPanel /> }.into_any(),
                "payslip" => view! { <AdminPayslipPanel /> }.into_any(),
                _         => view! { <></> }.into_any(),
            }}
        </div>
    }
}

// ── Upload Panel ──────────────────────────────────────────────────────────────

#[component]
fn UploadPanel() -> impl IntoView {
    let auth    = use_auth();
    let file    = RwSignal::new(Option::<web_sys::File>::None);
    let preview = RwSignal::new(Option::<CsvPreviewResponse>::None);
    let loading = RwSignal::new(false);
    let message = RwSignal::new(Option::<(bool, String)>::None);

    let on_file = move |e: web_sys::Event| {
        let input = e.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        if let Some(files) = input.files() {
            if let Some(f) = files.get(0) {
                file.set(Some(f));
                preview.set(None);
                message.set(None);
            }
        }
    };

    let on_preview = move |_| {
        let Some(f) = file.get() else { return; };
        let tok = auth.session.get().map(|s| s.token).unwrap_or_default();
        loading.set(true);
        spawn_local(async move {
            match api::csv_preview(&tok, f).await {
                Ok(p)  => { preview.set(Some(p)); message.set(None); }
                Err(e) => message.set(Some((false, e))),
            }
            loading.set(false);
        });
    };

    let on_upload = move |_| {
        let Some(f) = file.get() else { return; };
        let tok = auth.session.get().map(|s| s.token).unwrap_or_default();
        loading.set(true);
        spawn_local(async move {
            match api::csv_upload(&tok, f).await {
                Ok(r)  => { message.set(Some((true, r.message))); preview.set(None); }
                Err(e) => message.set(Some((false, e))),
            }
            loading.set(false);
        });
    };

    view! {
        <div class="panel">
            // Drop zone
            <label class="file-drop">
                {move || match file.get() {
                    Some(f) => view! {
                        <span class="file-name">"📄 " {f.name()}</span>
                    }.into_any(),
                    None => view! {
                        <span>"Click to select CSV / XLS file"</span>
                    }.into_any(),
                }}
                <input type="file" accept=".csv,.xls,.xlsx" on:change=on_file />
            </label>

            <div class="btn-row">
                <button
                    class="btn btn-secondary"
                    on:click=on_preview
                    prop:disabled=move || file.get().is_none() || loading.get()
                >"Preview"</button>
                <button
                    class="btn btn-primary"
                    on:click=on_upload
                    prop:disabled=move || file.get().is_none() || loading.get()
                >
                    {move || if loading.get() {
                        view! { <><span class="spinner"></span>" Uploading..."</> }.into_any()
                    } else {
                        view! { "Upload" }.into_any()
                    }}
                </button>
            </div>

            // Status message
            {move || message.get().map(|(ok, txt)| view! {
                <div class=if ok { "msg-ok" } else { "msg-err" }>{txt}</div>
            })}

            // CSV preview table
            {move || preview.get().map(|p| view! {
                <p class="preview-count">"Showing first 5 of " {p.total} " rows"</p>
                <div class="tbl-wrap">
                    <table class="csv">
                        <thead>
                            <tr>{p.headers.iter().map(|h| view! { <th>{h.clone()}</th> }).collect_view()}</tr>
                        </thead>
                        <tbody>
                            {p.rows.into_iter().map(|row| view! {
                                <tr>{row.into_iter().map(|c| view! { <td>{c}</td> }).collect_view()}</tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            })}
        </div>
    }
}

// ── Admin Payslip Panel ───────────────────────────────────────────────────────

#[component]
fn AdminPayslipPanel() -> impl IntoView {
    let auth    = use_auth();
    let emp_id  = RwSignal::new(String::new());
    let year    = RwSignal::new(String::new());
    let month   = RwSignal::new(String::new());
    let pdf_url = RwSignal::new(Option::<String>::None);
    let error   = RwSignal::new(Option::<String>::None);

    let validate = move || -> Option<(String, i32, i32)> {
        let tok = auth.session.get().map(|s| s.token).unwrap_or_default();
        let y: i32 = year.get().trim().parse().ok()?;
        let m: i32 = month.get().trim().parse().ok()?;
        let eid = emp_id.get();
        if eid.trim().is_empty() || m < 1 || m > 12 { return None; }
        Some((tok, y, m))
    };

    let on_preview = move |_| {
        error.set(None);
        match validate() {
            Some((tok, y, m)) => {
                pdf_url.set(Some(api::admin_preview_url(&tok, y, m, &emp_id.get())));
            }
            None => error.set(Some("Please fill in Employee ID, Year, and Month".into())),
        }
    };

    let on_download = move |_| {
        error.set(None);
        match validate() {
            Some((tok, y, m)) => {
                let url = api::admin_download_url(&tok, y, m, &emp_id.get());
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target(&url, "_blank");
                }
            }
            None => error.set(Some("Please fill in Employee ID, Year, and Month".into())),
        }
    };

    view! {
        <div class="panel">
            <div class="field">
                <label>"Employee ID (UUID)"</label>
                <input
                    placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                    prop:value=move || emp_id.get()
                    on:input=move |ev| emp_id.set(event_target_value(&ev))
                />
            </div>
            <div class="field">
                <label>"Year"</label>
                <input
                    type="number"
                    placeholder="2024"
                    prop:value=move || year.get()
                    on:input=move |ev| year.set(event_target_value(&ev))
                />
            </div>
            <div class="field">
                <label>"Month"</label>
                <select on:change=move |e| month.set(event_target_value(&e))>
                    <option value="">"— Select month —"</option>
                    {months_options()}
                </select>
            </div>

            {move || error.get().map(|e| view! { <div class="msg-err">{e}</div> })}

            <div class="btn-row">
                <button class="btn btn-secondary" on:click=on_preview>"Preview"</button>
                <button class="btn btn-primary"   on:click=on_download>"Download PDF"</button>
            </div>

            {move || pdf_url.get().map(|url| view! {
                <iframe class="pdf-frame" src=url></iframe>
            })}
        </div>
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn months_options() -> impl IntoView {
    let names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    (1u32..=12).map(|m| view! {
        <option value=m.to_string()>{names[(m-1) as usize]}</option>
    }).collect_view()
}
