use had_tracer_core::{HadOrderView, PatientCockpit, PatientSummary};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, CreateTracerInput};
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, severity_tone};
use crate::state::{Page, notify, notify_error, use_app};

#[component]
pub fn PatientsPage() -> impl IntoView {
    let ctx = use_app();
    let query = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<PatientSummary>::new());
    let loading = RwSignal::new(false);
    let searched = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let search = move || {
        let value = query.get();
        if value.trim().is_empty() {
            notify(&ctx, "กรอก HN หรือชื่อผู้ป่วยก่อนค้นหา");
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match api::search_patients(value, Some(30)).await {
                Ok(items) => {
                    results.set(items);
                    searched.set(true);
                }
                Err(e) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    };

    view! {
        <div class="page">
            <PageHeader
                title="ค้นหาผู้ป่วย"
                subtitle="ค้นด้วย HN หรือชื่อ แล้วเปิด HAD cockpit เพื่อดูคำสั่งยาและผลแล็บล่าสุด"
            >
                <button class="btn btn-secondary" on:click=move |_| ctx.page.set(Page::Dashboard)>
                    "กลับภาพรวม"
                </button>
            </PageHeader>

            <Card>
                <form class="search-bar" on:submit=move |ev| {
                    ev.prevent_default();
                    search();
                }>
                    <input
                        class="input search-input"
                        placeholder="HN, ชื่อ, หรือนามสกุล"
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                    />
                    <button class="btn btn-primary" type="submit" disabled=move || loading.get()>
                        {move || if loading.get() { "กำลังค้นหา" } else { "ค้นหา" }}
                    </button>
                </form>
                <Show when=move || error.get().is_some()>
                    <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
                </Show>
            </Card>

            <Show when=move || loading.get()>
                <Card><Spinner label="กำลังค้นหาผู้ป่วยใน HOSxP" /></Card>
            </Show>

            <Show when=move || !loading.get() && searched.get()>
                <Card class="table-card">
                    <div class="table-head">
                        <div>
                            <div class="eyebrow">"HOSxP PATIENT INDEX"</div>
                            <h3>{move || format!("ผลลัพธ์ {} ราย", results.get().len())}</h3>
                        </div>
                    </div>
                    {move || {
                        let items = results.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ไม่พบผู้ป่วย" hint="ลองค้นด้วย HN แบบเต็ม หรือตรวจสอบการเชื่อมต่อ HOSxP" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead>
                                            <tr>
                                                <th>"HN"</th>
                                                <th>"ชื่อ"</th>
                                                <th>"วันเกิด"</th>
                                                <th>"CID"</th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.into_iter().map(|patient| {
                                                let hn = patient.hn.clone();
                                                view! {
                                                    <tr>
                                                        <td class="mono strong">{patient.hn.clone()}</td>
                                                        <td>{patient.name.clone()}</td>
                                                        <td>{patient.birthday.clone().unwrap_or_else(|| "-".into())}</td>
                                                        <td class="mono">{patient.cid_masked.clone().unwrap_or_else(|| "-".into())}</td>
                                                        <td class="actions">
                                                            <button class="btn btn-primary btn-sm" on:click=move |_| ctx.open_patient(hn.clone())>
                                                                "เปิด Cockpit"
                                                            </button>
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

            <Show when=move || !searched.get() && !loading.get()>
                <Card>
                    <EmptyState title="พร้อมค้นหา" hint="พิมพ์ HN หรือชื่อผู้ป่วยด้านบน ระบบจะอ่านข้อมูลแบบ read-only จาก HOSxP" />
                </Card>
            </Show>
        </div>
    }
}

#[component]
pub fn PatientCockpitPage() -> impl IntoView {
    let ctx = use_app();
    let cockpit = RwSignal::new(None::<PatientCockpit>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let tab = RwSignal::new("orders".to_string());
    let show_tracer = RwSignal::new(false);
    let tracer_title = RwSignal::new(String::new());
    let tracer_purpose = RwSignal::new(String::new());
    let tracer_lead = RwSignal::new(String::new());

    Effect::new(move |_| {
        ctx.refresh.get();
        let hn = ctx.selected_hn.get();
        if hn.trim().is_empty() {
            loading.set(false);
            error.set(Some("ยังไม่ได้เลือกผู้ป่วย".to_string()));
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match api::get_patient_cockpit(hn).await {
                Ok(value) => cockpit.set(Some(value)),
                Err(e) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    let evaluate = move || {
        let hn = ctx.selected_hn.get();
        spawn_local(async move {
            match api::evaluate_alerts(hn).await {
                Ok(created) => notify(
                    &ctx,
                    format!("ประเมิน alert แล้ว สร้างใหม่ {} รายการ", created.len()),
                ),
                Err(e) => notify_error(&ctx, e),
            }
        });
    };

    view! {
        <div class="page">
            <PageHeader
                title="Patient HAD Cockpit"
                subtitle="คำสั่งยาเสี่ยงสูง ผลแล็บ สัญญาณชีพ และประวัติแพ้ยาในหน้าเดียว"
            >
                <button class="btn btn-secondary" on:click=move |_| ctx.page.set(Page::Patients)>
                    "กลับหน้าค้นหา"
                </button>
                <button class="btn btn-secondary" on:click=move |_| evaluate()>
                    "ประเมิน Alerts"
                </button>
                <button class="btn btn-primary" on:click=move |_| show_tracer.set(true)>
                    "สร้าง Tracer"
                </button>
            </PageHeader>

            <Show when=move || loading.get()>
                <Card><Spinner label="กำลังโหลดข้อมูลผู้ป่วย" /></Card>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            {move || cockpit.get().map(|data| {
                let patient = data.patient.clone();
                let warnings = data.warnings.clone();
                let panel_data = data.clone();
                view! {
                    <Card class="patient-hero">
                        <div class="patient-identity">
                            <div class="patient-avatar">"PT"</div>
                            <div>
                                <div class="eyebrow">"PATIENT"</div>
                                <h2>{patient.name.clone()}</h2>
                                <div class="patient-meta">
                                    <span class="mono">"HN " {patient.hn.clone()}</span>
                                    <span>{patient.birthday.clone().unwrap_or_else(|| "-".into())}</span>
                                    <span>{patient.cid_masked.clone().unwrap_or_else(|| "-".into())}</span>
                                </div>
                            </div>
                        </div>
                        <div class="patient-context">
                            <div><span>"AN"</span><strong>{data.active_an.clone().unwrap_or_else(|| "-".into())}</strong></div>
                            <div><span>"Ward"</span><strong>{data.active_ward.clone().unwrap_or_else(|| "-".into())}</strong></div>
                            <div><span>"Admit"</span><strong>{data.active_admission_date.clone().unwrap_or_else(|| "-".into())}</strong></div>
                            <div><span>"Allergy"</span><strong>{data.allergies.len()}</strong></div>
                        </div>
                    </Card>

                    {warnings.into_iter().map(|warning| view! {
                        <div class="banner banner-warn">{warning}</div>
                    }).collect_view()}

                    <div class="tabs">
                        {tab_button("orders", "HAD Orders", tab)}
                        {tab_button("labs", "Labs", tab)}
                        {tab_button("vitals", "Vitals", tab)}
                        {tab_button("allergy", "Allergy", tab)}
                        {tab_button("diagnoses", "Diagnoses", tab)}
                    </div>

                    {move || match tab.get().as_str() {
                        "labs" => view! { <LabsPanel data=panel_data.clone() /> }.into_any(),
                        "vitals" => view! { <VitalsPanel data=panel_data.clone() /> }.into_any(),
                        "allergy" => view! { <AllergyPanel data=panel_data.clone() /> }.into_any(),
                        "diagnoses" => view! { <DiagnosesPanel data=panel_data.clone() /> }.into_any(),
                        _ => view! { <OrdersPanel data=panel_data.clone() /> }.into_any(),
                    }}
                }
            })}

            <Show when=move || show_tracer.get()>
                <div class="modal-backdrop" on:click=move |_| show_tracer.set(false)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-head">
                            <div>
                                <div class="eyebrow">"NEW TRACER"</div>
                                <h3>"สร้าง clinical tracer"</h3>
                            </div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_tracer.set(false)>"ปิด"</button>
                        </div>
                        <form class="form" on:submit=move |ev| {
                            ev.prevent_default();
                            let hn = ctx.selected_hn.get();
                            let title = if tracer_title.get().trim().is_empty() {
                                format!("HAD tracer {}", hn)
                            } else {
                                tracer_title.get()
                            };
                            let input = CreateTracerInput {
                                case_type: "individual".to_string(),
                                title,
                                purpose: tracer_purpose.get(),
                                hn,
                                an: cockpit.get().and_then(|data| data.active_an),
                                vn: None,
                                lead_user: Some(tracer_lead.get()).filter(|value| !value.trim().is_empty()),
                                trigger_type: Some("manual".to_string()),
                                trigger_ref: None,
                            };
                            spawn_local(async move {
                                match api::create_tracer_case(input).await {
                                    Ok(workspace) => {
                                        show_tracer.set(false);
                                        notify(&ctx, "สร้าง tracer แล้ว");
                                        ctx.open_tracer(workspace.case.id);
                                    }
                                    Err(e) => notify_error(&ctx, e),
                                }
                            });
                        }>
                            <label class="field">
                                <span>"ชื่อเคส"</span>
                                <input class="input" prop:value=move || tracer_title.get()
                                    on:input=move |ev| tracer_title.set(event_target_value(&ev))
                                    placeholder="เช่น HAD-Warfarin-001" />
                            </label>
                            <label class="field">
                                <span>"วัตถุประสงค์"</span>
                                <textarea class="input textarea" prop:value=move || tracer_purpose.get()
                                    on:input=move |ev| tracer_purpose.set(event_target_value(&ev))
                                    placeholder="ทบทวนเคส INR สูงและหา system gap" />
                            </label>
                            <label class="field">
                                <span>"ผู้รับผิดชอบ"</span>
                                <input class="input" prop:value=move || tracer_lead.get()
                                    on:input=move |ev| tracer_lead.set(event_target_value(&ev))
                                    placeholder="ชื่อเภสัชกรหรือพยาบาล" />
                            </label>
                            <div class="modal-actions">
                                <button type="button" class="btn btn-secondary" on:click=move |_| show_tracer.set(false)>"ยกเลิก"</button>
                                <button type="submit" class="btn btn-primary">"สร้าง Tracer"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn tab_button(id: &'static str, label: &'static str, tab: RwSignal<String>) -> impl IntoView {
    view! {
        <button
            class=move || if tab.get() == id { "tab active" } else { "tab" }
            on:click=move |_| tab.set(id.to_string())
        >{label}</button>
    }
}

#[component]
fn OrdersPanel(data: PatientCockpit) -> impl IntoView {
    view! {
        <Card class="table-card">
            <div class="table-head">
                <div>
                    <div class="eyebrow">"HIGH-ALERT ORDERS"</div>
                    <h3>"รายการยาล่าสุด"</h3>
                </div>
                <span class="muted">{format!("{} รายการ", data.had_orders.len())}</span>
            </div>
            {if data.had_orders.is_empty() {
                view! { <EmptyState title="ไม่พบ HAD order" hint="ตรวจสอบว่า HAD catalog map กับ icode แล้วหรือยัง" /> }.into_any()
            } else {
                view! {
                    <div class="table-wrap">
                        <table class="table">
                            <thead>
                                <tr>
                                    <th>"Severity"</th><th>"ยา"</th><th>"กลุ่ม"</th>
                                    <th>"วันที่"</th><th>"Qty"</th><th>"Sig"</th><th>"Source"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {data.had_orders.into_iter().map(|order: HadOrderView| view! {
                                    <tr>
                                        <td><Badge tone=severity_tone(order.severity)>{order.severity.label_th()}</Badge></td>
                                        <td>
                                            <div class="strong">{order.drug_name.clone()}</div>
                                            <div class="muted">{format!("{} {}", order.strength.clone().unwrap_or_default(), order.units.clone().unwrap_or_default())}</div>
                                        </td>
                                        <td>{order.had_drug_class.clone().unwrap_or_else(|| "-".into())}</td>
                                        <td class="mono">{order.vstdate.clone().unwrap_or_else(|| "-".into())}</td>
                                        <td class="mono">{order.qty.map(|value| value.to_string()).unwrap_or_else(|| "-".into())}</td>
                                        <td>{order.sig.clone().or(order.special_use.clone()).unwrap_or_else(|| "-".into())}</td>
                                        <td class="mono">{order.an.clone().map(|an| format!("AN {an}")).or_else(|| order.vn.clone().map(|vn| format!("VN {vn}"))).unwrap_or_else(|| "-".into())}</td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
        </Card>
    }
}

#[component]
fn LabsPanel(data: PatientCockpit) -> impl IntoView {
    view! {
        <Card class="table-card">
            <div class="table-head">
                <div>
                    <div class="eyebrow">"LABORATORY"</div>
                    <h3>"ผลแล็บที่เกี่ยวข้อง"</h3>
                </div>
            </div>
            {if data.labs.is_empty() {
                view! { <EmptyState title="ยังไม่มีผลแล็บ" hint="ต้องมี alert rule ที่ active และ lab items code ตรงกับ HOSxP" /> }.into_any()
            } else {
                view! {
                    <div class="table-wrap">
                        <table class="table">
                            <thead><tr><th>"Lab"</th><th>"ผล"</th><th>"หน่วย"</th><th>"วันที่"</th><th>"Order"</th></tr></thead>
                            <tbody>
                                {data.labs.into_iter().map(|lab| view! {
                                    <tr>
                                        <td class="strong">{lab.lab_items_name}</td>
                                        <td class="mono strong">{lab.raw_value}</td>
                                        <td>{lab.unit.unwrap_or_else(|| "-".into())}</td>
                                        <td class="mono">{lab.observed_at.unwrap_or_else(|| "-".into())}</td>
                                        <td class="mono muted">{lab.lab_order_number}</td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
        </Card>
    }
}

#[component]
fn VitalsPanel(data: PatientCockpit) -> impl IntoView {
    view! {
        <Card>
            <div class="table-head">
                <div>
                    <div class="eyebrow">"VITAL SIGNS"</div>
                    <h3>"OPD และ IPD"</h3>
                </div>
            </div>
            {if data.vitals.is_empty() {
                view! { <EmptyState title="ยังไม่มีสัญญาณชีพ" hint="OPD อ่านจาก opdscreen และ IPD อ่านจาก ipt_vital_sign" /> }.into_any()
            } else {
                view! {
                    <div class="vital-grid">
                        {data.vitals.into_iter().map(|vital| view! {
                            <div class="vital-card">
                                <div class="vital-source">{vital.source}</div>
                                <div class="vital-name">{vital.name}</div>
                                <div class="vital-value">{vital.value}<span>{vital.unit.unwrap_or_default()}</span></div>
                                <div class="vital-time">{vital.observed_at.unwrap_or_else(|| "-".into())}</div>
                            </div>
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </Card>
    }
}

#[component]
fn AllergyPanel(data: PatientCockpit) -> impl IntoView {
    view! {
        <Card>
            <div class="table-head">
                <div>
                    <div class="eyebrow">"ALLERGY"</div>
                    <h3>"ประวัติแพ้ยา"</h3>
                </div>
            </div>
            {if data.allergies.is_empty() {
                view! { <EmptyState title="ไม่พบประวัติแพ้ยา" hint="opd_allergy ไม่มีรหัสยา ต้องตรวจทานชื่อด้วยคน" /> }.into_any()
            } else {
                view! {
                    <div class="list">
                        {data.allergies.into_iter().map(|allergy| view! {
                            <div class="list-row static">
                                <span class="allergy-dot"></span>
                                <div class="list-main">
                                    <div class="list-title">{allergy.agent}</div>
                                    <div class="list-sub">{allergy.symptom.unwrap_or_else(|| "-".into())}</div>
                                </div>
                                <div class="list-meta">{allergy.report_date.unwrap_or_else(|| "-".into())}</div>
                            </div>
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </Card>
    }
}

#[component]
fn DiagnosesPanel(data: PatientCockpit) -> impl IntoView {
    view! {
        <Card class="table-card">
            <div class="table-head">
                <div>
                    <div class="eyebrow">"DIAGNOSES"</div>
                    <h3>"การวินิจฉัย"</h3>
                </div>
            </div>
            {if data.diagnoses.is_empty() {
                view! { <EmptyState title="ไม่พบการวินิจฉัย" hint="ตรวจสอบ ovstdiag และ iptdiag" /> }.into_any()
            } else {
                view! {
                    <div class="table-wrap">
                        <table class="table">
                            <thead><tr><th>"ICD-10"</th><th>"ชื่อ"</th><th>"ประเภท"</th><th>"Source"</th></tr></thead>
                            <tbody>
                                {data.diagnoses.into_iter().map(|diagnosis| view! {
                                    <tr>
                                        <td class="mono strong">{diagnosis.icd10}</td>
                                        <td>{diagnosis.name.unwrap_or_else(|| "-".into())}</td>
                                        <td>{diagnosis.diag_type.unwrap_or_else(|| "-".into())}</td>
                                        <td class="mono">{diagnosis.source}</td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
        </Card>
    }
}
