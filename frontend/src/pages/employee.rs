// frontend/src/pages/employee.rs
use leptos::prelude::*;

use crate::{api, components::header::AppHeader, store::use_auth};

#[component]
pub fn EmployeeDashboard() -> impl IntoView {
    let auth    = use_auth();
    let name    = move || auth.session.get().map(|s| s.name).unwrap_or_else(|| "Employee".into());

    let year    = RwSignal::new(String::new());
    let month   = RwSignal::new(String::new());
    let pdf_url = RwSignal::new(Option::<String>::None);
    let error   = RwSignal::new(Option::<String>::None);

    let validate = move || -> Option<(String, i32, i32)> {
        let tok = auth.session.get().map(|s| s.token).unwrap_or_default();
        let y: i32 = year.get().trim().parse().ok()?;
        let m: i32 = month.get().trim().parse().ok()?;
        if m < 1 || m > 12 { return None; }
        Some((tok, y, m))
    };

    let on_preview = move |_| {
        error.set(None);
        match validate() {
            Some((tok, y, m)) => pdf_url.set(Some(api::emp_preview_url(&tok, y, m))),
            None => error.set(Some("Please select Year and Month".into())),
        }
    };

    let on_download = move |_| {
        error.set(None);
        match validate() {
            Some((tok, y, m)) => {
                let url = api::emp_download_url(&tok, y, m);
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target(&url, "_blank");
                }
            }
            None => error.set(Some("Please select Year and Month".into())),
        }
    };

    view! {
        <AppHeader show_logout=true />
        <div class="dashboard">
            <div class="welcome">"Hello " {name} "!!"</div>

            <div class="panel">
                <div class="field">
                    <label>"Year"</label>
                    <input
                        type="number"
                        placeholder="2024"
                        prop:value=move || year.get()
                        on:input=move |e| year.set(event_target_value(&e))
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
        </div>
    }
}

fn months_options() -> impl IntoView {
    let names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    (1u32..=12).map(|m| view! {
        <option value=m.to_string()>{names[(m-1) as usize]}</option>
    }).collect_view()
}
