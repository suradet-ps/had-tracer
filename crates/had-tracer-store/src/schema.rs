//! SQLite schema for app-owned data.
//!
//! The schema is versioned through `PRAGMA user_version`. Migration 1 is the
//! initial schema. Future migrations must be additive and must preserve local
//! tracer evidence.

pub const SCHEMA_VERSION: i64 = 1;

pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS had_drug (
    id TEXT PRIMARY KEY,
    icode TEXT UNIQUE,
    name TEXT NOT NULL,
    generic_name TEXT,
    drug_class TEXT NOT NULL,
    route TEXT,
    severity TEXT NOT NULL,
    safety_bundle_json TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    version INTEGER NOT NULL DEFAULT 1,
    approved_by TEXT,
    approved_at TEXT,
    effective_at TEXT,
    review_due_at TEXT
);

CREATE TABLE IF NOT EXISTS alert_rule (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    drug_scope_type TEXT NOT NULL,
    drug_scope_value TEXT,
    lab_items_code TEXT NOT NULL,
    lab_items_name TEXT,
    operator TEXT NOT NULL,
    threshold_low REAL,
    threshold_high REAL,
    unit TEXT,
    lookback_hours INTEGER NOT NULL DEFAULT 72,
    severity TEXT NOT NULL,
    action_text TEXT NOT NULL DEFAULT '',
    require_active_order INTEGER NOT NULL DEFAULT 1,
    active INTEGER NOT NULL DEFAULT 1,
    version INTEGER NOT NULL DEFAULT 1,
    created_by TEXT,
    approved_by TEXT,
    approved_at TEXT,
    effective_at TEXT,
    review_due_at TEXT
);

CREATE TABLE IF NOT EXISTS alert_event (
    id TEXT PRIMARY KEY,
    dedupe_key TEXT NOT NULL UNIQUE,
    rule_id TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    hn TEXT NOT NULL,
    an TEXT,
    vn TEXT,
    drug_icode TEXT,
    drug_name TEXT,
    lab_order_number TEXT,
    lab_items_code TEXT NOT NULL,
    lab_items_name TEXT,
    raw_value TEXT NOT NULL,
    numeric_value REAL,
    comparator TEXT,
    unit TEXT,
    observed_at TEXT,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    action_text TEXT NOT NULL DEFAULT '',
    acknowledged_by TEXT,
    acknowledged_at TEXT,
    override_reason TEXT,
    resolved_by TEXT,
    resolved_at TEXT,
    resolve_note TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tracer_case (
    id TEXT PRIMARY KEY,
    case_type TEXT NOT NULL,
    title TEXT NOT NULL,
    purpose TEXT NOT NULL,
    hn TEXT NOT NULL,
    an TEXT,
    vn TEXT,
    trigger_type TEXT,
    trigger_ref TEXT,
    lead_user TEXT,
    status TEXT NOT NULL,
    opened_at TEXT NOT NULL,
    closed_at TEXT
);

CREATE TABLE IF NOT EXISTS tracer_phase (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES tracer_case(id) ON DELETE CASCADE,
    phase_code TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    summary TEXT NOT NULL DEFAULT '',
    UNIQUE(case_id, phase_code)
);

CREATE TABLE IF NOT EXISTS tracer_check (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES tracer_case(id) ON DELETE CASCADE,
    phase_id TEXT NOT NULL REFERENCES tracer_phase(id) ON DELETE CASCADE,
    standard_code TEXT,
    question TEXT NOT NULL,
    expected TEXT NOT NULL DEFAULT '',
    observed TEXT NOT NULL DEFAULT '',
    gap TEXT NOT NULL DEFAULT '',
    evidence_summary TEXT NOT NULL DEFAULT '',
    finding_id TEXT,
    UNIQUE(phase_id, question)
);

CREATE TABLE IF NOT EXISTS tracer_evidence (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES tracer_case(id) ON DELETE CASCADE,
    phase_id TEXT,
    evidence_type TEXT NOT NULL,
    title TEXT NOT NULL,
    source TEXT,
    captured_at TEXT,
    captured_by TEXT,
    content TEXT,
    file_path TEXT
);

CREATE TABLE IF NOT EXISTS finding (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES tracer_case(id) ON DELETE CASCADE,
    phase_id TEXT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    standard_code TEXT,
    severity TEXT NOT NULL,
    likelihood INTEGER NOT NULL DEFAULT 3,
    risk_score INTEGER NOT NULL DEFAULT 6,
    cause_category TEXT NOT NULL,
    immediate_action TEXT NOT NULL DEFAULT '',
    recommended_action TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS capa (
    id TEXT PRIMARY KEY,
    finding_id TEXT NOT NULL REFERENCES finding(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    owner TEXT NOT NULL,
    due_date TEXT,
    status TEXT NOT NULL,
    verification_method TEXT NOT NULL DEFAULT '',
    verification_date TEXT,
    effectiveness_result TEXT NOT NULL DEFAULT '',
    evidence TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    closed_at TEXT
);

CREATE TABLE IF NOT EXISTS standard_ref (
    id TEXT PRIMARY KEY,
    framework TEXT NOT NULL,
    code TEXT NOT NULL,
    title TEXT NOT NULL,
    requirement TEXT NOT NULL,
    edition TEXT,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS kpi_definition (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    numerator TEXT NOT NULL,
    denominator TEXT NOT NULL,
    data_source TEXT NOT NULL,
    frequency TEXT NOT NULL,
    target TEXT,
    owner TEXT,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS kpi_value (
    id TEXT PRIMARY KEY,
    kpi_definition_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    numerator_value REAL,
    denominator_value REAL,
    result_value REAL,
    note TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    detail_json TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_alert_event_status ON alert_event(status, severity);
CREATE INDEX IF NOT EXISTS idx_alert_event_hn ON alert_event(hn);
CREATE INDEX IF NOT EXISTS idx_tracer_case_status ON tracer_case(status);
CREATE INDEX IF NOT EXISTS idx_finding_case ON finding(case_id, status);
CREATE INDEX IF NOT EXISTS idx_capa_status ON capa(status, due_date);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_log(created_at);
"#;

/// Tables included in JSON export and import.
pub const EXPORT_TABLES: &[&str] = &[
    "app_meta",
    "had_drug",
    "alert_rule",
    "alert_event",
    "tracer_case",
    "tracer_phase",
    "tracer_check",
    "tracer_evidence",
    "finding",
    "capa",
    "standard_ref",
    "kpi_definition",
    "kpi_value",
    "audit_log",
];
