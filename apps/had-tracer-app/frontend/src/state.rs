use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, ConnectionHealth};

/// Client-side page. This is a desktop app, so navigation is signal-based
/// instead of URL-based. That avoids deep-link and asset-path issues in the
/// Tauri webview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Patients,
    PatientCockpit(String),
    Alerts,
    Rules,
    HadCatalog,
    Tracers,
    TracerDetail(String),
    Findings,
    Reports,
    Settings,
}

impl Page {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "ภาพรวม",
            Self::Patients | Self::PatientCockpit(_) => "ผู้ป่วย",
            Self::Alerts => "Alert Board",
            Self::Rules => "Alert Rules",
            Self::HadCatalog => "HAD Catalog",
            Self::Tracers | Self::TracerDetail(_) => "Clinical Tracer",
            Self::Findings => "Findings & CAPA",
            Self::Reports => "รายงานและตัวชี้วัด",
            Self::Settings => "ตั้งค่าและสำรองข้อมูล",
        }
    }

    pub fn section(&self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Patients | Self::PatientCockpit(_) => "patients",
            Self::Alerts => "alerts",
            Self::Rules => "rules",
            Self::HadCatalog => "had",
            Self::Tracers | Self::TracerDetail(_) => "tracers",
            Self::Findings => "findings",
            Self::Reports => "reports",
            Self::Settings => "settings",
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppContext {
    pub page: RwSignal<Page>,
    pub selected_hn: RwSignal<String>,
    pub selected_case: RwSignal<String>,
    pub health: RwSignal<ConnectionHealth>,
    pub site_name: RwSignal<String>,
    pub toast: RwSignal<Option<String>>,
    pub refresh: RwSignal<u32>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            page: RwSignal::new(Page::Dashboard),
            selected_hn: RwSignal::new(String::new()),
            selected_case: RwSignal::new(String::new()),
            health: RwSignal::new(ConnectionHealth::Unconfigured),
            site_name: RwSignal::new("HAD Tracer".to_string()),
            toast: RwSignal::new(None),
            refresh: RwSignal::new(0),
        }
    }

    pub fn open_patient(&self, hn: String) {
        self.selected_hn.set(hn.clone());
        self.page.set(Page::PatientCockpit(hn));
    }

    pub fn open_tracer(&self, id: String) {
        self.selected_case.set(id.clone());
        self.page.set(Page::TracerDetail(id));
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_app() -> AppContext {
    use_context::<AppContext>().expect("invariant: AppContext is provided at the root")
}

pub fn notify(ctx: &AppContext, message: impl Into<String>) {
    let message = message.into();
    ctx.toast.set(Some(message));
    let toast = ctx.toast;
    spawn_local(async move {
        api::sleep_ms(4_000).await;
        toast.set(None);
    });
}

pub fn notify_error(ctx: &AppContext, error: api::ApiError) {
    notify(ctx, format!("{}: {}", error.kind, error.message));
}
