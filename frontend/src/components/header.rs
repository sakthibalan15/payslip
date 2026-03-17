// frontend/src/components/header.rs
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::store::use_auth;

#[component]
pub fn AppHeader(
    #[prop(default = false)] show_logout: bool,
) -> impl IntoView {
    let auth     = use_auth();
    let navigate = use_navigate();

    let on_logout = move |_| {
        auth.set_session.set(None);
        navigate("/login", Default::default());
    };

    let logout_cb = on_logout.clone();

    view! {
        <header class="app-header">
            <div class="logo">"LOGO"</div>

            <div class="header-titles">
                <h1>"COMPANY NAME"</h1>
                <h2>"PAYSLIP APP"</h2>
            </div>

            {
                move || {
                    let logout_cb = logout_cb.clone();

                    show_logout.then(|| {
                        view! {
                            <button
                                class="logout-btn"
                                on:click=move |_| logout_cb(())
                            >
                                "Logout"
                            </button>
                        }
                    })
                }
            }
        </header>
    }
}
