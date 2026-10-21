use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AppStatus, AuditEntry, ConnectionInput, ConnectionTest, SiteSettings};
use crate::components::{Card, EmptyState, PageHeader, Spinner};
use crate::state::{notify, notify_error, use_app};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let ctx = use_app();
    let status = RwSignal::new(None::<AppStatus>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let host = RwSignal::new(String::new());
    let port = RwSignal::new("3306".to_string());
    let database = RwSignal::new(String::new());
    let user = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let tls = RwSignal::new("preferred".to_string());
    let test_result = RwSignal::new(None::<ConnectionTest>);
    let settings = RwSignal::new(SiteSettings {
        site_name: String::new(),
        history_days: 30,
        alert_lookback_hours: 72,
        auto_backup: false,
        backup_retention: 14,
    });
    let audit = RwSignal::new(Vec::<AuditEntry>::new());
    let import_mode = RwSignal::new("merge".to_string());
    let dry_run = RwSignal::new(true);

    Effect::new(move |_| {
        ctx.refresh.get();
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            let status_result = api::get_app_status().await;
            let connection_result = api::get_connection().await;
            let settings_result = api::get_settings().await;
            let audit_result = api::list_audit(Some(30)).await;
            match status_result {
                Ok(value) => status.set(Some(value)),
                Err(e) => error.set(Some(e.message)),
            }
            if let Ok(Some(connection)) = connection_result {
                host.set(connection.host);
                port.set(connection.port.to_string());
                database.set(connection.database);
                user.set(connection.user);
            }
            if let Ok(value) = settings_result {
                settings.set(value);
            }
            if let Ok(items) = audit_result {
                audit.set(items);
            }
            loading.set(false);
        });
    });

    let reload = move || ctx.refresh.update(|value| *value += 1);

    view! {
        <div class="page">
            <PageHeader
                title="ตั้งค่าและสำรองข้อมูล"
                subtitle="เชื่อมต่อ HOSxP แบบ read-only, ตั้งค่า site, และจัดการ backup/import/export/restore"
            >
                <button class="btn btn-secondary" on:click=move |_| reload()>"รีเฟรช"</button>
            </PageHeader>

            <Show when=move || error.get().is_some()>
                <div class="banner banner-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || loading.get()><Card><Spinner label="กำลังโหลดการตั้งค่า" /></Card></Show>

            <div class="grid-2">
                <Card>
                    <div class="panel-head">
                        <div><div class="eyebrow">"HOSxP CONNECTION"</div><h3>"อ่านข้อมูลแบบ read-only"</h3></div>
                        <span class="badge badge-soft">"SELECT only"</span>
                    </div>
                    <form class="form" on:submit=move |ev| {
                        ev.prevent_default();
                        let input = ConnectionInput {
                            host: host.get(),
                            port: port.get().parse().unwrap_or(3306),
                            database: database.get(),
                            user: user.get(),
                            password: password.get(),
                            tls_mode: Some(tls.get()),
                        };
                        spawn_local(async move {
                            match api::save_connection(input).await {
                                Ok(_) => { notify(&ctx, "บันทึกการเชื่อมต่อแล้ว"); password.set(String::new()); reload(); }
                                Err(e) => notify_error(&ctx, e),
                            }
                        });
                    }>
                        <div class="form-row">
                            <label class="field"><span>"Host"</span><input class="input" prop:value=move || host.get()
                                on:input=move |ev| host.set(event_target_value(&ev)) /></label>
                            <label class="field"><span>"Port"</span><input class="input" prop:value=move || port.get()
                                on:input=move |ev| port.set(event_target_value(&ev)) /></label>
                        </div>
                        <label class="field"><span>"Database"</span><input class="input" prop:value=move || database.get()
                            on:input=move |ev| database.set(event_target_value(&ev)) /></label>
                        <div class="form-row">
                            <label class="field"><span>"User"</span><input class="input" prop:value=move || user.get()
                                on:input=move |ev| user.set(event_target_value(&ev)) /></label>
                            <label class="field"><span>"Password"</span><input class="input" type="password" prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev)) /></label>
                        </div>
                        <label class="field"><span>"TLS"</span>
                            <select class="input" prop:value=move || tls.get()
                                on:change=move |ev| tls.set(event_target_value(&ev))>
                                <option value="preferred">"Preferred"</option>
                                <option value="required">"Required"</option>
                                <option value="verify_ca">"Verify CA"</option>
                                <option value="verify_identity">"Verify identity"</option>
                                <option value="disabled">"Disabled"</option>
                            </select>
                        </label>
                        <div class="inline-actions">
                            <button type="submit" class="btn btn-primary">"บันทึก"</button>
                            <button type="button" class="btn btn-secondary" on:click=move |_| {
                                let input = ConnectionInput {
                                    host: host.get(), port: port.get().parse().unwrap_or(3306),
                                    database: database.get(), user: user.get(), password: password.get(),
                                    tls_mode: Some(tls.get()),
                                };
                                spawn_local(async move {
                                    match api::test_connection(input).await {
                                        Ok(result) => { test_result.set(Some(result)); notify(&ctx, "ทดสอบการเชื่อมต่อแล้ว"); }
                                        Err(e) => notify_error(&ctx, e),
                                    }
                                });
                            }>"ทดสอบการเชื่อมต่อ"</button>
                            <button type="button" class="btn btn-ghost" on:click=move |_| {
                                spawn_local(async move {
                                    match api::schema_probe().await {
                                        Ok(report) => notify(&ctx, if report.ok { "schema ครบถ้วน" } else { "schema ยังไม่ครบ ตรวจ warnings" }),
                                        Err(e) => notify_error(&ctx, e),
                                    }
                                });
                            }>"Schema Probe"</button>
                        </div>
                    </form>
                    {move || test_result.get().map(|result| view! {
                        <div class=if result.schema_ok { "banner banner-ok" } else { "banner banner-warn" }>
                            <strong>{format!("{} ms · {}", result.latency_ms, result.server_version)}</strong>
                            <div>{format!("tables {} · missing {}", result.schema_tables, result.missing.len())}</div>
                            {(!result.missing.is_empty()).then(|| view! {
                                <ul class="compact-list">{result.missing.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}</ul>
                            })}
                            {(!result.warnings.is_empty()).then(|| view! {
                                <div class="muted">{result.warnings.join(" · ")}</div>
                            })}
                        </div>
                    })}
                </Card>

                <Card>
                    <div class="panel-head"><div><div class="eyebrow">"SITE SETTINGS"</div><h3>"บริบทโรงพยาบาล"</h3></div></div>
                    <form class="form" on:submit=move |ev| {
                        ev.prevent_default();
                        let value = settings.get();
                        spawn_local(async move {
                            match api::save_settings(value).await {
                                Ok(_) => { notify(&ctx, "บันทึก site settings แล้ว"); reload(); }
                                Err(e) => notify_error(&ctx, e),
                            }
                        });
                    }>
                        <label class="field"><span>"ชื่อสถานพยาบาล"</span>
                            <input class="input" prop:value=move || settings.get().site_name
                                on:input=move |ev| settings.update(|value| value.site_name = event_target_value(&ev)) />
                        </label>
                        <div class="form-row">
                            <label class="field"><span>"History days"</span>
                                <input class="input" type="number" prop:value=move || settings.get().history_days.to_string()
                                    on:input=move |ev| settings.update(|value| value.history_days = event_target_value(&ev).parse().unwrap_or(30)) />
                            </label>
                            <label class="field"><span>"Alert lookback (ชม.)"</span>
                                <input class="input" type="number" prop:value=move || settings.get().alert_lookback_hours.to_string()
                                    on:input=move |ev| settings.update(|value| value.alert_lookback_hours = event_target_value(&ev).parse().unwrap_or(72)) />
                            </label>
                        </div>
                        <label class="field checkbox-field">
                            <input type="checkbox" prop:checked=move || settings.get().auto_backup
                                on:change=move |ev| settings.update(|value| value.auto_backup = event_target_checked(&ev)) />
                            <span>"เปิด auto backup ในเครื่อง"</span>
                        </label>
                        <label class="field"><span>"จำนวน backup ที่เก็บ"</span>
                            <input class="input" type="number" prop:value=move || settings.get().backup_retention.to_string()
                                on:input=move |ev| settings.update(|value| value.backup_retention = event_target_value(&ev).parse().unwrap_or(14)) />
                        </label>
                        <button class="btn btn-primary" type="submit">"บันทึก Site Settings"</button>
                    </form>
                </Card>
            </div>

            <Card>
                <div class="panel-head"><div><div class="eyebrow">"LOCAL DATA"</div><h3>"Backup, Export, Import, Restore"</h3></div></div>
                <div class="data-actions">
                    <button class="btn btn-primary" on:click=move |_| {
                        spawn_local(async move {
                            match api::backup_database(None).await {
                                Ok(result) => notify(&ctx, format!("Backup แล้ว: {} ({} bytes)", result.path, result.bytes)),
                                Err(e) => notify_error(&ctx, e),
                            }
                        });
                    }>"Backup"
                    </button>
                    <button class="btn btn-secondary" on:click=move |_| {
                        spawn_local(async move {
                            match api::export_database(None).await {
                                Ok(result) => notify(&ctx, format!("Export แล้ว: {} ({} rows)", result.path, result.rows)),
                                Err(e) => notify_error(&ctx, e),
                            }
                        });
                    }>"Export JSON"</button>
                    <button class="btn btn-secondary" on:click=move |_| {
                        let mode = import_mode.get();
                        let dry = dry_run.get();
                        spawn_local(async move {
                            match api::import_database(None, Some(mode), Some(dry)).await {
                                Ok(result) => notify(&ctx, format!("Import {}: {} tables, {} rows", if result.dry_run { "dry-run" } else { "done" }, result.tables, result.rows)),
                                Err(e) => notify_error(&ctx, e),
                            }
                        });
                    }>"Import JSON"</button>
                    <button class="btn btn-danger" on:click=move |_| {
                        let confirmed = web_sys::window()
                            .and_then(|window| window.confirm_with_message("Restore จะแทนที่ฐานข้อมูลปัจจุบัน ต้องการดำเนินการหรือไม่").ok())
                            .unwrap_or(false);
                        if !confirmed { return; }
                        spawn_local(async move {
                            match api::restore_database(None).await {
                                Ok(result) => notify(&ctx, format!("Restore แล้ว: {}", result.path)),
                                Err(e) => notify_error(&ctx, e),
                            }
                        });
                    }>"Restore"</button>
                </div>
                <div class="form-row">
                    <label class="field"><span>"Import mode"</span>
                        <select class="input" prop:value=move || import_mode.get()
                            on:change=move |ev| import_mode.set(event_target_value(&ev))>
                            <option value="merge">"Merge"</option>
                            <option value="replace">"Replace"</option>
                        </select>
                    </label>
                    <label class="field checkbox-field">
                        <input type="checkbox" prop:checked=move || dry_run.get()
                            on:change=move |ev| dry_run.set(event_target_checked(&ev)) />
                        <span>"Dry run ก่อนเขียน"</span>
                    </label>
                </div>
                {move || status.get().map(|status| view! {
                    <div class="path-grid">
                        <div><span>"Database"</span><code>{status.db_path}</code></div>
                        <div><span>"Backup dir"</span><code>{status.backup_dir}</code></div>
                        <div><span>"App version"</span><code>{status.app_version}</code></div>
                        <div><span>"Configured"</span><code>{status.configured.to_string()}</code></div>
                    </div>
                })}
            </Card>

            <Card class="table-card">
                <div class="table-head"><div><div class="eyebrow">"AUDIT"</div><h3>"กิจกรรมล่าสุด"</h3></div></div>
                {move || {
                    let items = audit.get();
                    if items.is_empty() {
                        view! { <EmptyState title="ยังไม่มี audit" hint="ทุก action สำคัญจะถูกบันทึก" /> }.into_any()
                    } else {
                        view! {
                            <div class="table-wrap">
                                <table class="table">
                                    <thead><tr><th>"เวลา"</th><th>"Actor"</th><th>"Action"</th><th>"Entity"</th><th>"Detail"</th></tr></thead>
                                    <tbody>
                                        {items.into_iter().map(|entry| view! {
                                            <tr>
                                                <td class="mono">{entry.created_at.chars().take(19).collect::<String>()}</td>
                                                <td>{entry.actor}</td>
                                                <td class="strong">{entry.action}</td>
                                                <td class="mono">{format!("{} {}", entry.entity_type, entry.entity_id.unwrap_or_default())}</td>
                                                <td class="muted mono">{entry.detail_json.unwrap_or_default()}</td>
                                            </tr>
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }
                }}
            </Card>
        </div>
    }
}
