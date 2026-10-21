use had_tracer_core::AlertEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AlertCounts};
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, StatCard, severity_tone};
use crate::state::{notify, notify_error, use_app};

#[component]
pub fn AlertsPage() -> impl IntoView {
    let ctx = use_app();
    let filter = RwSignal::new("all".to_string());
    let alerts = RwSignal::new(Vec::<AlertEvent>::new());
    let counts = RwSignal::new(None::<AlertCounts>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let override_id = RwSignal::new(None::<String>);
    let override_reason = RwSignal::new(String::new());
    let resolve_id = RwSignal::new(None::<String>);
    let resolve_note = RwSignal::new(String::new());

    Effect::new(move |_| {
        ctx.refresh.get();
        let current = filter.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let status = if current == "all" {
                None
            } else {
                Some(current)
            };
            let list_result = api::list_alert_events(status, Some(150)).await;
            let counts_result = api::alert_counts().await;
            match (list_result, counts_result) {
                (Ok(items), Ok(summary)) => {
                    alerts.set(items);
                    counts.set(Some(summary));
                }
                (Err(e), _) | (_, Err(e)) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    let refresh = move || ctx.refresh.update(|value| *value += 1);

    view! {
        <div class="page">
            <PageHeader
                title="Alert Board"
                subtitle="ตรวจสอบผลแล็บที่เข้าเงื่อนไข HAD rule พร้อมหลักฐานและ action ที่แนะนำ"
            >
                <button class="btn btn-secondary" on:click=move |_| refresh()>"รีเฟรช"</button>
            </PageHeader>

            {move || counts.get().map(|counts| view! {
                <div class="stat-grid stat-grid-4">
                    <StatCard label="ใหม่" value=counts.new.to_string() hint="ยังไม่รับทราบ" tone="orange" />
                    <StatCard label="วิกฤต" value=counts.critical.to_string() hint="ต้องรีบตรวจ" tone="red" />
                    <StatCard label="รับทราบแล้ว" value=counts.acknowledged.to_string() hint="รอดำเนินการ" tone="teal" />
                    <StatCard label="แก้ไขแล้ว" value=counts.resolved.to_string() hint="ปิด loop" tone="green" />
                </div>
            })}

            <div class="tabs">
                {["all", "new", "acknowledged", "overridden", "resolved"].into_iter().map(|value| {
                    view! {
                        <button
                            class=move || if filter.get() == value { "tab active" } else { "tab" }
                            on:click=move |_| filter.set(value.to_string())
                        >{filter_label(value)}</button>
                    }
                }).collect_view()}
            </div>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <Show when=move || loading.get()>
                <Card><Spinner label="กำลังโหลด alert" /></Card>
            </Show>

            <Show when=move || !loading.get()>
                <Card class="table-card">
                    {move || {
                        let items = alerts.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ไม่มี alert ในมุมมองนี้" hint="เปิดใช้ rule และกดประเมิน alerts จากหน้า patient cockpit" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead>
                                            <tr>
                                                <th>"Severity"</th><th>"ผู้ป่วย / ยา"</th><th>"หลักฐาน"</th>
                                                <th>"Rule"</th><th>"เวลา"</th><th>"Status"</th><th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.into_iter().map(|alert| {
                                                let hn = alert.hn.clone();
                                                let severity = alert.severity;
                                                let status = alert.status;
                                                let status_str = status.as_str();
                                                let status_label = status.label_th();
                                                let id_ack = alert.id.clone();
                                                let id_override = alert.id.clone();
                                                let id_resolve = alert.id.clone();
                                                view! {
                                                    <tr>
                                                        <td><Badge tone=severity_tone(severity)>{severity.label_th()}</Badge></td>
                                                        <td>
                                                            <button class="link-btn strong" on:click=move |_| ctx.open_patient(hn.clone())>
                                                                {format!("HN {}", alert.hn)}
                                                            </button>
                                                            <div class="muted">{alert.drug_name.clone().unwrap_or_else(|| "-".into())}</div>
                                                        </td>
                                                        <td>
                                                            <div class="mono strong">{format!("{} {}", alert.raw_value, alert.unit.clone().unwrap_or_default())}</div>
                                                            <div class="muted">{alert.lab_items_name.clone().unwrap_or_else(|| alert.lab_items_code.clone())}</div>
                                                        </td>
                                                        <td>{alert.rule_name.clone()}</td>
                                                        <td class="mono">{short_time(&alert.created_at)}</td>
                                                        <td><span class=format!("status-pill status-{}", status_str)>{status_label}</span></td>
                                                        <td class="actions">
                                                            <Show when=move || status == had_tracer_core::AlertStatus::New>
                                                                <button class="btn btn-secondary btn-sm" on:click={
                                                                    let id = id_ack.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn_local(async move {
                                                                            match api::acknowledge_alert(id).await {
                                                                                Ok(()) => { notify(&ctx, "รับทราบ alert แล้ว"); refresh(); }
                                                                                Err(e) => notify_error(&ctx, e),
                                                                            }
                                                                        });
                                                                    }
                                                                }>"รับทราบ"</button>
                                                            </Show>
                                                            <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                                override_id.set(Some(id_override.clone()));
                                                                override_reason.set(String::new());
                                                            }>"Override"</button>
                                                            <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                                resolve_id.set(Some(id_resolve.clone()));
                                                                resolve_note.set(String::new());
                                                            }>"แก้ไข"</button>
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

            <Show when=move || override_id.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| override_id.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-head">
                            <div><div class="eyebrow">"OVERRIDE"</div><h3>"ระบุเหตุผลที่ไม่ดำเนินการตาม alert"</h3></div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| override_id.set(None)>"ปิด"</button>
                        </div>
                        <form class="form" on:submit=move |ev| {
                            ev.prevent_default();
                            let Some(id) = override_id.get() else { return };
                            let reason = override_reason.get();
                            spawn_local(async move {
                                match api::override_alert(id, reason).await {
                                    Ok(()) => { override_id.set(None); notify(&ctx, "บันทึก override แล้ว"); refresh(); }
                                    Err(e) => notify_error(&ctx, e),
                                }
                            });
                        }>
                            <label class="field">
                                <span>"เหตุผล"</span>
                                <textarea class="input textarea" prop:value=move || override_reason.get()
                                    on:input=move |ev| override_reason.set(event_target_value(&ev))
                                    placeholder="เช่น ค่า lab ซ้ำซ้อน, ผู้ป่วยมีแผนรับยาอื่น, หรือแพทย์รับทราบแล้ว" />
                            </label>
                            <div class="modal-actions">
                                <button type="button" class="btn btn-secondary" on:click=move |_| override_id.set(None)>"ยกเลิก"</button>
                                <button type="submit" class="btn btn-primary">"บันทึก Override"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>

            <Show when=move || resolve_id.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| resolve_id.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-head">
                            <div><div class="eyebrow">"RESOLVE"</div><h3>"บันทึกการแก้ไข"</h3></div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| resolve_id.set(None)>"ปิด"</button>
                        </div>
                        <form class="form" on:submit=move |ev| {
                            ev.prevent_default();
                            let Some(id) = resolve_id.get() else { return };
                            let note = resolve_note.get();
                            spawn_local(async move {
                                match api::resolve_alert(id, note).await {
                                    Ok(()) => { resolve_id.set(None); notify(&ctx, "ปิด alert แล้ว"); refresh(); }
                                    Err(e) => notify_error(&ctx, e),
                                }
                            });
                        }>
                            <label class="field">
                                <span>"สิ่งที่ดำเนินการ"</span>
                                <textarea class="input textarea" prop:value=move || resolve_note.get()
                                    on:input=move |ev| resolve_note.set(event_target_value(&ev))
                                    placeholder="เช่น แจ้งแพทย์แล้ว ปรับขนาดยา และนัดตรวจ INR ซ้ำ" />
                            </label>
                            <div class="modal-actions">
                                <button type="button" class="btn btn-secondary" on:click=move |_| resolve_id.set(None)>"ยกเลิก"</button>
                                <button type="submit" class="btn btn-primary">"ปิด Alert"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn filter_label(value: &str) -> &'static str {
    match value {
        "new" => "ใหม่",
        "acknowledged" => "รับทราบ",
        "overridden" => "Override",
        "resolved" => "แก้ไขแล้ว",
        _ => "ทั้งหมด",
    }
}

fn short_time(value: &str) -> String {
    value.chars().take(16).collect()
}
