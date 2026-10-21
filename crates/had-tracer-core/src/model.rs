use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generate a new UUIDv4 string. All app-owned entity ids use this form.
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// Current UTC timestamp as RFC 3339.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// HAD severity, ordered from most to least urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HadSeverity {
    Critical,
    High,
    #[default]
    Medium,
    Low,
    Info,
}

impl HadSeverity {
    /// Stable database and filter value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Info => "info",
        }
    }

    /// Parse from storage.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "critical" => Self::Critical,
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            _ => Self::Info,
        }
    }

    /// Thai label for the UI.
    pub fn label_th(self) -> &'static str {
        match self {
            Self::Critical => "วิกฤต",
            Self::High => "สูง",
            Self::Medium => "ปานกลาง",
            Self::Low => "ต่ำ",
            Self::Info => "ข้อมูล",
        }
    }
}

/// Structure that holds the per-drug safety bundle.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SafetyBundle {
    pub indications: Vec<String>,
    pub usual_dose: String,
    pub max_dose: String,
    pub max_rate: String,
    pub concentration_limit: String,
    pub diluent: String,
    pub compatibility: String,
    pub line_filter: String,
    pub monitoring: Vec<String>,
    pub monitoring_frequency: String,
    pub hold_criteria: String,
    pub antidote: String,
    pub double_check_required: bool,
    pub storage: String,
    pub access_control: String,
    pub look_alike_sound_alike: Vec<String>,
    pub high_risk_populations: Vec<String>,
    pub patient_education: Vec<String>,
    pub references: Vec<String>,
    pub notes: String,
}

/// A high-alert drug catalog entry owned by the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HadDrug {
    pub id: String,
    pub icode: Option<String>,
    pub name: String,
    pub generic_name: Option<String>,
    pub drug_class: String,
    pub route: Option<String>,
    pub severity: HadSeverity,
    pub safety_bundle: SafetyBundle,
    pub active: bool,
    pub version: i64,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub effective_at: Option<String>,
    pub review_due_at: Option<String>,
}

impl HadDrug {
    /// Whether this entry is mapped to a real `drugitems.icode`.
    pub fn is_mapped(&self) -> bool {
        self.icode
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
    }
}

/// A patient identity returned from HOSxP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientSummary {
    pub hn: String,
    pub name: String,
    pub birthday: Option<String>,
    pub cid_masked: Option<String>,
}

/// A high-alert order or dispensing line from `opitemrece`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HadOrderView {
    pub vn: Option<String>,
    pub an: Option<String>,
    pub icode: String,
    pub drug_name: String,
    pub strength: Option<String>,
    pub units: Option<String>,
    pub vstdate: Option<String>,
    pub qty: Option<i64>,
    pub sig: Option<String>,
    pub special_use: Option<String>,
    pub income_name: Option<String>,
    pub had_drug_id: Option<String>,
    pub had_drug_class: Option<String>,
    pub severity: HadSeverity,
}

/// A laboratory result prepared for the UI and rule engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabResultView {
    pub lab_order_number: String,
    pub lab_items_code: String,
    pub lab_items_name: String,
    pub unit: Option<String>,
    pub raw_value: String,
    pub numeric_value: Option<f64>,
    pub comparator: Option<String>,
    pub observed_at: Option<String>,
}

/// A vital sign from OPD or IPD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VitalSignView {
    pub source: String,
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
    pub observed_at: Option<String>,
    pub std_min: Option<f64>,
    pub std_max: Option<f64>,
}

/// An allergy record from `opd_allergy`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllergyView {
    pub agent: String,
    pub symptom: Option<String>,
    pub reporter: Option<String>,
    pub report_date: Option<String>,
    pub note: Option<String>,
}

/// A diagnosis with a resolved ICD label where possible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisView {
    pub icd10: String,
    pub name: Option<String>,
    pub diag_type: Option<String>,
    pub source: String,
    pub encounter: Option<String>,
}

/// Everything the patient cockpit needs in one payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientCockpit {
    pub patient: PatientSummary,
    pub active_an: Option<String>,
    pub active_ward: Option<String>,
    pub active_admission_date: Option<String>,
    pub had_orders: Vec<HadOrderView>,
    pub labs: Vec<LabResultView>,
    pub vitals: Vec<VitalSignView>,
    pub allergies: Vec<AllergyView>,
    pub diagnoses: Vec<DiagnosisView>,
    pub warnings: Vec<String>,
}

/// One table in the live HOSxP schema capability report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaCapability {
    pub table: String,
    pub present: bool,
    pub missing_columns: Vec<String>,
}

/// Result of probing a live HOSxP database before use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaProbeReport {
    pub server_version: String,
    pub database: String,
    pub checked_at: String,
    pub tables: Vec<SchemaCapability>,
    pub ok: bool,
    pub warnings: Vec<String>,
}

/// Non-secret connection summary returned to the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub site_name: String,
}
