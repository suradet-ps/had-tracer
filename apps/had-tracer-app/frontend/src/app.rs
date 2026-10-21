use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, ConnectionHealth};
use crate::pages;
use crate::state::{AppContext, Page, use_app};

pub fn run() {
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let ctx = AppContext::new();
    provide_context(ctx);

    spawn_local(async move {
        let _ = api::show_main_window().await;
        if let Ok(status) = api::get_app_status().await
            && !status.site_name.trim().is_empty()
        {
            ctx.site_name.set(status.site_name);
        }
    });

    let health_ctx = ctx;
    spawn_local(async move {
        loop {
            if let Ok(health) = api::connection_health().await {
                health_ctx.health.set(health);
            }
            api::sleep_ms(30_000).await;
        }
    });

    view! {
        <div class="app-shell">
            <Sidebar />
            <div class="app-main">
                <TopBar />
                <main class="content">
                    {move || match ctx.page.get() {
                        Page::Dashboard => view! { <pages::dashboard::DashboardPage /> }.into_any(),
                        Page::Patients => view! { <pages::patients::PatientsPage /> }.into_any(),
                        Page::PatientCockpit(_) => view! { <pages::patients::PatientCockpitPage /> }.into_any(),
                        Page::Alerts => view! { <pages::alerts::AlertsPage /> }.into_any(),
                        Page::Rules => view! { <pages::rules::RulesPage /> }.into_any(),
                        Page::HadCatalog => view! { <pages::had::HadCatalogPage /> }.into_any(),
                        Page::Tracers => view! { <pages::tracers::TracersPage /> }.into_any(),
                        Page::TracerDetail(_) => view! { <pages::tracers::TracerDetailPage /> }.into_any(),
                        Page::Findings => view! { <pages::findings::FindingsPage /> }.into_any(),
                        Page::Reports => view! { <pages::reports::ReportsPage /> }.into_any(),
                        Page::Settings => view! { <pages::settings::SettingsPage /> }.into_any(),
                    }}
                </main>
            </div>
            <ToastHost />
        </div>
    }
}

#[component]
fn Sidebar() -> impl IntoView {
    let ctx = use_app();
    let nav = [
        ("dashboard", "DB", "ภาพรวม", Page::Dashboard),
        ("patients", "PT", "ผู้ป่วย", Page::Patients),
        ("alerts", "AL", "Alerts", Page::Alerts),
        ("rules", "RL", "Rules", Page::Rules),
        ("had", "HD", "HAD Catalog", Page::HadCatalog),
        ("tracers", "TR", "Tracers", Page::Tracers),
        ("findings", "FN", "Findings", Page::Findings),
        ("reports", "RP", "Reports", Page::Reports),
        ("settings", "ST", "Settings", Page::Settings),
    ];

    view! {
        <aside class="sidebar">
            <div class="brand">
                <div class="brand-mark">"H"</div>
                <div>
                    <div class="brand-name">"HAD Tracer"</div>
                    <div class="brand-sub">"Clinical tracer workspace"</div>
                </div>
            </div>

            <nav class="nav">
                {nav.into_iter().map(|(section, tag, label, page)| {
                    view! {
                        <button
                            class=move || if ctx.page.get().section() == section { "nav-item active" } else { "nav-item" }
                            on:click=move |_| ctx.page.set(page.clone())
                        >
                            <span class=format!("nav-tag tag-{section}")>{tag}</span>
                            <span>{label}</span>
                        </button>
                    }
                }).collect_view()}
            </nav>

            <div class="sidebar-footer">
                <div class="health-line">
                    <HealthDot />
                    <span>{move || health_label(ctx.health.get())}</span>
                </div>
                <div class="site-line">{move || ctx.site_name.get()}</div>
                <div class="version-line">{format!("v{}", env!("CARGO_PKG_VERSION"))}</div>
            </div>
        </aside>
    }
}

#[component]
fn TopBar() -> impl IntoView {
    let ctx = use_app();
    view! {
        <header class="topbar">
            <div>
                <div class="topbar-eyebrow">{move || ctx.page.get().title()}</div>
                <div class="topbar-title">{move || ctx.site_name.get()}</div>
            </div>
            <div class="topbar-actions">
                <div class=move || format!("health-pill {}", health_tone(ctx.health.get()))>
                    <HealthDot />
                    <span>{move || health_label(ctx.health.get())}</span>
                </div>
                <button class="btn btn-secondary btn-sm" on:click=move |_| {
                    ctx.refresh.update(|value| *value += 1);
                }>"รีเฟรช"</button>
            </div>
        </header>
    }
}

#[component]
fn HealthDot() -> impl IntoView {
    let ctx = use_app();
    view! { <span class=move || format!("dot {}", health_tone(ctx.health.get()))></span> }
}

#[component]
fn ToastHost() -> impl IntoView {
    let ctx = use_app();
    view! {
        <Show when=move || ctx.toast.get().is_some()>
            <div class="toast" role="status" aria-live="polite">
                <span>{move || ctx.toast.get().unwrap_or_default()}</span>
                <button class="toast-close" on:click=move |_| ctx.toast.set(None)>"×"</button>
            </div>
        </Show>
    }
}

fn health_tone(health: ConnectionHealth) -> &'static str {
    match health {
        ConnectionHealth::Connected => "ok",
        ConnectionHealth::Connecting => "warn",
        ConnectionHealth::Degraded => "warn",
        ConnectionHealth::Disconnected => "bad",
        ConnectionHealth::Unconfigured => "muted",
    }
}

fn health_label(health: ConnectionHealth) -> &'static str {
    match health {
        ConnectionHealth::Connected => "HOSxP เชื่อมต่อแล้ว",
        ConnectionHealth::Connecting => "กำลังเชื่อมต่อ",
        ConnectionHealth::Degraded => "เชื่อมต่อแบบจำกัด",
        ConnectionHealth::Disconnected => "HOSxP ขาดการเชื่อมต่อ",
        ConnectionHealth::Unconfigured => "ยังไม่ตั้งค่า HOSxP",
    }
}
