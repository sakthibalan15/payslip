// frontend/src/lib.rs
// Leptos 0.8 — uses `leptos::prelude::*` and leptos_router 0.8

mod api;
mod components;
mod pages;
mod store;

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use pages::{
    auth::{LoginPage, OtpPage},
    admin::AdminDashboard,
    employee::EmployeeDashboard,
    not_found::NotFound,
};
use store::{provide_auth, use_auth};

#[component]
pub fn App() -> impl IntoView {
    provide_auth();

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/")         view=RootRedirect />
                <Route path=path!("/login")    view=LoginPage />
                <Route path=path!("/otp")      view=OtpPage />
                <Route path=path!("/admin")    view=move || view! {
                    <Protected required_role="admin">
                        <AdminDashboard />
                    </Protected>
                }/>
                <Route path=path!("/employee") view=move || view! {
                    <Protected required_role="employee">
                        <EmployeeDashboard />
                    </Protected>
                }/>
            </Routes>
        </Router>
    }
}

/// Redirect / to the correct dashboard based on stored session, or to /login
#[component]
fn RootRedirect() -> impl IntoView {
    let auth     = use_auth();
    let navigate = leptos_router::hooks::use_navigate();

    // Leptos 0.8: Effect::new replaces create_effect
    Effect::new(move |_| {
        let dest = match auth.session.get() {
            Some(s) => match s.role {
                shared::UserRole::Admin    => "/admin",
                shared::UserRole::Employee => "/employee",
            },
            None => "/login",
        };
        navigate(dest, Default::default());
    });

    view! { <p style="padding:40px;color:#888">"Redirecting..." </p> }
}

/// Guard component — redirects if not logged in or wrong role
#[component]
fn Protected(required_role: &'static str, children: Children) -> impl IntoView {
    let auth     = use_auth();
    let navigate = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        match auth.session.get() {
            None => { navigate("/login", Default::default()); }
            Some(s) => {
                let ok = match s.role {
                    shared::UserRole::Admin    => required_role == "admin",
                    shared::UserRole::Employee => required_role == "employee",
                };
                if !ok { navigate("/login", Default::default()); }
            }
        }
    });

    children()
}

/// WASM entry point
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() { leptos::mount::mount_to_body(App); }