use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, KpiDefinition, StandardRef};
use crate::components::{Card, EmptyState, PageHeader, Spinner};
use crate::state::use_app;

#[component]
pub fn ReportsPage() -> impl IntoView {
    let ctx = use_app();
    let kpis = RwSignal::new(Vec::<KpiDefinition>::new());
    let standards = RwSignal::new(Vec::<StandardRef>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let kpi_result = api::list_kpi_definitions().await;
            let standard_result = api::list_standard_refs().await;
            match (kpi_result, standard_result) {
                (Ok(k), Ok(s)) => {
                    kpis.set(k);
                    standards.set(s);
                }
                (Err(e), _) | (_, Err(e)) => error.set(Some(e.message)),
            }
            loading.set(false);
        });
    });

    view! {
        <div class="page print-page">
            <PageHeader
                title="รายงานและตัวชี้วัด"
                subtitle="กรอบคุณภาพ Structure, Process, Outcome และ Balancing ภายใต้ HA ฉบับที่ 6 และ 3P Safety"
            >
                <button class="btn btn-secondary" on:click=move |_| {
                    if let Some(window) = web_sys::window() {
                        let _ = window.print();
                    }
                }>"พิมพ์รายงาน"</button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || loading.get()><Card><Spinner label="กำลังโหลดรายงาน" /></Card></Show>

            <div class="grid-3">
                {move || kpis.get().into_iter().map(|kpi| view! {
                    <Card class=format!("kpi-card domain-{}", kpi.domain)>
                        <div class="eyebrow">{kpi.domain.to_uppercase()}</div>
                        <h4>{kpi.name.clone()}</h4>
                        <div class="kpi-code mono">{kpi.code.clone()}</div>
                        <div class="kpi-formula">
                            <div><span>"Numerator"</span>{kpi.numerator.clone()}</div>
                            <div><span>"Denominator"</span>{kpi.denominator.clone()}</div>
                        </div>
                        <div class="muted">{format!("{} · {}", kpi.data_source, kpi.frequency)}</div>
                    </Card>
                }).collect_view()}
            </div>

            <Card class="table-card">
                <div class="table-head">
                    <div><div class="eyebrow">"STANDARD REFERENCE"</div><h3>"มาตรฐานและเครื่องมือที่เชื่อมโยง"</h3></div>
                </div>
                {move || {
                    let items = standards.get();
                    if items.is_empty() {
                        view! { <EmptyState title="ยังไม่มี standard reference" hint="seed ข้อมูลมาตรฐานใน app store" /> }.into_any()
                    } else {
                        view! {
                            <div class="table-wrap">
                                <table class="table">
                                    <thead><tr><th>"Framework"</th><th>"Code"</th><th>"หัวข้อ"</th><th>"ข้อกำหนด"</th><th>"Edition"</th></tr></thead>
                                    <tbody>
                                        {items.into_iter().map(|item| view! {
                                            <tr>
                                                <td class="strong">{item.framework}</td>
                                                <td class="mono">{item.code}</td>
                                                <td>{item.title}</td>
                                                <td class="muted">{item.requirement}</td>
                                                <td>{item.edition.unwrap_or_else(|| "-".into())}</td>
                                            </tr>
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }
                }}
            </Card>

            <Card>
                <div class="panel-head"><div><div class="eyebrow">"QUALITY NARRATIVE"</div><h3>"โครงรายงาน tracer"</h3></div></div>
                <div class="narrative-grid">
                    <div><span>"1. Context"</span><p>"บริบทของหอผู้ป่วย ประชากร และความเสี่ยงของ HAD"</p></div>
                    <div><span>"2. Concern"</span><p>"เหตุการณ์หรือ trigger ที่ทำให้เลือกเคสนี้"</p></div>
                    <div><span>"3. CTQ"</span><p>"Critical to quality ที่ต้องการรักษาไว้"</p></div>
                    <div><span>"4. What we saw"</span><p>"สิ่งที่เกิดขึ้นจริงในทั้ง 7 ระยะ พร้อมหลักฐาน"</p></div>
                    <div><span>"5. System gaps"</span><p>"ช่องว่างเชิงระบบและปัจจัยมนุษย์"</p></div>
                    <div><span>"6. Actions"</span><p>"CAPA ที่มี owner และ due date"</p></div>
                    <div><span>"7. Results"</span><p>"ผลลัพธ์และ KPI หลังดำเนินการ"</p></div>
                    <div><span>"8. Re-trace"</span><p>"ยืนยันประสิทธิผลด้วย tracer รอบถัดไป"</p></div>
                </div>
            </Card>
        </div>
    }
}
