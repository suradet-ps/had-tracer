use leptos::prelude::*;

#[component]
pub fn Card(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! { <section class=format!("card {class}")>{children()}</section> }
}

#[component]
pub fn Badge(#[prop(into)] tone: String, children: Children) -> impl IntoView {
    view! { <span class=format!("badge badge-{tone}")>{children()}</span> }
}

#[component]
pub fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] hint: String,
    #[prop(into)] tone: String,
) -> impl IntoView {
    view! {
        <div class=format!("stat-card stat-{tone}")>
            <div class="stat-label">{label}</div>
            <div class="stat-value">{value}</div>
            <div class="stat-hint">{hint}</div>
        </div>
    }
}

#[component]
pub fn PageHeader(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    children: Children,
) -> impl IntoView {
    view! {
        <header class="page-header">
            <div>
                <div class="eyebrow">HAD CLINICAL TRACER</div>
                <h1>{title}</h1>
                <p>{subtitle}</p>
            </div>
            <div class="page-actions">{children()}</div>
        </header>
    }
}

#[component]
pub fn EmptyState(#[prop(into)] title: String, #[prop(into)] hint: String) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-mark">{"◇"}</div>
            <h3>{title}</h3>
            <p>{hint}</p>
        </div>
    }
}

#[component]
pub fn Spinner(#[prop(into)] label: String) -> impl IntoView {
    view! {
        <div class="spinner-row" role="status" aria-live="polite">
            <span class="spinner"></span>
            <span>{label}</span>
        </div>
    }
}

pub fn severity_tone(severity: had_tracer_core::HadSeverity) -> &'static str {
    use had_tracer_core::HadSeverity;
    match severity {
        HadSeverity::Critical => "critical",
        HadSeverity::High => "high",
        HadSeverity::Medium => "medium",
        HadSeverity::Low => "low",
        HadSeverity::Info => "info",
    }
}

pub fn finding_severity_tone(severity: had_tracer_core::FindingSeverity) -> &'static str {
    use had_tracer_core::FindingSeverity;
    match severity {
        FindingSeverity::Critical => "critical",
        FindingSeverity::High => "high",
        FindingSeverity::Medium => "medium",
        FindingSeverity::Low => "low",
    }
}
