use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AlertEvent, DashboardMetrics, TracerCase};
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, StatCard, severity_tone};
use crate::state::{Page, notify_error, use_app};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let ctx = use_app();
    let metrics = RwSignal::new(None::<DashboardMetrics>);
    let alerts = RwSignal::new(Vec::<AlertEvent>::new());
    let tracers = RwSignal::new(Vec::<TracerCase>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let metrics_result = api::dashboard_metrics().await;
            let alerts_result = api::list_alert_events(None, Some(6)).await;
            let tracers_result = api::list_tracer_cases(Some(6)).await;
            match (metrics_result, alerts_result, tracers_result) {
                (Ok(m), Ok(a), Ok(t)) => {
                    metrics.set(Some(m));
                    alerts.set(a);
                    tracers.set(t);
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    error.set(Some(e.message));
                }
            }
            loading.set(false);
        });
    });

    view! {
        <div class="page">
            <PageHeader
                title="HAD Clinical Tracer"
                subtitle="ตามรอยการใช้ยากลุ่มเสี่ยงสูงด้วยข้อมูลจริงจาก HOSxP และหลักฐานจากหน้างาน"
            >
                <button class="btn btn-primary" on:click=move |_| ctx.page.set(Page::Patients)>
                    "เริ่มค้นหาผู้ป่วย"
                </button>
                <button class="btn btn-secondary" on:click=move |_| ctx.page.set(Page::Alerts)>
                    "ดู Alert ทั้งหมด"
                </button>
            </PageHeader>

            <section class="hero-band">
                <div class="hero-copy">
                    <div class="hero-eyebrow">"PATIENT, PERSONNEL, PEOPLE SAFETY"</div>
                    <h2>"One tracer. Every handoff."</h2>
                    <p>
                        "เชื่อม HAD order, laboratory alert, และ tracer worksheet ไว้ในที่เดียว"
                        " เพื่อให้เห็นว่าระบบพังตรงไหน และพิสูจน์ได้ว่าการแก้ไขได้ผลจริง"
                    </p>
                    <div class="hero-actions">
                        <button class="btn btn-primary" on:click=move |_| ctx.page.set(Page::HadCatalog)>
                            "จัดการ HAD Catalog"
                        </button>
                        <button class="btn btn-secondary-on-dark" on:click=move |_| ctx.page.set(Page::Tracers)>
                            "เปิด Tracer Workspace"
                        </button>
                    </div>
                </div>
                <div class="code-mockup">
                    <div class="code-dots"><span></span><span></span><span></span></div>
                    <pre>{r#"{
  "tracer": "HAD-Warfarin-001",
  "phase": "monitoring",
  "signal": "INR 6.2",
  "action": "hold + vitamin K review",
  "finding": "no repeat INR after dose change",
  "next": "re-trace in 14 days"
}"#}</pre>
                </div>
            </section>

            <Show when=move || loading.get()>
                <Card><Spinner label="กำลังโหลดภาพรวม" /></Card>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">
                    {move || error.get().unwrap_or_default()}
                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                        if let Some(message) = error.get() {
                            let fake = api::ApiError::new("error", message);
                            notify_error(&ctx, fake);
                        }
                    }>"ปิด"</button>
                </div>
            </Show>

            {move || metrics.get().map(|m| view! {
                <div class="stat-grid">
                    <StatCard label="HAD ทั้งหมด" value=m.had_total.to_string()
                        hint=format!("map แล้ว {} รายการ", m.had_mapped) tone="teal" />
                    <StatCard label="Rules ที่เปิดใช้" value=m.active_rules.to_string()
                        hint="ตรวจสอบ lab alert" tone="green" />
                    <StatCard label="Alert ใหม่" value=m.open_alerts.to_string()
                        hint=format!("วิกฤต {}", m.critical_alerts) tone="orange" />
                    <StatCard label="Tracer เปิดอยู่" value=m.tracer_open.to_string()
                        hint="individual และ system" tone="purple" />
                    <StatCard label="Findings ค้าง" value=m.findings_open.to_string()
                        hint="ต้องมี owner และ due date" tone="orange" />
                    <StatCard label="CAPA เกินกำหนด" value=m.capa_overdue.to_string()
                        hint="ติดตามใน dashboard" tone="red" />
                </div>
            })}

            <div class="grid-2">
                <Card class="panel">
                    <div class="panel-head">
                        <div>
                            <div class="eyebrow">"ALERT BOARD"</div>
                            <h3>"สัญญาณล่าสุด"</h3>
                        </div>
                        <button class="btn btn-ghost btn-sm" on:click=move |_| ctx.page.set(Page::Alerts)>
                            "ดูทั้งหมด"
                        </button>
                    </div>
                    {move || {
                        let items = alerts.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี alert" hint="ตรวจสอบ HAD mapping และเปิดใช้ rule ก่อน" /> }.into_any()
                        } else {
                            view! {
                                <div class="list">
                                    {items.into_iter().map(|alert| {
                                        let hn = alert.hn.clone();
                                        view! {
                                            <button class="list-row" on:click=move |_| ctx.open_patient(hn.clone())>
                                                <Badge tone=severity_tone(alert.severity)>{alert.severity.label_th()}</Badge>
                                                <div class="list-main">
                                                    <div class="list-title">{alert.drug_name.clone().unwrap_or_else(|| "-".into())}</div>
                                                    <div class="list-sub">{format!("{} {} {}", alert.lab_items_name.clone().unwrap_or_default(), alert.raw_value, alert.unit.clone().unwrap_or_default())}</div>
                                                </div>
                                                <div class="list-meta">{alert.hn.clone()}<br/><span>{short_time(&alert.created_at)}</span></div>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </Card>

                <Card class="panel">
                    <div class="panel-head">
                        <div>
                            <div class="eyebrow">"TRACERS"</div>
                            <h3>"เคสล่าสุด"</h3>
                        </div>
                        <button class="btn btn-ghost btn-sm" on:click=move |_| ctx.page.set(Page::Tracers)>
                            "ดูทั้งหมด"
                        </button>
                    </div>
                    {move || {
                        let items = tracers.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี tracer" hint="สร้างเคสจากผู้ป่วยหรือจาก alert" /> }.into_any()
                        } else {
                            view! {
                                <div class="list">
                                    {items.into_iter().map(|case| {
                                        let id = case.id.clone();
                                        view! {
                                            <button class="list-row" on:click=move |_| ctx.open_tracer(id.clone())>
                                                <span class="case-type">{case.case_type.label_th()}</span>
                                                <div class="list-main">
                                                    <div class="list-title">{case.title}</div>
                                                    <div class="list-sub">{format!("HN {} · {}", case.hn, short_time(&case.opened_at))}</div>
                                                </div>
                                                <span class=format!("status-pill status-{}", case.status.as_str())>{case.status.label_th()}</span>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </Card>
            </div>

            <div class="grid-4">
                <Card class="dimension-card">
                    <div class="dimension-tag">"STRUCTURE"</div>
                    <h4>"พร้อมหรือยัง"</h4>
                    <p>"HAD list, safety bundle, competency, storage และ antidote"</p>
                </Card>
                <Card class="dimension-card">
                    <div class="dimension-tag">"PROCESS"</div>
                    <h4>"ทำจริงหรือไม่"</h4>
                    <p>"pharmacist review, double check, monitoring และ handoff"</p>
                </Card>
                <Card class="dimension-card">
                    <div class="dimension-tag">"OUTCOME"</div>
                    <h4>"ผลลัพธ์เป็นอย่างไร"</h4>
                    <p>"ADE, near miss, harm severity และ trigger ที่ตามรอยได้"</p>
                </Card>
                <Card class="dimension-card">
                    <div class="dimension-tag">"BALANCING"</div>
                    <h4>" burdens เท่าไร"</h4>
                    <p>"alert override, alert burden และเวลาที่เพิ่มขึ้น"</p>
                </Card>
            </div>
        </div>
    }
}

fn short_time(value: &str) -> String {
    value.chars().take(16).collect()
}
