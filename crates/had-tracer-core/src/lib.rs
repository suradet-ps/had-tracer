//! Domain model for HAD Tracer.
//!
//! This crate is pure: no SQL, no filesystem, no Tauri, no network. Every
//! other crate depends on these types and rules.

pub mod alert;
pub mod error;
pub mod model;
pub mod tracer;

pub use alert::{
    AlertEvaluation, AlertEvent, AlertOperator, AlertRule, AlertStatus, Comparator, ParsedLabValue,
    evaluate_rule, parse_lab_value,
};
pub use error::CoreError;
pub use model::{
    AllergyView, ConnectionInfo, DiagnosisView, HadDrug, HadOrderView, HadSeverity, LabResultView,
    PatientCockpit, PatientSummary, SafetyBundle, SchemaCapability, SchemaProbeReport,
    VitalSignView, new_id, now_rfc3339,
};
pub use tracer::{
    Capa, CapaStatus, CaseType, CauseCategory, Finding, FindingSeverity, FindingStatus, PhaseCode,
    TracerCase, TracerCheck, TracerPhase, TracerStatus, TracerWorkspace,
};

/// Return a short, human-readable label for a severity.
pub fn severity_label(severity: HadSeverity) -> &'static str {
    match severity {
        HadSeverity::Critical => "Critical",
        HadSeverity::High => "High",
        HadSeverity::Medium => "Medium",
        HadSeverity::Low => "Low",
        HadSeverity::Info => "Info",
    }
}
