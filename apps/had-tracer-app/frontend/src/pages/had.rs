use had_tracer_core::{HadDrug, HadSeverity, SafetyBundle, new_id};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, DrugItemRow};
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, severity_tone};
use crate::state::{notify, notify_error, use_app};

#[component]
pub fn HadCatalogPage() -> impl IntoView {
    let ctx = use_app();
    let drugs = RwSignal::new(Vec::<HadDrug>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let show_editor = RwSignal::new(false);
    let editing = RwSignal::new(blank_drug());
    let show_mapping = RwSignal::new(None::<HadDrug>);
    let drug_query = RwSignal::new(String::new());
    let drug_results = RwSignal::new(Vec::<DrugItemRow>::new());
    let mapping_loading = RwSignal::new(false);

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match api::list_had_drugs().await {
                Ok(items) => drugs.set(items),
                Err(e) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    let reload = move || ctx.refresh.update(|value| *value += 1);

    view! {
        <div class="page">
            <PageHeader
                title="HAD Catalog"
                subtitle="กำหนดรายการยาเสี่ยงสูง ผูกกับ drugitems.icode และเก็บ safety bundle ที่ผ่านการอนุมัติ"
            >
                <button class="btn btn-secondary" on:click=move |_| reload()>"รีเฟรช"</button>
                <button class="btn btn-primary" on:click=move |_| {
                    editing.set(blank_drug());
                    show_editor.set(true);
                }>"เพิ่ม HAD"</button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <Show when=move || loading.get()>
                <Card><Spinner label="กำลังโหลด HAD catalog" /></Card>
            </Show>

            <Show when=move || !loading.get()>
                <Card class="table-card">
                    {move || {
                        let items = drugs.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี HAD" hint="เพิ่มรายการยาและ map กับ drugitems.icode" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead>
                                            <tr>
                                                <th>"ยา"</th><th>"กลุ่ม"</th><th>"Route"</th>
                                                <th>"Severity"</th><th>"Mapping"</th><th>"Safety bundle"</th>
                                                <th>"Version"</th><th>"Active"</th><th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.into_iter().map(|drug| {
                                                let id = drug.id.clone();
                                                let severity = drug.severity;
                                                let mapped = drug.is_mapped();
                                                let monitoring = drug.safety_bundle.monitoring.len();
                                                let double_check = drug.safety_bundle.double_check_required;
                                                let active = drug.active;
                                                let version = drug.version;
                                                let name = drug.name.clone();
                                                let generic = drug.generic_name.clone().unwrap_or_default();
                                                let class = drug.drug_class.clone();
                                                let route = drug.route.clone().unwrap_or_else(|| "-".into());
                                                let icode = drug.icode.clone();
                                                let edit_drug = drug.clone();
                                                let map_drug = drug.clone();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <div class="strong">{name}</div>
                                                            <div class="muted">{generic}</div>
                                                        </td>
                                                        <td>{class}</td>
                                                        <td>{route}</td>
                                                        <td><Badge tone=severity_tone(severity)>{severity.label_th()}</Badge></td>
                                                        <td>
                                                            {if mapped {
                                                                view! { <span class="mono ok-text">{icode.unwrap_or_default()}</span> }.into_any()
                                                            } else {
                                                                view! { <span class="warn-text">"ยังไม่ map"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td>
                                                            <div>{format!("{} monitoring", monitoring)}</div>
                                                            <div class="muted">{if double_check { "double check" } else { "standard check" }}</div>
                                                        </td>
                                                        <td class="mono">"v" {version}</td>
                                                        <td>
                                                            <button
                                                                class=if active { "toggle on" } else { "toggle" }
                                                                on:click={
                                                                    let id = id.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn_local(async move {
                                                                            match api::set_had_drug_active(id, !active).await {
                                                                                Ok(()) => { notify(&ctx, "อัปเดตสถานะ HAD แล้ว"); reload(); }
                                                                                Err(e) => notify_error(&ctx, e),
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            >
                                                                <span></span>
                                                            </button>
                                                        </td>
                                                        <td class="actions">
                                                            <button class="btn btn-secondary btn-sm" on:click={
                                                                let drug = edit_drug.clone();
                                                                move |_| {
                                                                    editing.set(drug.clone());
                                                                    show_editor.set(true);
                                                                }
                                                            }>"แก้ไข"</button>
                                                            <button class="btn btn-ghost btn-sm" on:click={
                                                                let drug = map_drug.clone();
                                                                move |_| {
                                                                    show_mapping.set(Some(drug.clone()));
                                                                    drug_query.set(String::new());
                                                                    drug_results.set(Vec::new());
                                                                }
                                                            }>"Map"</button>
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
                            <div><div class="eyebrow">"HAD SAFETY BUNDLE"</div><h3>"แก้ไขรายการยา"</h3></div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_editor.set(false)>"ปิด"</button>
                        </div>
                        <form class="form form-grid" on:submit=move |ev| {
                            ev.prevent_default();
                            let mut drug = editing.get();
                            if drug.id.trim().is_empty() {
                                drug.id = new_id();
                            }
                            spawn_local(async move {
                                match api::save_had_drug(drug).await {
                                    Ok(_) => { show_editor.set(false); notify(&ctx, "บันทึก HAD แล้ว"); reload(); }
                                    Err(e) => notify_error(&ctx, e),
                                }
                            });
                        }>
                            <label class="field">
                                <span>"ชื่อยา"</span>
                                <input class="input" prop:value=move || editing.get().name
                                    on:input=move |ev| editing.update(|drug| drug.name = event_target_value(&ev)) />
                            </label>
                            <label class="field">
                                <span>"Generic name"</span>
                                <input class="input" prop:value=move || editing.get().generic_name.unwrap_or_default()
                                    on:input=move |ev| editing.update(|drug| drug.generic_name = Some(event_target_value(&ev))) />
                            </label>
                            <label class="field">
                                <span>"กลุ่มยา"</span>
                                <input class="input" prop:value=move || editing.get().drug_class
                                    on:input=move |ev| editing.update(|drug| drug.drug_class = event_target_value(&ev)) />
                            </label>
                            <label class="field">
                                <span>"Route"</span>
                                <input class="input" prop:value=move || editing.get().route.unwrap_or_default()
                                    on:input=move |ev| editing.update(|drug| drug.route = Some(event_target_value(&ev))) />
                            </label>
                            <label class="field">
                                <span>"Severity"</span>
                                <select class="input" prop:value=move || editing.get().severity.as_str()
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        editing.update(|drug| drug.severity = HadSeverity::from_str(&value));
                                    }>
                                    <option value="critical">"Critical"</option>
                                    <option value="high">"High"</option>
                                    <option value="medium">"Medium"</option>
                                    <option value="low">"Low"</option>
                                    <option value="info">"Info"</option>
                                </select>
                            </label>
                            <label class="field">
                                <span>"Monitoring"</span>
                                <textarea class="input textarea" prop:value=move || editing.get().safety_bundle.monitoring.join("\n")
                                    on:input=move |ev| {
                                        let lines = event_target_value(&ev).lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect();
                                        editing.update(|drug| drug.safety_bundle.monitoring = lines);
                                    } />
                            </label>
                            <label class="field">
                                <span>"Antidote / reversal"</span>
                                <input class="input" prop:value=move || editing.get().safety_bundle.antidote
                                    on:input=move |ev| editing.update(|drug| drug.safety_bundle.antidote = event_target_value(&ev)) />
                            </label>
                            <label class="field">
                                <span>"Hold criteria"</span>
                                <input class="input" prop:value=move || editing.get().safety_bundle.hold_criteria
                                    on:input=move |ev| editing.update(|drug| drug.safety_bundle.hold_criteria = event_target_value(&ev)) />
                            </label>
                            <label class="field">
                                <span>"Storage / access"</span>
                                <input class="input" prop:value=move || editing.get().safety_bundle.storage
                                    on:input=move |ev| editing.update(|drug| drug.safety_bundle.storage = event_target_value(&ev)) />
                            </label>
                            <label class="field checkbox-field">
                                <input type="checkbox" prop:checked=move || editing.get().safety_bundle.double_check_required
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        editing.update(|drug| drug.safety_bundle.double_check_required = checked);
                                    } />
                                <span>"ต้องมี independent double check"</span>
                            </label>
                            <label class="field field-full">
                                <span>"Notes"</span>
                                <textarea class="input textarea" prop:value=move || editing.get().safety_bundle.notes
                                    on:input=move |ev| editing.update(|drug| drug.safety_bundle.notes = event_target_value(&ev)) />
                            </label>
                            <div class="modal-actions field-full">
                                <button type="button" class="btn btn-secondary" on:click=move |_| show_editor.set(false)>"ยกเลิก"</button>
                                <button type="submit" class="btn btn-primary">"บันทึก"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>

            <Show when=move || show_mapping.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| show_mapping.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-head">
                            <div>
                                <div class="eyebrow">"MAP TO DRUGITEMS"</div>
                                <h3>{move || show_mapping.get().map(|drug| drug.name).unwrap_or_default()}</h3>
                            </div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_mapping.set(None)>"ปิด"</button>
                        </div>
                        <form class="search-bar" on:submit=move |ev| {
                            ev.prevent_default();
                            mapping_loading.set(true);
                            let query = drug_query.get();
                            spawn_local(async move {
                                match api::search_drugitems(query).await {
                                    Ok(items) => drug_results.set(items),
                                    Err(e) => notify_error(&ctx, e),
                                }
                                mapping_loading.set(false);
                            });
                        }>
                            <input class="input search-input" placeholder="ค้นชื่อยาหรือ icode"
                                prop:value=move || drug_query.get()
                                on:input=move |ev| drug_query.set(event_target_value(&ev)) />
                            <button class="btn btn-primary" type="submit">"ค้นหา"</button>
                        </form>
                        <Show when=move || mapping_loading.get()><Spinner label="กำลังค้น drugitems" /></Show>
                        <div class="list mapping-list">
                            {move || drug_results.get().into_iter().map(|item| {
                                let icode = item.icode.clone();
                                view! {
                                    <button class="list-row" on:click={
                                        let icode = icode.clone();
                                        move |_| {
                                            let Some(drug) = show_mapping.get() else { return };
                                            let id = drug.id.clone();
                                            let icode = icode.clone();
                                            spawn_local(async move {
                                                match api::map_had_drug(id, icode).await {
                                                    Ok(()) => { show_mapping.set(None); notify(&ctx, "map icode แล้ว"); reload(); }
                                                    Err(e) => notify_error(&ctx, e),
                                                }
                                            });
                                        }
                                    }>
                                        <span class="mono tag-code">{item.icode.clone()}</span>
                                        <div class="list-main">
                                            <div class="list-title">{item.name.clone()}</div>
                                            <div class="list-sub">{format!("{} {}", item.strength.unwrap_or_default(), item.units.unwrap_or_default())}</div>
                                        </div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn blank_drug() -> HadDrug {
    HadDrug {
        id: String::new(),
        icode: None,
        name: String::new(),
        generic_name: None,
        drug_class: String::new(),
        route: None,
        severity: HadSeverity::High,
        safety_bundle: SafetyBundle::default(),
        active: true,
        version: 1,
        approved_by: None,
        approved_at: None,
        effective_at: None,
        review_due_at: None,
    }
}
