//! HAD Tracer desktop shell (Tauri 2 backend).

pub mod commands;
pub mod state;

use std::time::Duration;

use state::AppState;
use tauri::Manager;

const WINDOW_SHOW_FALLBACK: Duration = Duration::from_secs(5);

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "had_tracer_app=info,had_tracer_hosxp=info,had_tracer_store=info".into()
            }),
        )
        .init();

    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(WINDOW_SHOW_FALLBACK).await;
                    let _ = window.show();
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::show_main_window,
            commands::get_app_status,
            commands::is_configured,
            commands::get_connection,
            commands::save_connection,
            commands::get_settings,
            commands::save_settings,
            commands::test_connection,
            commands::connection_health,
            commands::schema_probe,
            commands::search_patients,
            commands::get_patient_cockpit,
            commands::search_drugitems,
            commands::search_lab_items,
            commands::list_had_drugs,
            commands::save_had_drug,
            commands::map_had_drug,
            commands::set_had_drug_active,
            commands::list_alert_rules,
            commands::save_alert_rule,
            commands::set_alert_rule_active,
            commands::evaluate_alerts,
            commands::list_alert_events,
            commands::alert_counts,
            commands::acknowledge_alert,
            commands::override_alert,
            commands::resolve_alert,
            commands::dashboard_metrics,
            commands::list_tracer_cases,
            commands::create_tracer_case,
            commands::get_tracer_workspace,
            commands::update_tracer_phase,
            commands::update_tracer_check,
            commands::close_tracer_case,
            commands::create_finding,
            commands::list_findings,
            commands::update_finding_status,
            commands::create_capa,
            commands::list_capas,
            commands::update_capa_status,
            commands::list_standard_refs,
            commands::list_kpi_definitions,
            commands::list_audit,
            commands::backup_database,
            commands::export_database,
            commands::import_database,
            commands::restore_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running HAD Tracer");
}
