use had_tracer_core::{AlertOperator, AlertRule, HadSeverity};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, LabItemRow};
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, severity_tone};
use crate::state::{notify, notify_error, use_app};

#[component]
pub fn RulesPage() -> impl IntoView {
    let ctx = use_app();
    let rules = RwSignal::new(Vec::<AlertRule>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let show_editor = RwSignal::new(false);
    let editing = RwSignal::new(blank_rule());
    let lab_query = RwSignal::new(String::new());
    let lab_results = RwSignal::new(Vec::<LabItemRow>::new());
    let lab_loading = RwSignal::new(false);

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match api::list_alert_rules().await {
                Ok(items) => rules.set(items),
                Err(e) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    let reload = move || ctx.refresh.update(|value| *value += 1);

    view! {
        <div class="page">
            <PageHeader
                title="Alert Rules"
                subtitle="ผูกยา HAD กับ lab item และ threshold โดยเก็บ version และประวัติการแก้ไข"
            >
                <button class="btn btn-secondary" on:click=move |_| reload()>"รีเฟรช"</button>
                <button class="btn btn-primary" on:click=move |_| {
                    editing.set(blank_rule());
                    lab_results.set(Vec::new());
                    show_editor.set(true);
                }>"เพิ่ม Rule"</button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || loading.get()><Card><Spinner label="กำลังโหลด rules" /></Card></Show>

            <Show when=move || !loading.get()>
                <Card class="table-card">
                    {move || {
                        let items = rules.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี rule" hint="สร้าง rule จาก template แล้ว map lab items code" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead><tr><th>"Rule"</th><th>"ขอบเขต"</th><th>"Lab"</th><th>"เงื่อนไข"</th><th>"Severity"</th><th>"Active"</th><th></th></tr></thead>
                                        <tbody>
                                            {items.into_iter().map(|rule| {
                                                let id = rule.id.clone();
                                                let active = rule.active;
                                                let severity = rule.severity;
                                                let edit_rule = rule.clone();
                                                let threshold = rule.threshold_label();
                                                let scope = match rule.drug_scope_type.as_str() {
                                                    "drug_icode" => format!("icode {}", rule.drug_scope_value.clone().unwrap_or_default()),
                                                    "drug_class" => format!("class {}", rule.drug_scope_value.clone().unwrap_or_default()),
                                                    "any_had" => "HAD ทั้งหมด".to_string(),
                                                    other => other.to_string(),
                                                };
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <div class="strong">{rule.name.clone()}</div>
                                                            <div class="muted">{rule.description.clone()}</div>
                                                        </td>
                                                        <td class="mono">{scope}</td>
                                                        <td>
                                                            <div class="mono strong">{rule.lab_items_code.clone()}</div>
                                                            <div class="muted">{rule.lab_items_name.clone().unwrap_or_default()}</div>
                                                        </td>
                                                        <td class="mono">{threshold}</td>
                                                        <td><Badge tone=severity_tone(severity)>{severity.label_th()}</Badge></td>
                                                        <td>
                                                            <button class=if active { "toggle on" } else { "toggle" }
                                                                on:click={
                                                                    let id = id.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn_local(async move {
                                                                            match api::set_alert_rule_active(id, !active).await {
                                                                                Ok(()) => { notify(&ctx, "อัปเดต rule แล้ว"); reload(); }
                                                                                Err(e) => notify_error(&ctx, e),
                                                                            }
                                                                        });
                                                                    }
                                                                }><span></span></button>
                                                        </td>
                                                        <td class="actions">
                                                            <button class="btn btn-secondary btn-sm" on:click={
                                                                let rule = edit_rule.clone();
                                                                move |_| {
                                                                    editing.set(rule.clone());
                                                                    lab_results.set(Vec::new());
                                                                    show_editor.set(true);
                                                                }
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

            <Show when=move || show_editor.get()>
                <div class="modal-backdrop" on:click=move |_| show_editor.set(false)>
                    <div class="modal modal-wide" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-head">
                            <div><div class="eyebrow">"ALERT RULE"</div><h3>"แก้ไข rule"</h3></div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_editor.set(false)>"ปิด"</button>
                        </div>
                        <form class="form form-grid" on:submit=move |ev| {
                            ev.prevent_default();
                            spawn_local(async move {
                                match api::save_alert_rule(editing.get()).await {
                                    Ok(_) => { show_editor.set(false); notify(&ctx, "บันทึก rule แล้ว"); reload(); }
                                    Err(e) => notify_error(&ctx, e),
                                }
                            });
                        }>
                            <label class="field field-full"><span>"ชื่อ rule"</span>
                                <input class="input" prop:value=move || editing.get().name
                                    on:input=move |ev| editing.update(|rule| rule.name = event_target_value(&ev)) />
                            </label>
                            <label class="field field-full"><span>"คำอธิบาย"</span>
                                <input class="input" prop:value=move || editing.get().description
                                    on:input=move |ev| editing.update(|rule| rule.description = event_target_value(&ev)) />
                            </label>
                            <label class="field"><span>"ขอบเขตยา"</span>
                                <select class="input" prop:value=move || editing.get().drug_scope_type
                                    on:change=move |ev| editing.update(|rule| rule.drug_scope_type = event_target_value(&ev))>
                                    <option value="drug_class">"Drug class"</option>
                                    <option value="drug_icode">"Drug icode"</option>
                                    <option value="any_had">"Any HAD"</option>
                                </select>
                            </label>
                            <label class="field"><span>"ค่า scope"</span>
                                <input class="input mono" prop:value=move || editing.get().drug_scope_value.unwrap_or_default()
                                    on:input=move |ev| editing.update(|rule| rule.drug_scope_value = Some(event_target_value(&ev)).filter(|value| !value.trim().is_empty())) />
                            </label>
                            <label class="field"><span>"Lab items code"</span>
                                <input class="input mono" prop:value=move || editing.get().lab_items_code
                                    on:input=move |ev| editing.update(|rule| rule.lab_items_code = event_target_value(&ev)) />
                            </label>
                            <label class="field"><span>"Lab name"</span>
                                <input class="input" prop:value=move || editing.get().lab_items_name.unwrap_or_default()
                                    on:input=move |ev| editing.update(|rule| rule.lab_items_name = Some(event_target_value(&ev))) />
                            </label>
                            <label class="field"><span>"Operator"</span>
                                <select class="input" prop:value=move || editing.get().operator.as_str()
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        editing.update(|rule| rule.operator = AlertOperator::from_str(&value));
                                    }>
                                    {operator_options()}
                                </select>
                            </label>
                            <label class="field"><span>"Threshold ต่ำ / ค่าเดียว"</span>
                                <input class="input" prop:value=move || editing.get().threshold_low.map(|value| value.to_string()).unwrap_or_default()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev).parse::<f64>().ok();
                                        editing.update(|rule| rule.threshold_low = value);
                                    } />
                            </label>
                            <label class="field"><span>"Threshold สูง"</span>
                                <input class="input" prop:value=move || editing.get().threshold_high.map(|value| value.to_string()).unwrap_or_default()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev).parse::<f64>().ok();
                                        editing.update(|rule| rule.threshold_high = value);
                                    } />
                            </label>
                            <label class="field"><span>"Unit"</span>
                                <input class="input" prop:value=move || editing.get().unit.unwrap_or_default()
                                    on:input=move |ev| editing.update(|rule| rule.unit = Some(event_target_value(&ev)).filter(|value| !value.trim().is_empty())) />
                            </label>
                            <label class="field"><span>"Lookback (ชม.)"</span>
                                <input class="input" type="number" prop:value=move || editing.get().lookback_hours.to_string()
                                    on:input=move |ev| editing.update(|rule| rule.lookback_hours = event_target_value(&ev).parse().unwrap_or(72)) />
                            </label>
                            <label class="field"><span>"Severity"</span>
                                <select class="input" prop:value=move || editing.get().severity.as_str()
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        editing.update(|rule| rule.severity = HadSeverity::from_str(&value));
                                    }>
                                    <option value="critical">"Critical"</option><option value="high">"High"</option>
                                    <option value="medium">"Medium"</option><option value="low">"Low"</option>
                                    <option value="info">"Info"</option>
                                </select>
                            </label>
                            <label class="field field-full"><span>"Action"</span>
                                <textarea class="input textarea" prop:value=move || editing.get().action_text
                                    on:input=move |ev| editing.update(|rule| rule.action_text = event_target_value(&ev)) />
                            </label>
                            <label class="field checkbox-field">
                                <input type="checkbox" prop:checked=move || editing.get().active
                                    on:change=move |ev| editing.update(|rule| rule.active = event_target_checked(&ev)) />
                                <span>"เปิดใช้งาน rule นี้"</span>
                            </label>
                            <label class="field checkbox-field">
                                <input type="checkbox" prop:checked=move || editing.get().require_active_order
                                    on:change=move |ev| editing.update(|rule| rule.require_active_order = event_target_checked(&ev)) />
                                <span>"ต้องมี HAD order ที่ยัง active"</span>
                            </label>
                            <div class="field field-full">
                                <span>"ค้น lab items จาก HOSxP"</span>
                                <div class="search-bar">
                                    <input class="input search-input" placeholder="ชื่อ lab หรือ code"
                                        prop:value=move || lab_query.get()
                                        on:input=move |ev| lab_query.set(event_target_value(&ev)) />
                                    <button type="button" class="btn btn-secondary" on:click=move |_| {
                                        lab_loading.set(true);
                                        let query = lab_query.get();
                                        spawn_local(async move {
                                            match api::search_lab_items(query).await {
                                                Ok(items) => lab_results.set(items),
                                                Err(e) => notify_error(&ctx, e),
                                            }
                                            lab_loading.set(false);
                                        });
                                    }>"ค้นหา"</button>
                                </div>
                                <Show when=move || lab_loading.get()><Spinner label="กำลังค้น lab items" /></Show>
                                <div class="list mapping-list">
                                    {move || lab_results.get().into_iter().map(|item| {
                                        let code = item.lab_items_code.clone();
                                        let name = item.lab_items_name.clone();
                                        let unit = item.lab_items_unit.clone();
                                        view! {
                                            <button type="button" class="list-row" on:click={
                                                let code = code.clone();
                                                let name = name.clone();
                                                let unit = unit.clone();
                                                move |_| {
                                                    editing.update(|rule| {
                                                        rule.lab_items_code = code.clone();
                                                        rule.lab_items_name = Some(name.clone());
                                                        rule.unit = unit.clone();
                                                    });
                                                }
                                            }>
                                                <span class="tag-code mono">{item.lab_items_code.clone()}</span>
                                                <div class="list-main">
                                                    <div class="list-title">{item.lab_items_name.clone()}</div>
                                                    <div class="list-sub">{item.lab_items_unit.clone().unwrap_or_default()}</div>
                                                </div>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                            <div class="modal-actions field-full">
                                <button type="button" class="btn btn-secondary" on:click=move |_| show_editor.set(false)>"ยกเลิก"</button>
                                <button type="submit" class="btn btn-primary">"บันทึก Rule"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn operator_options() -> impl IntoView {
    [
        AlertOperator::Gt,
        AlertOperator::Ge,
        AlertOperator::Lt,
        AlertOperator::Le,
        AlertOperator::Eq,
        AlertOperator::Between,
        AlertOperator::Outside,
    ]
    .into_iter()
    .map(|operator| view! { <option value=operator.as_str()>{operator.label_th()}</option> })
    .collect_view()
}

fn blank_rule() -> AlertRule {
    AlertRule {
        id: String::new(),
        name: String::new(),
        description: String::new(),
        drug_scope_type: "drug_class".to_string(),
        drug_scope_value: None,
        lab_items_code: String::new(),
        lab_items_name: None,
        operator: AlertOperator::Gt,
        threshold_low: Some(0.0),
        threshold_high: None,
        unit: None,
        lookback_hours: 72,
        severity: HadSeverity::High,
        action_text: String::new(),
        require_active_order: true,
        active: false,
        version: 1,
        created_by: None,
        approved_by: None,
        approved_at: None,
        effective_at: None,
        review_due_at: None,
    }
}
