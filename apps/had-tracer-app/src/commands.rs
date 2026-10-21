use std::path::PathBuf;

use chrono::Local;
use had_tracer_config::{ConnectionConfig, SiteSettings};
use had_tracer_core::{
    AlertEvent, AlertRule, AlertStatus, Capa, CapaStatus, CaseType, CauseCategory, ConnectionInfo,
    Finding, FindingSeverity, HadDrug, HadOrderView, PatientCockpit, PatientSummary,
    SchemaProbeReport, TracerCase, TracerCheck, TracerWorkspace, evaluate_rule, new_id,
    now_rfc3339, parse_lab_value,
};
use had_tracer_hosxp::{DrugItemRow, HosxpClient, HosxpConfig, LabItemRow};
use had_tracer_store::{
    AlertCounts, AuditEntry, BackupResult, DashboardMetrics, ExportResult, ImportOptions,
    ImportResult, KpiDefinition, StandardRef,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{State, WebviewWindow};

use crate::state::{AppState, ConnectionHealth};

const DEFAULT_ACTOR: &str = "local-user";

/// Typed backend error returned to the UI.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: String,
    pub message: String,
}

impl CommandError {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}

impl From<had_tracer_config::Error> for CommandError {
    fn from(error: had_tracer_config::Error) -> Self {
        Self::new("config", error.to_string())
    }
}

impl From<had_tracer_hosxp::Error> for CommandError {
    fn from(error: had_tracer_hosxp::Error) -> Self {
        let kind = match &error {
            had_tracer_hosxp::Error::Connect(_) => "connection",
            had_tracer_hosxp::Error::Guard(_) => "guard",
            had_tracer_hosxp::Error::Config(_) => "config",
            _ => "query",
        };
        Self::new(kind, error.to_string())
    }
}

impl From<had_tracer_store::StoreError> for CommandError {
    fn from(error: had_tracer_store::StoreError) -> Self {
        Self::new("store", error.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        Self::new("io", error.to_string())
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(error: serde_json::Error) -> Self {
        Self::new("json", error.to_string())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub configured: bool,
    pub site_name: String,
    pub db_path: String,
    pub backup_dir: String,
    pub app_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub tls_mode: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTest {
    pub latency_ms: u64,
    pub server_version: String,
    pub schema_ok: bool,
    pub schema_tables: usize,
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTracerInput {
    pub case_type: String,
    pub title: String,
    pub purpose: String,
    pub hn: String,
    pub an: Option<String>,
    pub vn: Option<String>,
    pub lead_user: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingInput {
    pub case_id: String,
    pub phase_id: Option<String>,
    pub title: String,
    pub description: String,
    pub standard_code: Option<String>,
    pub severity: String,
    pub likelihood: i64,
    pub cause_category: String,
    pub immediate_action: String,
    pub recommended_action: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapaInput {
    pub finding_id: String,
    pub action: String,
    pub owner: String,
    pub due_date: Option<String>,
    pub verification_method: String,
}

#[tauri::command]
pub fn show_main_window(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
}

#[tauri::command]
pub async fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, CommandError> {
    let settings = state.config().load_settings().unwrap_or_default();
    Ok(AppStatus {
        configured: state.config().connection_exists(),
        site_name: settings.site_name,
        db_path: state.config().database_path().display().to_string(),
        backup_dir: state.config().backup_dir().display().to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub async fn is_configured(state: State<'_, AppState>) -> Result<bool, CommandError> {
    Ok(state.config().connection_exists())
}

#[tauri::command]
pub async fn get_connection(
    state: State<'_, AppState>,
) -> Result<Option<ConnectionInfo>, CommandError> {
    if !state.config().connection_exists() {
        return Ok(None);
    }
    let connection = state.config().load_connection()?;
    let settings = state.config().load_settings().unwrap_or_default();
    Ok(Some(ConnectionInfo {
        host: connection.host,
        port: connection.port,
        database: connection.database,
        user: connection.user,
        site_name: settings.site_name,
    }))
}

#[tauri::command]
pub async fn save_connection(
    input: ConnectionInput,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<ConnectionInfo, CommandError> {
    if input.host.trim().is_empty()
        || input.database.trim().is_empty()
        || input.user.trim().is_empty()
    {
        return Err(CommandError::new(
            "invalid",
            "host, database และ user ต้องไม่ว่าง",
        ));
    }
    let tls_mode = input.tls_mode.unwrap_or_else(|| "preferred".to_string());
    let connection = ConnectionConfig {
        host: input.host.trim().to_string(),
        port: input.port,
        database: input.database.trim().to_string(),
        user: input.user.trim().to_string(),
        password: SecretString::from(input.password),
        tls_mode,
    };
    state.config().ensure_dirs()?;
    state.config().save_connection(&connection)?;
    state.reset_hosxp().await;
    let settings = state.config().load_settings().unwrap_or_default();
    let actor = actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string());
    state
        .store()
        .await?
        .audit(
            &actor,
            "save_connection",
            "config",
            None,
            json!({ "host": connection.host, "database": connection.database }),
        )
        .await?;
    Ok(ConnectionInfo {
        host: connection.host,
        port: connection.port,
        database: connection.database,
        user: connection.user,
        site_name: settings.site_name,
    })
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<SiteSettings, CommandError> {
    Ok(state.config().load_settings()?)
}

#[tauri::command]
pub async fn save_settings(
    settings: SiteSettings,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<SiteSettings, CommandError> {
    state.config().ensure_dirs()?;
    state.config().save_settings(&settings)?;
    let actor = actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string());
    state
        .store()
        .await?
        .audit(
            &actor,
            "save_settings",
            "config",
            None,
            json!({ "siteName": settings.site_name, "historyDays": settings.history_days }),
        )
        .await?;
    Ok(settings)
}

#[tauri::command]
pub async fn test_connection(input: ConnectionInput) -> Result<ConnectionTest, CommandError> {
    let config = HosxpConfig {
        host: input.host,
        port: input.port,
        database: input.database,
        user: input.user,
        password: SecretString::from(input.password),
        tls_mode: input.tls_mode.unwrap_or_else(|| "preferred".to_string()),
        statement_timeout_ms: 15_000,
    };
    let client = HosxpClient::connect(&config).await?;
    let latency_ms = client.ping().await?;
    let report = client.schema_probe().await?;
    let missing = missing_schema(&report);
    Ok(ConnectionTest {
        latency_ms,
        server_version: report.server_version,
        schema_ok: report.ok,
        schema_tables: report.tables.len(),
        missing,
        warnings: report.warnings,
    })
}

#[tauri::command]
pub async fn connection_health(
    state: State<'_, AppState>,
) -> Result<ConnectionHealth, CommandError> {
    if !state.config().connection_exists() {
        return Ok(ConnectionHealth::Unconfigured);
    }
    match state.hosxp().await {
        Ok(client) => match client.ping().await {
            Ok(_) => Ok(ConnectionHealth::Connected),
            Err(_) => {
                state.set_health(ConnectionHealth::Disconnected).await;
                Ok(ConnectionHealth::Disconnected)
            }
        },
        Err(_) => Ok(ConnectionHealth::Disconnected),
    }
}

#[tauri::command]
pub async fn schema_probe(state: State<'_, AppState>) -> Result<SchemaProbeReport, CommandError> {
    Ok(state.hosxp().await?.schema_probe().await?)
}

#[tauri::command]
pub async fn search_patients(
    query: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<PatientSummary>, CommandError> {
    Ok(state
        .hosxp()
        .await?
        .search_patients(&query, limit.unwrap_or(25))
        .await?)
}

#[tauri::command]
pub async fn get_patient_cockpit(
    hn: String,
    state: State<'_, AppState>,
) -> Result<PatientCockpit, CommandError> {
    load_cockpit(&state, &hn).await
}

#[tauri::command]
pub async fn search_drugitems(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<DrugItemRow>, CommandError> {
    Ok(state.hosxp().await?.search_drugitems(&query).await?)
}

#[tauri::command]
pub async fn search_lab_items(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<LabItemRow>, CommandError> {
    Ok(state.hosxp().await?.search_lab_items(&query).await?)
}

#[tauri::command]
pub async fn list_had_drugs(state: State<'_, AppState>) -> Result<Vec<HadDrug>, CommandError> {
    Ok(state.store().await?.list_had_drugs().await?)
}

#[tauri::command]
pub async fn save_had_drug(
    mut drug: HadDrug,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<HadDrug, CommandError> {
    if drug.name.trim().is_empty() || drug.drug_class.trim().is_empty() {
        return Err(CommandError::new("invalid", "ชื่อยาและกลุ่มยาต้องไม่ว่าง"));
    }
    if drug.id.trim().is_empty() {
        drug.id = new_id();
        drug.version = 1;
    } else {
        drug.version = drug.version.saturating_add(1);
    }
    let store = state.store().await?;
    store.upsert_had_drug(&drug).await?;
    let actor = actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string());
    store
        .audit(
            &actor,
            "save_had_drug",
            "had_drug",
            Some(&drug.id),
            json!({ "name": drug.name, "icode": drug.icode }),
        )
        .await?;
    Ok(drug)
}

#[tauri::command]
pub async fn map_had_drug(
    id: String,
    icode: String,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<(), CommandError> {
    if icode.trim().is_empty() {
        return Err(CommandError::new("invalid", "icode ต้องไม่ว่าง"));
    }
    let store = state.store().await?;
    store.map_had_drug(&id, &icode).await?;
    store
        .audit(
            &actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string()),
            "map_had_drug",
            "had_drug",
            Some(&id),
            json!({ "icode": icode }),
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn set_had_drug_active(
    id: String,
    active: bool,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    Ok(state
        .store()
        .await?
        .set_had_drug_active(&id, active)
        .await?)
}

#[tauri::command]
pub async fn list_alert_rules(state: State<'_, AppState>) -> Result<Vec<AlertRule>, CommandError> {
    Ok(state.store().await?.list_alert_rules().await?)
}

#[tauri::command]
pub async fn save_alert_rule(
    mut rule: AlertRule,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<AlertRule, CommandError> {
    if rule.lab_items_code.trim().is_empty() {
        return Err(CommandError::new("invalid", "ต้องระบุ lab items code"));
    }
    if rule.threshold_low.is_none() {
        return Err(CommandError::new("invalid", "ต้องระบุ threshold"));
    }
    if matches!(
        rule.operator,
        had_tracer_core::AlertOperator::Between | had_tracer_core::AlertOperator::Outside
    ) && rule.threshold_high.is_none()
    {
        return Err(CommandError::new("invalid", "rule แบบช่วงต้องมี threshold บน"));
    }
    if rule.id.trim().is_empty() {
        rule.id = new_id();
        rule.version = 1;
    } else {
        rule.version = rule.version.saturating_add(1);
    }
    if rule.created_by.is_none() {
        rule.created_by = Some(actor.clone().unwrap_or_else(|| DEFAULT_ACTOR.to_string()));
    }
    let store = state.store().await?;
    store.upsert_alert_rule(&rule).await?;
    store
        .audit(
            &actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string()),
            "save_alert_rule",
            "alert_rule",
            Some(&rule.id),
            json!({ "name": rule.name, "lab": rule.lab_items_code, "active": rule.active }),
        )
        .await?;
    Ok(rule)
}

#[tauri::command]
pub async fn set_alert_rule_active(
    id: String,
    active: bool,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    Ok(state
        .store()
        .await?
        .set_alert_rule_active(&id, active)
        .await?)
}

#[tauri::command]
pub async fn evaluate_alerts(
    hn: String,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<Vec<AlertEvent>, CommandError> {
    let store = state.store().await?;
    let rules = store.list_alert_rules().await?;
    let active_rules: Vec<AlertRule> = rules.into_iter().filter(|rule| rule.active).collect();
    if active_rules.is_empty() {
        return Ok(Vec::new());
    }
    let cockpit = load_cockpit(&state, &hn).await?;
    let mut created = Vec::new();
    for rule in &active_rules {
        for order in &cockpit.had_orders {
            if !rule_matches(rule, order) {
                continue;
            }
            let Some((raw, unit, observed_at, lab_order_number, lab_name)) =
                matching_lab(rule, &cockpit)
            else {
                continue;
            };
            let parsed = parse_lab_value(&raw);
            let evaluation = evaluate_rule(rule, &parsed);
            if !evaluation.triggered {
                continue;
            }
            let event = AlertEvent {
                id: new_id(),
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                hn: hn.clone(),
                an: order.an.clone(),
                vn: order.vn.clone(),
                drug_icode: Some(order.icode.clone()),
                drug_name: Some(order.drug_name.clone()),
                lab_order_number,
                lab_items_code: rule.lab_items_code.clone(),
                lab_items_name: lab_name.or(rule.lab_items_name.clone()),
                raw_value: parsed.raw.clone(),
                numeric_value: parsed.numeric,
                comparator: parsed.comparator.map(|value| value.as_str().to_string()),
                unit: unit.or(rule.unit.clone()),
                observed_at,
                severity: rule.severity,
                status: AlertStatus::New,
                action_text: rule.action_text.clone(),
                acknowledged_by: None,
                acknowledged_at: None,
                override_reason: None,
                resolved_by: None,
                resolved_at: None,
                resolve_note: None,
                created_at: now_rfc3339(),
            };
            if store.create_alert_event(&event).await? {
                created.push(event);
            }
        }
    }
    store
        .audit(
            &actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string()),
            "evaluate_alerts",
            "patient",
            Some(&hn),
            json!({ "created": created.len(), "rules": active_rules.len() }),
        )
        .await?;
    Ok(created)
}

#[tauri::command]
pub async fn list_alert_events(
    status: Option<String>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<AlertEvent>, CommandError> {
    Ok(state
        .store()
        .await?
        .list_alert_events(status.as_deref(), limit.unwrap_or(100))
        .await?)
}

#[tauri::command]
pub async fn alert_counts(state: State<'_, AppState>) -> Result<AlertCounts, CommandError> {
    Ok(state.store().await?.alert_counts().await?)
}

#[tauri::command]
pub async fn acknowledge_alert(
    id: String,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<(), CommandError> {
    let actor = actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string());
    let store = state.store().await?;
    store.acknowledge_alert(&id, &actor).await?;
    store
        .audit(
            &actor,
            "acknowledge_alert",
            "alert_event",
            Some(&id),
            json!({}),
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn override_alert(
    id: String,
    reason: String,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<(), CommandError> {
    let actor = actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string());
    let store = state.store().await?;
    store.override_alert(&id, &actor, &reason).await?;
    store
        .audit(
            &actor,
            "override_alert",
            "alert_event",
            Some(&id),
            json!({ "reason": reason }),
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn resolve_alert(
    id: String,
    note: String,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<(), CommandError> {
    let actor = actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string());
    let store = state.store().await?;
    store.resolve_alert(&id, &actor, &note).await?;
    store
        .audit(
            &actor,
            "resolve_alert",
            "alert_event",
            Some(&id),
            json!({ "note": note }),
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn dashboard_metrics(
    state: State<'_, AppState>,
) -> Result<DashboardMetrics, CommandError> {
    Ok(state.store().await?.dashboard_metrics().await?)
}

#[tauri::command]
pub async fn list_tracer_cases(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<TracerCase>, CommandError> {
    Ok(state
        .store()
        .await?
        .list_tracer_cases(limit.unwrap_or(100))
        .await?)
}

#[tauri::command]
pub async fn create_tracer_case(
    input: CreateTracerInput,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<TracerWorkspace, CommandError> {
    if input.title.trim().is_empty() || input.hn.trim().is_empty() {
        return Err(CommandError::new("invalid", "title และ HN ต้องไม่ว่าง"));
    }
    let mut case = TracerCase::new(
        CaseType::from_str(&input.case_type),
        input.title,
        input.purpose,
        input.hn,
        input.lead_user,
    );
    case.an = input.an;
    case.vn = input.vn;
    case.trigger_type = input.trigger_type;
    case.trigger_ref = input.trigger_ref;
    let store = state.store().await?;
    let workspace = store.create_tracer_case(&case).await?;
    store
        .audit(
            &actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string()),
            "create_tracer_case",
            "tracer_case",
            Some(&workspace.case.id),
            json!({ "title": workspace.case.title, "hn": workspace.case.hn }),
        )
        .await?;
    Ok(workspace)
}

#[tauri::command]
pub async fn get_tracer_workspace(
    id: String,
    state: State<'_, AppState>,
) -> Result<TracerWorkspace, CommandError> {
    Ok(state.store().await?.get_tracer_workspace(&id).await?)
}

#[tauri::command]
pub async fn update_tracer_phase(
    phase_id: String,
    status: String,
    summary: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    Ok(state
        .store()
        .await?
        .update_tracer_phase(&phase_id, &status, &summary)
        .await?)
}

#[tauri::command]
pub async fn update_tracer_check(
    check: TracerCheck,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    Ok(state.store().await?.update_tracer_check(&check).await?)
}

#[tauri::command]
pub async fn close_tracer_case(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    Ok(state.store().await?.close_tracer_case(&id).await?)
}

#[tauri::command]
pub async fn create_finding(
    input: FindingInput,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<Finding, CommandError> {
    if input.title.trim().is_empty() {
        return Err(CommandError::new("invalid", "ต้องระบุหัวข้อ finding"));
    }
    let mut finding = Finding::new(
        input.case_id,
        input.phase_id,
        input.title,
        input.description,
        FindingSeverity::from_str(&input.severity),
        input.likelihood,
        CauseCategory::from_str(&input.cause_category),
    );
    finding.standard_code = input.standard_code;
    finding.immediate_action = input.immediate_action;
    finding.recommended_action = input.recommended_action;
    let store = state.store().await?;
    store.create_finding(&finding).await?;
    store
        .audit(
            &actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string()),
            "create_finding",
            "finding",
            Some(&finding.id),
            json!({ "title": finding.title, "severity": finding.severity.as_str() }),
        )
        .await?;
    Ok(finding)
}

#[tauri::command]
pub async fn list_findings(
    case_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Finding>, CommandError> {
    Ok(state
        .store()
        .await?
        .list_findings(case_id.as_deref())
        .await?)
}

#[tauri::command]
pub async fn update_finding_status(
    id: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    Ok(state
        .store()
        .await?
        .update_finding_status(&id, &status)
        .await?)
}

#[tauri::command]
pub async fn create_capa(
    input: CapaInput,
    state: State<'_, AppState>,
    actor: Option<String>,
) -> Result<Capa, CommandError> {
    if input.action.trim().is_empty() || input.owner.trim().is_empty() {
        return Err(CommandError::new("invalid", "action และ owner ต้องไม่ว่าง"));
    }
    let capa = Capa {
        id: new_id(),
        finding_id: input.finding_id,
        action: input.action,
        owner: input.owner,
        due_date: input.due_date,
        status: CapaStatus::Open,
        verification_method: input.verification_method,
        verification_date: None,
        effectiveness_result: String::new(),
        evidence: String::new(),
        created_at: now_rfc3339(),
        closed_at: None,
    };
    let store = state.store().await?;
    store.create_capa(&capa).await?;
    store
        .audit(
            &actor.unwrap_or_else(|| DEFAULT_ACTOR.to_string()),
            "create_capa",
            "capa",
            Some(&capa.id),
            json!({ "findingId": capa.finding_id, "owner": capa.owner }),
        )
        .await?;
    Ok(capa)
}

#[tauri::command]
pub async fn list_capas(
    finding_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Capa>, CommandError> {
    Ok(state
        .store()
        .await?
        .list_capas(finding_id.as_deref())
        .await?)
}

#[tauri::command]
pub async fn update_capa_status(
    id: String,
    status: String,
    effectiveness_result: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    Ok(state
        .store()
        .await?
        .update_capa_status(&id, &status, &effectiveness_result)
        .await?)
}

#[tauri::command]
pub async fn list_standard_refs(
    state: State<'_, AppState>,
) -> Result<Vec<StandardRef>, CommandError> {
    Ok(state.store().await?.list_standard_refs().await?)
}

#[tauri::command]
pub async fn list_kpi_definitions(
    state: State<'_, AppState>,
) -> Result<Vec<KpiDefinition>, CommandError> {
    Ok(state.store().await?.list_kpi_definitions().await?)
}

#[tauri::command]
pub async fn list_audit(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<AuditEntry>, CommandError> {
    Ok(state
        .store()
        .await?
        .list_audit(limit.unwrap_or(100))
        .await?)
}

#[tauri::command]
pub async fn backup_database(
    path: Option<String>,
    state: State<'_, AppState>,
) -> Result<BackupResult, CommandError> {
    let default = state.config().backup_dir().join(format!(
        "had-tracer-backup-{}.db",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    let path = match path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => pick_save_path(default, "HAD Tracer backup".to_string(), "db".to_string()).await?,
    };
    Ok(state.store().await?.backup_to(&path).await?)
}

#[tauri::command]
pub async fn export_database(
    path: Option<String>,
    state: State<'_, AppState>,
) -> Result<ExportResult, CommandError> {
    let default = state.config().backup_dir().join(format!(
        "had-tracer-export-{}.json",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    let path = match path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => pick_save_path(default, "HAD Tracer export".to_string(), "json".to_string()).await?,
    };
    Ok(state.store().await?.export_json(&path).await?)
}

#[tauri::command]
pub async fn import_database(
    path: Option<String>,
    mode: Option<String>,
    dry_run: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ImportResult, CommandError> {
    let path = match path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => pick_open_path("HAD Tracer export".to_string(), "json".to_string()).await?,
    };
    let options = ImportOptions {
        mode: mode.unwrap_or_else(|| "merge".to_string()),
        dry_run: dry_run.unwrap_or(false),
    };
    Ok(state.store().await?.import_json(&path, &options).await?)
}

#[tauri::command]
pub async fn restore_database(
    path: Option<String>,
    state: State<'_, AppState>,
) -> Result<BackupResult, CommandError> {
    let path = match path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => pick_open_path("HAD Tracer backup".to_string(), "db".to_string()).await?,
    };
    had_tracer_store::verify_sqlite_file(&path).await?;
    let store = state.store().await?;
    let db_path = store.path().to_path_buf();
    store.checkpoint().await?;
    store.close().await;
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        let _ = tokio::fs::remove_file(sidecar).await;
    }

    let result = tokio::fs::copy(&path, &db_path).await;
    if let Err(error) = result {
        if let Ok(reopened) = had_tracer_store::Store::open(&db_path).await {
            state.set_store(reopened).await;
        }
        return Err(error.into());
    }
    let reopened = had_tracer_store::Store::open(&db_path).await?;
    let checksum = had_tracer_store::file_checksum(&path).await?;
    let bytes = tokio::fs::metadata(&path).await?.len();
    reopened
        .audit(
            DEFAULT_ACTOR,
            "restore",
            "database",
            None,
            json!({ "path": path.to_string_lossy(), "checksum": checksum }),
        )
        .await?;
    state.set_store(reopened).await;
    Ok(BackupResult {
        path: path.to_string_lossy().to_string(),
        manifest_path: format!("{}.manifest.json", path.display()),
        checksum,
        bytes,
        created_at: now_rfc3339(),
        integrity: "ok".to_string(),
    })
}

async fn load_cockpit(state: &AppState, hn: &str) -> Result<PatientCockpit, CommandError> {
    let store = state.store().await?;
    let had_drugs = store.list_had_drugs().await?;
    let rules = store.list_alert_rules().await?;
    let mut lab_codes: Vec<String> = rules
        .iter()
        .filter(|rule| rule.active)
        .map(|rule| rule.lab_items_code.clone())
        .filter(|code| !code.trim().is_empty())
        .collect();
    lab_codes.sort();
    lab_codes.dedup();
    let settings = state.config().load_settings().unwrap_or_default();
    let mut cockpit = state
        .hosxp()
        .await?
        .patient_cockpit(hn, &had_drugs, &lab_codes, settings.history_days)
        .await?;
    if had_drugs.iter().all(|drug| !drug.is_mapped()) {
        cockpit
            .warnings
            .push("ยังไม่มี HAD ที่ map กับ drugitems.icode".to_string());
    }
    if rules.iter().all(|rule| !rule.active) {
        cockpit
            .warnings
            .push("ยังไม่มี alert rule ที่เปิดใช้งาน".to_string());
    }
    Ok(cockpit)
}

fn rule_matches(rule: &AlertRule, order: &HadOrderView) -> bool {
    match rule.drug_scope_type.as_str() {
        "drug_icode" => rule
            .drug_scope_value
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(&order.icode)),
        "drug_class" => rule
            .drug_scope_value
            .as_deref()
            .is_some_and(|value| order.had_drug_class.as_deref() == Some(value)),
        "any_had" => true,
        _ => false,
    }
}

type LabEvidence = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn matching_lab(rule: &AlertRule, cockpit: &PatientCockpit) -> Option<LabEvidence> {
    if let Some(lab) = cockpit.labs.iter().find(|lab| {
        lab.lab_items_code
            .eq_ignore_ascii_case(rule.lab_items_code.trim())
    }) {
        return Some((
            lab.raw_value.clone(),
            lab.unit.clone(),
            lab.observed_at.clone(),
            Some(lab.lab_order_number.clone()),
            Some(lab.lab_items_name.clone()),
        ));
    }
    let wants_respiratory = rule.lab_items_code.eq_ignore_ascii_case("RR")
        || rule
            .lab_items_name
            .as_deref()
            .is_some_and(|name| name.to_ascii_lowercase().contains("respir"));
    if wants_respiratory
        && let Some(vital) = cockpit
            .vitals
            .iter()
            .find(|vital| vital.name.to_ascii_lowercase().contains("respir"))
    {
        return Some((
            vital.value.clone(),
            vital.unit.clone(),
            vital.observed_at.clone(),
            None,
            Some(vital.name.clone()),
        ));
    }
    None
}

fn missing_schema(report: &SchemaProbeReport) -> Vec<String> {
    let mut missing = Vec::new();
    for table in &report.tables {
        if !table.present {
            missing.push(format!("{} (table missing)", table.table));
        } else {
            for column in &table.missing_columns {
                missing.push(format!("{}.{column}", table.table));
            }
        }
    }
    missing
}

async fn pick_save_path(
    default: PathBuf,
    filter_name: String,
    extension: String,
) -> Result<PathBuf, CommandError> {
    let file_name = default
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "had-tracer".to_string());
    let directory = default.parent().map(PathBuf::from);
    let result = tokio::task::spawn_blocking(move || {
        let extensions = [extension.as_str()];
        let mut dialog = rfd::FileDialog::new()
            .add_filter(&filter_name, &extensions)
            .set_file_name(&file_name);
        if let Some(directory) = directory {
            dialog = dialog.set_directory(directory);
        }
        dialog.save_file()
    })
    .await
    .map_err(|error| CommandError::new("io", error.to_string()))?;
    result.ok_or_else(|| CommandError::new("cancelled", "ยกเลิกการบันทึก"))
}

async fn pick_open_path(filter_name: String, extension: String) -> Result<PathBuf, CommandError> {
    let result = tokio::task::spawn_blocking(move || {
        let extensions = [extension.as_str()];
        rfd::FileDialog::new()
            .add_filter(&filter_name, &extensions)
            .pick_file()
    })
    .await
    .map_err(|error| CommandError::new("io", error.to_string()))?;
    result.ok_or_else(|| CommandError::new("cancelled", "ยกเลิกการเลือกไฟล์"))
}
