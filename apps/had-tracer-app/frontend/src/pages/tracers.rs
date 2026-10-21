use had_tracer_core::{CauseCategory, FindingStatus, TracerCase, TracerCheck, TracerWorkspace};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, CapaInput, CreateTracerInput, FindingInput};
use crate::components::{Badge, Card, EmptyState, PageHeader, Spinner, finding_severity_tone};
use crate::state::{Page, notify, notify_error, use_app};

#[component]
pub fn TracersPage() -> impl IntoView {
    let ctx = use_app();
    let cases = RwSignal::new(Vec::<TracerCase>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let show_new = RwSignal::new(false);
    let case_type = RwSignal::new("individual".to_string());
    let title = RwSignal::new(String::new());
    let purpose = RwSignal::new(String::new());
    let hn = RwSignal::new(String::new());
    let lead = RwSignal::new(String::new());

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match api::list_tracer_cases(Some(200)).await {
                Ok(items) => cases.set(items),
                Err(e) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    view! {
        <div class="page">
            <PageHeader
                title="Clinical Tracer"
                subtitle="ตามรอยผู้ป่วยหรือระบบงาน 7 ระยะ พร้อม finding และ CAPA ที่ตรวจสอบผลได้"
            >
                <button class="btn btn-secondary" on:click=move |_| ctx.refresh.update(|value| *value += 1)>
                    "รีเฟรช"
                </button>
                <button class="btn btn-primary" on:click=move |_| show_new.set(true)>
                    "สร้าง Tracer"
                </button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <Show when=move || loading.get()>
                <Card><Spinner label="กำลังโหลด tracer" /></Card>
            </Show>

            <Show when=move || !loading.get()>
                <Card class="table-card">
                    {move || {
                        let items = cases.get();
                        if items.is_empty() {
                            view! { <EmptyState title="ยังไม่มี tracer" hint="เริ่มจากผู้ป่วยหนึ่งราย หรือสร้าง system tracer" /> }.into_any()
                        } else {
                            view! {
                                <div class="table-wrap">
                                    <table class="table">
                                        <thead><tr><th>"เคส"</th><th>"ประเภท"</th><th>"HN"</th><th>"เปิดเมื่อ"</th><th>"Status"</th><th></th></tr></thead>
                                        <tbody>
                                            {items.into_iter().map(|case| {
                                                let id = case.id.clone();
                                                let status = case.status;
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <div class="strong">{case.title.clone()}</div>
                                                            <div class="muted">{case.purpose.clone()}</div>
                                                        </td>
                                                        <td>{case.case_type.label_th()}</td>
                                                        <td class="mono">{case.hn.clone()}</td>
                                                        <td class="mono">{short_time(&case.opened_at)}</td>
                                                        <td><span class=format!("status-pill status-{}", status.as_str())>{status.label_th()}</span></td>
                                                        <td class="actions">
                                                            <button class="btn btn-primary btn-sm" on:click=move |_| ctx.open_tracer(id.clone())>
                                                                "เปิด"
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

            <Show when=move || show_new.get()>
                <div class="modal-backdrop" on:click=move |_| show_new.set(false)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-head">
                            <div><div class="eyebrow">"NEW TRACER"</div><h3>"สร้างเคสตามรอย"</h3></div>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_new.set(false)>"ปิด"</button>
                        </div>
                        <form class="form" on:submit=move |ev| {
                            ev.prevent_default();
                            let input = CreateTracerInput {
                                case_type: case_type.get(),
                                title: title.get(),
                                purpose: purpose.get(),
                                hn: hn.get(),
                                an: None,
                                vn: None,
                                lead_user: Some(lead.get()).filter(|value| !value.trim().is_empty()),
                                trigger_type: Some("manual".to_string()),
                                trigger_ref: None,
                            };
                            spawn_local(async move {
                                match api::create_tracer_case(input).await {
                                    Ok(workspace) => {
                                        show_new.set(false);
                                        notify(&ctx, "สร้าง tracer แล้ว");
                                        ctx.open_tracer(workspace.case.id);
                                    }
                                    Err(e) => notify_error(&ctx, e),
                                }
                            });
                        }>
                            <label class="field">
                                <span>"ประเภท"</span>
                                <select class="input" prop:value=move || case_type.get()
                                    on:change=move |ev| case_type.set(event_target_value(&ev))>
                                    <option value="individual">"Individual tracer"</option>
                                    <option value="system">"System tracer"</option>
                                    <option value="program_specific">"Program-specific tracer"</option>
                                </select>
                            </label>
                            <label class="field"><span>"ชื่อเคส"</span>
                                <input class="input" prop:value=move || title.get()
                                    on:input=move |ev| title.set(event_target_value(&ev)) />
                            </label>
                            <label class="field"><span>"HN"</span>
                                <input class="input mono" prop:value=move || hn.get()
                                    on:input=move |ev| hn.set(event_target_value(&ev)) />
                            </label>
                            <label class="field"><span>"วัตถุประสงค์"</span>
                                <textarea class="input textarea" prop:value=move || purpose.get()
                                    on:input=move |ev| purpose.set(event_target_value(&ev)) />
                            </label>
                            <label class="field"><span>"ผู้รับผิดชอบ"</span>
                                <input class="input" prop:value=move || lead.get()
                                    on:input=move |ev| lead.set(event_target_value(&ev)) />
                            </label>
                            <div class="modal-actions">
                                <button type="button" class="btn btn-secondary" on:click=move |_| show_new.set(false)>"ยกเลิก"</button>
                                <button type="submit" class="btn btn-primary">"สร้าง Tracer"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn TracerDetailPage() -> impl IntoView {
    let ctx = use_app();
    let workspace = RwSignal::new(None::<TracerWorkspace>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let selected_phase = RwSignal::new(String::new());
    let finding_title = RwSignal::new(String::new());
    let finding_description = RwSignal::new(String::new());
    let finding_severity = RwSignal::new("medium".to_string());
    let finding_likelihood = RwSignal::new("3".to_string());
    let finding_cause = RwSignal::new("task".to_string());
    let finding_standard = RwSignal::new(String::new());
    let capa_finding = RwSignal::new(String::new());
    let capa_action = RwSignal::new(String::new());
    let capa_owner = RwSignal::new(String::new());
    let capa_due = RwSignal::new(String::new());
    let capa_method = RwSignal::new(String::new());
    let capa_effectiveness = RwSignal::new(String::new());

    Effect::new(move |_| {
        ctx.refresh.get();
        let id = ctx.selected_case.get();
        if id.trim().is_empty() {
            loading.set(false);
            error.set(Some("ยังไม่ได้เลือก tracer".to_string()));
            return;
        }
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match api::get_tracer_workspace(id).await {
                Ok(value) => {
                    if let Some(first) = value.phases.first() {
                        selected_phase.set(first.id.clone());
                    }
                    workspace.set(Some(value));
                }
                Err(e) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    let reload = move || ctx.refresh.update(|value| *value += 1);

    view! {
        <div class="page">
            <PageHeader
                title="Tracer Workspace"
                subtitle="บันทึกสิ่งที่ควรเกิด สิ่งที่เกิดจริง ช่องว่าง และหลักฐานในแต่ละระยะ"
            >
                <button class="btn btn-secondary" on:click=move |_| ctx.page.set(Page::Tracers)>"กลับรายการ"</button>
                <button class="btn btn-secondary" on:click=move |_| reload()>"รีเฟรช"</button>
                <button class="btn btn-primary" on:click=move |_| {
                    let Some(ws) = workspace.get() else { return };
                    let id = ws.case.id.clone();
                    spawn_local(async move {
                        match api::close_tracer_case(id).await {
                            Ok(()) => { notify(&ctx, "ปิด tracer แล้ว"); reload(); }
                            Err(e) => notify_error(&ctx, e),
                        }
                    });
                }>"ปิดเคส"</button>
            </PageHeader>

            <Show when=move || loading.get()><Card><Spinner label="กำลังโหลด tracer" /></Card></Show>
            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            {move || workspace.get().map(|ws| {
                let case = ws.case.clone();
                let phases = ws.phases.clone();
                let checks = ws.checks.clone();
                let findings = ws.findings.clone();
                let findings_for_capa = findings.clone();
                let findings_for_register = findings.clone();
                let findings_for_effectiveness = findings.clone();
                let capas = ws.capas.clone();
                view! {
                    <Card class="tracer-hero">
                        <div>
                            <div class="eyebrow">"TRACER CASE"</div>
                            <h2>{case.title.clone()}</h2>
                            <p>{case.purpose.clone()}</p>
                            <div class="patient-meta">
                                <span class="mono">"HN " {case.hn.clone()}</span>
                                <span>{case.case_type.label_th()}</span>
                                <span>{case.lead_user.clone().unwrap_or_else(|| "-".into())}</span>
                            </div>
                        </div>
                        <span class=format!("status-pill status-{}", case.status.as_str())>{case.status.label_th()}</span>
                    </Card>

                    <div class="phase-tabs">
                        {phases.into_iter().map(|phase| {
                            let id = phase.id.clone();
                            let class_id = id.clone();
                            let done = checks.iter().filter(|check| check.phase_id == id && !check.observed.trim().is_empty()).count();
                            let total = checks.iter().filter(|check| check.phase_id == id).count();
                            let label = phase.phase_code.label_th();
                            view! {
                                <button class=move || if selected_phase.get() == class_id { "phase-tab active" } else { "phase-tab" }
                                    on:click={
                                        let id = id.clone();
                                        move |_| selected_phase.set(id.clone())
                                    }>
                                    <span class="phase-name">{label}</span>
                                    <span class="phase-count">{format!("{done}/{total}")}</span>
                                </button>
                            }
                        }).collect_view()}
                    </div>

                    {move || {
                        let Some(ws) = workspace.get() else { return view! { <div></div> }.into_any() };
                        let Some(phase) = ws.phases.iter().find(|phase| phase.id == selected_phase.get()).cloned() else {
                            return view! { <EmptyState title="เลือก phase" hint="เลือกขั้นตอนด้านบน" /> }.into_any();
                        };
                        let phase_checks: Vec<TracerCheck> = ws.checks.iter().filter(|check| check.phase_id == phase.id).cloned().collect();
                        view! {
                            <Card>
                                <div class="panel-head">
                                    <div>
                                        <div class="eyebrow">{phase.phase_code.as_str().to_uppercase()}</div>
                                        <h3>{phase.phase_code.label_th()}</h3>
                                        <p>{phase.phase_code.focus_th()}</p>
                                    </div>
                                    <span class="muted">{format!("{} checks", phase_checks.len())}</span>
                                </div>
                                <label class="field">
                                    <span>"สรุประยะนี้"</span>
                                    <textarea class="input textarea" prop:value=phase.summary.clone()
                                        on:input={
                                            let phase_id = phase.id.clone();
                                            move |ev| {
                                                let value = event_target_value(&ev);
                                                workspace.update(|ws| {
                                                    if let Some(ws) = ws
                                                        && let Some(target) = ws.phases.iter_mut().find(|item| item.id == phase_id)
                                                    {
                                                        target.summary = value.clone();
                                                    }
                                                });
                                            }
                                        } />
                                </label>
                                <div class="inline-actions">
                                    <button class="btn btn-secondary btn-sm" on:click={
                                        let phase_id = phase.id.clone();
                                        move |_| {
                                            let Some(ws) = workspace.get() else { return };
                                            let Some(target) = ws.phases.iter().find(|item| item.id == phase_id) else { return };
                                            let phase_id = target.id.clone();
                                            let summary = target.summary.clone();
                                            spawn_local(async move {
                                                match api::update_tracer_phase(phase_id, "in_progress".to_string(), summary).await {
                                                    Ok(()) => notify(&ctx, "บันทึกสรุประยะแล้ว"),
                                                    Err(e) => notify_error(&ctx, e),
                                                }
                                            });
                                        }
                                    }>"บันทึกสรุป"</button>
                                </div>

                                <div class="check-list">
                                    {phase_checks.into_iter().map(|check| {
                                        let check_id = check.id.clone();
                                        let question = check.question.clone();
                                        let expected = check.expected.clone();
                                        let observed = check.observed.clone();
                                        let gap = check.gap.clone();
                                        let evidence = check.evidence_summary.clone();
                                        let standard = check.standard_code.clone().unwrap_or_default();
                                        view! {
                                            <div class="check-card">
                                                <div class="check-question">{question}</div>
                                                <div class="form-grid">
                                                    <label class="field"><span>"Standard"</span>
                                                        <input class="input" prop:value=standard.clone()
                                                            on:input={
                                                                let id = check_id.clone();
                                                                move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    workspace.update(|ws| {
                                                                        if let Some(ws) = ws
                                                                            && let Some(target) = ws.checks.iter_mut().find(|item| item.id == id)
                                                                        {
                                                                            target.standard_code = Some(value.clone());
                                                                        }
                                                                    });
                                                                }
                                                            } />
                                                    </label>
                                                    <label class="field"><span>"สิ่งที่ควรเกิด"</span>
                                                        <textarea class="input textarea" prop:value=expected.clone()
                                                            on:input={
                                                                let id = check_id.clone();
                                                                move |ev| update_check(&workspace, &id, |check| check.expected = event_target_value(&ev))
                                                            } />
                                                    </label>
                                                    <label class="field"><span>"สิ่งที่เกิดจริง"</span>
                                                        <textarea class="input textarea" prop:value=observed.clone()
                                                            on:input={
                                                                let id = check_id.clone();
                                                                move |ev| update_check(&workspace, &id, |check| check.observed = event_target_value(&ev))
                                                            } />
                                                    </label>
                                                    <label class="field"><span>"ช่องว่าง"</span>
                                                        <textarea class="input textarea" prop:value=gap.clone()
                                                            on:input={
                                                                let id = check_id.clone();
                                                                move |ev| update_check(&workspace, &id, |check| check.gap = event_target_value(&ev))
                                                            } />
                                                    </label>
                                                    <label class="field field-full"><span>"หลักฐาน"</span>
                                                        <textarea class="input textarea" prop:value=evidence.clone()
                                                            on:input={
                                                                let id = check_id.clone();
                                                                move |ev| update_check(&workspace, &id, |check| check.evidence_summary = event_target_value(&ev))
                                                            } />
                                                    </label>
                                                </div>
                                                <div class="inline-actions">
                                                    <button class="btn btn-primary btn-sm" on:click={
                                                        let id = check_id.clone();
                                                        move |_| {
                                                            let Some(ws) = workspace.get() else { return };
                                                            let Some(check) = ws.checks.iter().find(|item| item.id == id).cloned() else { return };
                                                            spawn_local(async move {
                                                                match api::update_tracer_check(check).await {
                                                                    Ok(()) => notify(&ctx, "บันทึก check แล้ว"),
                                                                    Err(e) => notify_error(&ctx, e),
                                                                }
                                                            });
                                                        }
                                                    }>"บันทึก Check"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </Card>
                        }.into_any()
                    }}

                    <div class="grid-2">
                        <Card>
                            <div class="panel-head"><div><div class="eyebrow">"FINDINGS"</div><h3>"สร้าง Finding"</h3></div></div>
                            <form class="form" on:submit={
                                let case_id = case.id.clone();
                                move |ev| {
                                    ev.prevent_default();
                                    let input = FindingInput {
                                        case_id: case_id.clone(),
                                        phase_id: Some(selected_phase.get()),
                                        title: finding_title.get(),
                                        description: finding_description.get(),
                                        standard_code: Some(finding_standard.get()).filter(|value| !value.trim().is_empty()),
                                        severity: finding_severity.get(),
                                        likelihood: finding_likelihood.get().parse().unwrap_or(3),
                                        cause_category: finding_cause.get(),
                                        immediate_action: String::new(),
                                        recommended_action: String::new(),
                                    };
                                    spawn_local(async move {
                                        match api::create_finding(input).await {
                                            Ok(_) => { notify(&ctx, "สร้าง finding แล้ว"); finding_title.set(String::new()); finding_description.set(String::new()); reload(); }
                                            Err(e) => notify_error(&ctx, e),
                                        }
                                    });
                                }
                            }>
                                <label class="field"><span>"หัวข้อ"</span><input class="input" prop:value=move || finding_title.get()
                                    on:input=move |ev| finding_title.set(event_target_value(&ev)) /></label>
                                <label class="field"><span>"รายละเอียด"</span><textarea class="input textarea" prop:value=move || finding_description.get()
                                    on:input=move |ev| finding_description.set(event_target_value(&ev)) /></label>
                                <div class="form-row">
                                    <label class="field"><span>"Severity"</span>
                                        <select class="input" prop:value=move || finding_severity.get()
                                            on:change=move |ev| finding_severity.set(event_target_value(&ev))>
                                            <option value="low">"Low"</option><option value="medium">"Medium"</option>
                                            <option value="high">"High"</option><option value="critical">"Critical"</option>
                                        </select>
                                    </label>
                                    <label class="field"><span>"Likelihood 1-5"</span><input class="input" prop:value=move || finding_likelihood.get()
                                        on:input=move |ev| finding_likelihood.set(event_target_value(&ev)) /></label>
                                    <label class="field"><span>"Cause"</span>
                                        <select class="input" prop:value=move || finding_cause.get()
                                            on:change=move |ev| finding_cause.set(event_target_value(&ev))>
                                            {CauseCategory::ALL.into_iter().map(|cause| view! {
                                                <option value=cause.as_str()>{cause.label_th()}</option>
                                            }).collect_view()}
                                        </select>
                                    </label>
                                </div>
                                <label class="field"><span>"Standard code"</span><input class="input" prop:value=move || finding_standard.get()
                                    on:input=move |ev| finding_standard.set(event_target_value(&ev)) /></label>
                                <button class="btn btn-primary" type="submit">"เพิ่ม Finding"</button>
                            </form>
                        </Card>

                        <Card>
                            <div class="panel-head"><div><div class="eyebrow">"CAPA"</div><h3>"สร้าง CAPA"</h3></div></div>
                            <form class="form" on:submit=move |ev| {
                                ev.prevent_default();
                                let input = CapaInput {
                                    finding_id: capa_finding.get(),
                                    action: capa_action.get(),
                                    owner: capa_owner.get(),
                                    due_date: Some(capa_due.get()).filter(|value| !value.trim().is_empty()),
                                    verification_method: capa_method.get(),
                                };
                                spawn_local(async move {
                                    match api::create_capa(input).await {
                                        Ok(_) => { notify(&ctx, "สร้าง CAPA แล้ว"); capa_action.set(String::new()); reload(); }
                                        Err(e) => notify_error(&ctx, e),
                                    }
                                });
                            }>
                                <label class="field"><span>"Finding"</span>
                                    <select class="input" prop:value=move || capa_finding.get()
                                        on:change=move |ev| capa_finding.set(event_target_value(&ev))>
                                        <option value="">"เลือก finding"</option>
                                        {findings_for_capa.iter().map(|finding| view! {
                                            <option value=finding.id.clone()>{finding.title.clone()}</option>
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="field"><span>"Action"</span><textarea class="input textarea" prop:value=move || capa_action.get()
                                    on:input=move |ev| capa_action.set(event_target_value(&ev)) /></label>
                                <div class="form-row">
                                    <label class="field"><span>"Owner"</span><input class="input" prop:value=move || capa_owner.get()
                                        on:input=move |ev| capa_owner.set(event_target_value(&ev)) /></label>
                                    <label class="field"><span>"Due date"</span><input class="input" type="date" prop:value=move || capa_due.get()
                                        on:input=move |ev| capa_due.set(event_target_value(&ev)) /></label>
                                </div>
                                <label class="field"><span>"Verification method"</span><input class="input" prop:value=move || capa_method.get()
                                    on:input=move |ev| capa_method.set(event_target_value(&ev)) /></label>
                                <button class="btn btn-primary" type="submit">"เพิ่ม CAPA"</button>
                            </form>
                        </Card>
                    </div>

                    <div class="grid-2">
                        <Card>
                            <div class="panel-head"><div><div class="eyebrow">"FINDING REGISTER"</div><h3>"รายการ finding"</h3></div></div>
                            {if findings.is_empty() {
                                view! { <EmptyState title="ยังไม่มี finding" hint="บันทึกช่องว่างจาก worksheet" /> }.into_any()
                            } else {
                                view! {
                                    <div class="list">
                                        {findings_for_register.into_iter().map(|finding| {
                                            let id = finding.id.clone();
                                            let severity = finding.severity;
                                            let status = finding.status;
                                            view! {
                                                <div class="list-row static">
                                                    <Badge tone=finding_severity_tone(severity)>{severity.label_th()}</Badge>
                                                    <div class="list-main">
                                                        <div class="list-title">{finding.title.clone()}</div>
                                                        <div class="list-sub">{finding.description.clone()}</div>
                                                        <div class="list-sub">{format!("risk {} · {}", finding.risk_score, finding.cause_category.label_th())}</div>
                                                    </div>
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
                                                        <option value="open">{FindingStatus::Open.label_th()}</option>
                                                        <option value="assigned">{FindingStatus::Assigned.label_th()}</option>
                                                        <option value="in_progress">{FindingStatus::InProgress.label_th()}</option>
                                                        <option value="implemented">{FindingStatus::Implemented.label_th()}</option>
                                                        <option value="verified">{FindingStatus::Verified.label_th()}</option>
                                                        <option value="closed">{FindingStatus::Closed.label_th()}</option>
                                                        <option value="reopened">{FindingStatus::Reopened.label_th()}</option>
                                                    </select>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </Card>

                        <Card>
                            <div class="panel-head"><div><div class="eyebrow">"CAPA REGISTER"</div><h3>"รายการ CAPA"</h3></div></div>
                            {if capas.is_empty() {
                                view! { <EmptyState title="ยังไม่มี CAPA" hint="ผูก action กับ finding เพื่อปิด loop" /> }.into_any()
                            } else {
                                view! {
                                    <div class="list">
                                        {capas.into_iter().map(|capa| view! {
                                            <div class="list-row static">
                                                <span class="case-type">{capa.status.label_th()}</span>
                                                <div class="list-main">
                                                    <div class="list-title">{capa.action.clone()}</div>
                                                    <div class="list-sub">{format!("{} · due {}", capa.owner, capa.due_date.clone().unwrap_or_else(|| "-".into()))}</div>
                                                </div>
                                                <span class="muted">{capa.verification_method.clone()}</span>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </div>

                    <Card>
                        <div class="panel-head"><div><div class="eyebrow">"EFFECTIVENESS"</div><h3>"บันทึกผล CAPA"</h3></div></div>
                        <div class="form-row">
                            <label class="field"><span>"Finding"</span>
                                <select class="input" prop:value=move || capa_finding.get()
                                    on:change=move |ev| capa_finding.set(event_target_value(&ev))>
                                    <option value="">"เลือก finding"</option>
                                    {findings_for_effectiveness.iter().map(|finding| view! {
                                        <option value=finding.id.clone()>{finding.title.clone()}</option>
                                    }).collect_view()}
                                </select>
                            </label>
                            <label class="field"><span>"ผลลัพธ์"</span><input class="input" prop:value=move || capa_effectiveness.get()
                                on:input=move |ev| capa_effectiveness.set(event_target_value(&ev)) /></label>
                        </div>
                        <p class="muted">"ใช้หน้านี้เป็นที่จดผลเบื้องต้น แล้วยืนยันด้วย re-trace ในรอบถัดไป"</p>
                    </Card>
                }
            })}
        </div>
    }
}

fn update_check(
    workspace: &RwSignal<Option<TracerWorkspace>>,
    id: &str,
    update: impl FnOnce(&mut TracerCheck),
) {
    let id = id.to_string();
    workspace.update(|ws| {
        if let Some(ws) = ws
            && let Some(check) = ws.checks.iter_mut().find(|item| item.id == id)
        {
            update(check);
        }
    });
}

fn short_time(value: &str) -> String {
    value.chars().take(16).collect()
}
