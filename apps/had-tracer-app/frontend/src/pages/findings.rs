use had_tracer_core::{Capa, Finding};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, finding_severity_tone};
use crate::state::{notify, notify_error, use_app};

#[component]
pub fn FindingsPage() -> impl IntoView {
    let ctx = use_app();
    let findings = RwSignal::new(Vec::<Finding>::new());
    let capas = RwSignal::new(Vec::<Capa>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let findings_result = api::list_findings(None).await;
            let capas_result = api::list_capas(None).await;
            match (findings_result, capas_result) {
                (Ok(f), Ok(c)) => {
                    findings.set(f);
                    capas.set(c);
                }
                (Err(e), _) | (_, Err(e)) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    let reload = move || ctx.refresh.update(|value| *value += 1);

    view! {
        <div class="page">
            <PageHeader
                title="Findings & CAPA"
                subtitle="รวมช่องว่างจากทุก tracer พร้อมเจ้าภาพ วันครบกำหนด และการทวนสอบผล"
            >
                <button class="btn btn-secondary" on:click=move |_| reload()>"รีเฟรช"</button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || loading.get()><Card><Spinner label="กำลังโหลด findings" /></Card></Show>

            <Show when=move || !loading.get()>
                <Card class="table-card">
                    <div class="table-head">
                        <div><div class="eyebrow">"FINDING REGISTER"</div><h3>{move || format!("{} รายการ", findings.get().len())}</h3></div>
                    </div>
                    {move || {
                        let items = findings.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี finding" hint="เปิด tracer และบันทึกช่องว่างจาก worksheet" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead><tr><th>"Severity"</th><th>"Finding"</th><th>"Case"</th><th>"Risk"</th><th>"Cause"</th><th>"Status"</th></tr></thead>
                                        <tbody>
                                            {items.into_iter().map(|finding| {
                                                let id = finding.id.clone();
                                                let severity = finding.severity;
                                                let status = finding.status;
                                                view! {
                                                    <tr>
                                                        <td><Badge tone=finding_severity_tone(severity)>{severity.label_th()}</Badge></td>
                                                        <td>
                                                            <div class="strong">{finding.title.clone()}</div>
                                                            <div class="muted">{finding.description.clone()}</div>
                                                        </td>
                                                        <td class="mono">{finding.case_id.clone()}</td>
                                                        <td class="mono">{finding.risk_score}</td>
                                                        <td>{finding.cause_category.label_th()}</td>
                                                        <td>
                                                            <select class="input input-sm" prop:value=status.as_str()
                                                                on:change={
                                                                    let id = id.clone();
                                                                    move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        let id = id.clone();
                                                                        spawn_local(async move {
                                                                            match api::update_finding_status(id, value).await {
                                                                                Ok(()) => { notify(&ctx, "อัปเดต finding แล้ว"); reload(); }
                                                                                Err(e) => notify_error(&ctx, e),
                                                                            }
                                                                        });
                                                                    }
                                                                }>
                                                                <option value="open">"Open"</option>
                                                                <option value="assigned">"Assigned"</option>
                                                                <option value="in_progress">"In progress"</option>
                                                                <option value="implemented">"Implemented"</option>
                                                                <option value="verified">"Verified"</option>
                                                                <option value="closed">"Closed"</option>
                                                                <option value="reopened">"Reopened"</option>
                                                            </select>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }}
                </Card>

                <Card class="table-card">
                    <div class="table-head">
                        <div><div class="eyebrow">"CAPA REGISTER"</div><h3>{move || format!("{} actions", capas.get().len())}</h3></div>
                    </div>
                    {move || {
                        let items = capas.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี CAPA" hint="สร้าง CAPA จาก tracer workspace" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead><tr><th>"Action"</th><th>"Owner"</th><th>"Due"</th><th>"Status"</th><th>"Effectiveness"</th></tr></thead>
                                        <tbody>
                                            {items.into_iter().map(|capa| {
                                                let id = capa.id.clone();
                                                let status = capa.status;
                                                view! {
                                                    <tr>
                                                        <td class="strong">{capa.action.clone()}</td>
                                                        <td>{capa.owner.clone()}</td>
                                                        <td class="mono">{capa.due_date.clone().unwrap_or_else(|| "-".into())}</td>
                                                        <td>{capa.status.label_th()}</td>
                                                        <td>
                                                            <div class="inline-form">
                                                                <input class="input input-sm" placeholder="ผลทวนสอบ"
                                                                    prop:value=capa.effectiveness_result.clone()
                                                                    on:input=move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        capas.update(|items| {
                                                                            if let Some(target) = items.iter_mut().find(|item| item.id == id) {
                                                                                target.effectiveness_result = value.clone();
                                                                            }
                                                                        });
                                                                    } />
                                                                <button class="btn btn-secondary btn-sm" on:click={
                                                                    let id = capa.id.clone();
                                                                    move |_| {
                                                                        let Some(target) = capas.get().into_iter().find(|item| item.id == id) else { return };
                                                                        spawn_local(async move {
                                                                            match api::update_capa_status(target.id, status.as_str().to_string(), target.effectiveness_result).await {
                                                                                Ok(()) => { notify(&ctx, "อัปเดต CAPA แล้ว"); reload(); }
                                                                                Err(e) => notify_error(&ctx, e),
                                                                            }
                                                                        });
                                                                    }
                                                                }>"บันทึก"</button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }}
                </Card>
            </Show>
        </div>
    }
}
