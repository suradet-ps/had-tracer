//! Frontend API layer. Every call is a thin Tauri invoke.

pub use had_tracer_core::{
    AlertEvent, AlertRule, Capa, ConnectionInfo, Finding, HadDrug, PatientCockpit, PatientSummary,
    SchemaProbeReport, TracerCase, TracerCheck, TracerWorkspace,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub kind: String,
    pub message: String,
}

impl ApiError {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}

impl From<had_tracer_bridge::BridgeError> for ApiError {
    fn from(error: had_tracer_bridge::BridgeError) -> Self {
        if let had_tracer_bridge::BridgeError::Command(text) = &error
            && let Ok(typed) = serde_json::from_str::<ApiError>(text)
        {
            return typed;
        }
        Self::new("bridge", error.to_string())
    }
}

async fn call<T: serde::de::DeserializeOwned>(
    command: &str,
    args: impl Serialize,
) -> Result<T, ApiError> {
    had_tracer_bridge::invoke::<T>(command, args)
        .await
        .map_err(ApiError::from)
}

async fn call_empty<T: serde::de::DeserializeOwned>(command: &str) -> Result<T, ApiError> {
    call(command, serde_json::json!({})).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub configured: bool,
    pub site_name: String,
    pub db_path: String,
    pub backup_dir: String,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteSettings {
    pub site_name: String,
    pub history_days: u32,
    pub alert_lookback_hours: i64,
    pub auto_backup: bool,
    pub backup_retention: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub tls_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTest {
    pub latency_ms: u64,
    pub server_version: String,
    pub schema_ok: bool,
    pub schema_tables: usize,
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMetrics {
    pub had_total: i64,
    pub had_mapped: i64,
    pub active_rules: i64,
    pub open_alerts: i64,
    pub critical_alerts: i64,
    pub tracer_open: i64,
    pub findings_open: i64,
    pub capa_overdue: i64,
    pub audit_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertCounts {
    pub new: i64,
    pub acknowledged: i64,
    pub overridden: i64,
    pub resolved: i64,
    pub critical: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrugItemRow {
    pub icode: String,
    pub name: String,
    pub strength: Option<String>,
    pub units: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabItemRow {
    pub lab_items_code: String,
    pub lab_items_name: String,
    pub lab_items_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardRef {
    pub id: String,
    pub framework: String,
    pub code: String,
    pub title: String,
    pub requirement: String,
    pub edition: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDefinition {
    pub id: String,
    pub code: String,
    pub name: String,
    pub domain: String,
    pub numerator: String,
    pub denominator: String,
    pub data_source: String,
    pub frequency: String,
    pub target: Option<String>,
    pub owner: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: String,
    pub actor: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub detail_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub path: String,
    pub manifest_path: String,
    pub checksum: String,
    pub bytes: u64,
    pub created_at: String,
    pub integrity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub path: String,
    pub checksum: String,
    pub bytes: u64,
    pub tables: usize,
    pub rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub tables: usize,
    pub rows: usize,
    pub mode: String,
    pub dry_run: bool,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapaInput {
    pub finding_id: String,
    pub action: String,
    pub owner: String,
    pub due_date: Option<String>,
    pub verification_method: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionHealth {
    Unconfigured,
    Connecting,
    Connected,
    Degraded,
    Disconnected,
}

pub async fn show_main_window() -> Result<(), ApiError> {
    call_empty("show_main_window").await
}

pub async fn get_app_status() -> Result<AppStatus, ApiError> {
    call_empty("get_app_status").await
}

pub async fn get_connection() -> Result<Option<ConnectionInfo>, ApiError> {
    call_empty("get_connection").await
}

pub async fn save_connection(input: ConnectionInput) -> Result<ConnectionInfo, ApiError> {
    call("save_connection", serde_json::json!({ "input": input })).await
}

pub async fn test_connection(input: ConnectionInput) -> Result<ConnectionTest, ApiError> {
    call("test_connection", serde_json::json!({ "input": input })).await
}

pub async fn connection_health() -> Result<ConnectionHealth, ApiError> {
    call_empty("connection_health").await
}

pub async fn schema_probe() -> Result<SchemaProbeReport, ApiError> {
    call_empty("schema_probe").await
}

pub async fn get_settings() -> Result<SiteSettings, ApiError> {
    call_empty("get_settings").await
}

pub async fn save_settings(settings: SiteSettings) -> Result<SiteSettings, ApiError> {
    call("save_settings", serde_json::json!({ "settings": settings })).await
}

pub async fn search_patients(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<PatientSummary>, ApiError> {
    call(
        "search_patients",
        serde_json::json!({ "query": query, "limit": limit }),
    )
    .await
}

pub async fn get_patient_cockpit(hn: String) -> Result<PatientCockpit, ApiError> {
    call("get_patient_cockpit", serde_json::json!({ "hn": hn })).await
}

pub async fn search_drugitems(query: String) -> Result<Vec<DrugItemRow>, ApiError> {
    call("search_drugitems", serde_json::json!({ "query": query })).await
}

pub async fn search_lab_items(query: String) -> Result<Vec<LabItemRow>, ApiError> {
    call("search_lab_items", serde_json::json!({ "query": query })).await
}

pub async fn list_had_drugs() -> Result<Vec<HadDrug>, ApiError> {
    call_empty("list_had_drugs").await
}

pub async fn save_had_drug(drug: HadDrug) -> Result<HadDrug, ApiError> {
    call("save_had_drug", serde_json::json!({ "drug": drug })).await
}

pub async fn map_had_drug(id: String, icode: String) -> Result<(), ApiError> {
    call(
        "map_had_drug",
        serde_json::json!({ "id": id, "icode": icode }),
    )
    .await
}

pub async fn set_had_drug_active(id: String, active: bool) -> Result<(), ApiError> {
    call(
        "set_had_drug_active",
        serde_json::json!({ "id": id, "active": active }),
    )
    .await
}

pub async fn list_alert_rules() -> Result<Vec<AlertRule>, ApiError> {
    call_empty("list_alert_rules").await
}

pub async fn save_alert_rule(rule: AlertRule) -> Result<AlertRule, ApiError> {
    call("save_alert_rule", serde_json::json!({ "rule": rule })).await
}

pub async fn set_alert_rule_active(id: String, active: bool) -> Result<(), ApiError> {
    call(
        "set_alert_rule_active",
        serde_json::json!({ "id": id, "active": active }),
    )
    .await
}

pub async fn evaluate_alerts(hn: String) -> Result<Vec<AlertEvent>, ApiError> {
    call("evaluate_alerts", serde_json::json!({ "hn": hn })).await
}

pub async fn list_alert_events(
    status: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<AlertEvent>, ApiError> {
    call(
        "list_alert_events",
        serde_json::json!({ "status": status, "limit": limit }),
    )
    .await
}

pub async fn alert_counts() -> Result<AlertCounts, ApiError> {
    call_empty("alert_counts").await
}

pub async fn acknowledge_alert(id: String) -> Result<(), ApiError> {
    call("acknowledge_alert", serde_json::json!({ "id": id })).await
}

pub async fn override_alert(id: String, reason: String) -> Result<(), ApiError> {
    call(
        "override_alert",
        serde_json::json!({ "id": id, "reason": reason }),
    )
    .await
}

pub async fn resolve_alert(id: String, note: String) -> Result<(), ApiError> {
    call(
        "resolve_alert",
        serde_json::json!({ "id": id, "note": note }),
    )
    .await
}

pub async fn dashboard_metrics() -> Result<DashboardMetrics, ApiError> {
    call_empty("dashboard_metrics").await
}

pub async fn list_tracer_cases(limit: Option<u32>) -> Result<Vec<TracerCase>, ApiError> {
    call("list_tracer_cases", serde_json::json!({ "limit": limit })).await
}

pub async fn create_tracer_case(input: CreateTracerInput) -> Result<TracerWorkspace, ApiError> {
    call("create_tracer_case", serde_json::json!({ "input": input })).await
}

pub async fn get_tracer_workspace(id: String) -> Result<TracerWorkspace, ApiError> {
    call("get_tracer_workspace", serde_json::json!({ "id": id })).await
}

pub async fn update_tracer_phase(
    phase_id: String,
    status: String,
    summary: String,
) -> Result<(), ApiError> {
    call(
        "update_tracer_phase",
        serde_json::json!({ "phaseId": phase_id, "status": status, "summary": summary }),
    )
    .await
}

pub async fn update_tracer_check(check: TracerCheck) -> Result<(), ApiError> {
    call("update_tracer_check", serde_json::json!({ "check": check })).await
}

pub async fn close_tracer_case(id: String) -> Result<(), ApiError> {
    call("close_tracer_case", serde_json::json!({ "id": id })).await
}

pub async fn create_finding(input: FindingInput) -> Result<Finding, ApiError> {
    call("create_finding", serde_json::json!({ "input": input })).await
}

pub async fn list_findings(case_id: Option<String>) -> Result<Vec<Finding>, ApiError> {
    call("list_findings", serde_json::json!({ "caseId": case_id })).await
}

pub async fn update_finding_status(id: String, status: String) -> Result<(), ApiError> {
    call(
        "update_finding_status",
        serde_json::json!({ "id": id, "status": status }),
    )
    .await
}

pub async fn create_capa(input: CapaInput) -> Result<Capa, ApiError> {
    call("create_capa", serde_json::json!({ "input": input })).await
}

pub async fn list_capas(finding_id: Option<String>) -> Result<Vec<Capa>, ApiError> {
    call("list_capas", serde_json::json!({ "findingId": finding_id })).await
}

pub async fn update_capa_status(
    id: String,
    status: String,
    effectiveness_result: String,
) -> Result<(), ApiError> {
    call(
        "update_capa_status",
        serde_json::json!({ "id": id, "status": status, "effectivenessResult": effectiveness_result }),
    )
    .await
}

pub async fn list_standard_refs() -> Result<Vec<StandardRef>, ApiError> {
    call_empty("list_standard_refs").await
}

pub async fn list_kpi_definitions() -> Result<Vec<KpiDefinition>, ApiError> {
    call_empty("list_kpi_definitions").await
}

pub async fn list_audit(limit: Option<u32>) -> Result<Vec<AuditEntry>, ApiError> {
    call("list_audit", serde_json::json!({ "limit": limit })).await
}

pub async fn backup_database(path: Option<String>) -> Result<BackupResult, ApiError> {
    call("backup_database", serde_json::json!({ "path": path })).await
}

pub async fn export_database(path: Option<String>) -> Result<ExportResult, ApiError> {
    call("export_database", serde_json::json!({ "path": path })).await
}

pub async fn import_database(
    path: Option<String>,
    mode: Option<String>,
    dry_run: Option<bool>,
) -> Result<ImportResult, ApiError> {
    call(
        "import_database",
        serde_json::json!({ "path": path, "mode": mode, "dryRun": dry_run }),
    )
    .await
}

pub async fn restore_database(path: Option<String>) -> Result<BackupResult, ApiError> {
    call("restore_database", serde_json::json!({ "path": path })).await
}

/// Sleep helper for the health poll loop.
pub async fn sleep_ms(ms: i32) {
    use wasm_bindgen::JsCast;
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        if let Some(window) = web_sys::window() {
            let _ = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(resolve.unchecked_ref(), ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}
