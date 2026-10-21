use had_tracer_core::{AlertOperator, AlertRule, HadDrug, HadSeverity, SafetyBundle, new_id};

use crate::Store;
use crate::error::Result;

/// Seed the catalog, rule templates, standards, and KPIs on first run.
pub async fn seed_if_empty(store: &Store) -> Result<()> {
    let had_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM had_drug")
        .fetch_one(&store.pool)
        .await?;
    if had_count == 0 {
        for drug in default_had_drugs() {
            store.upsert_had_drug(&drug).await?;
        }
    }

    let rule_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_rule")
        .fetch_one(&store.pool)
        .await?;
    if rule_count == 0 {
        for rule in default_rules() {
            store.upsert_alert_rule(&rule).await?;
        }
    }

    let standard_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM standard_ref")
        .fetch_one(&store.pool)
        .await?;
    if standard_count == 0 {
        seed_standards(store).await?;
    }

    let kpi_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kpi_definition")
        .fetch_one(&store.pool)
        .await?;
    if kpi_count == 0 {
        seed_kpis(store).await?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn drug(
    name: &str,
    generic_name: &str,
    drug_class: &str,
    route: &str,
    severity: HadSeverity,
    monitoring: &[&str],
    antidote: &str,
    double_check: bool,
    hold_criteria: &str,
) -> HadDrug {
    HadDrug {
        id: new_id(),
        icode: None,
        name: name.to_string(),
        generic_name: Some(generic_name.to_string()),
        drug_class: drug_class.to_string(),
        route: Some(route.to_string()),
        severity,
        safety_bundle: SafetyBundle {
            monitoring: monitoring
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            antidote: antidote.to_string(),
            hold_criteria: hold_criteria.to_string(),
            double_check_required: double_check,
            ..SafetyBundle::default()
        },
        active: true,
        version: 1,
        approved_by: None,
        approved_at: None,
        effective_at: None,
        review_due_at: None,
    }
}

fn default_had_drugs() -> Vec<HadDrug> {
    vec![
        drug(
            "Warfarin",
            "warfarin",
            "anticoagulant_warfarin",
            "oral",
            HadSeverity::High,
            &["INR", "Hb/Hct", "bleeding signs"],
            "Vitamin K",
            true,
            "Hold if INR above target or active bleeding",
        ),
        drug(
            "Heparin injection",
            "unfractionated heparin",
            "anticoagulant_heparin",
            "IV",
            HadSeverity::High,
            &["aPTT", "platelet count", "bleeding signs"],
            "Protamine sulfate",
            true,
            "Hold if aPTT supratherapeutic or platelets fall",
        ),
        drug(
            "Enoxaparin",
            "enoxaparin",
            "anticoagulant_lmwh",
            "SC",
            HadSeverity::High,
            &["platelet count", "renal function", "bleeding signs"],
            "Protamine sulfate (partial)",
            true,
            "Hold if severe bleeding or renal impairment",
        ),
        drug(
            "Apixaban",
            "apixaban",
            "anticoagulant_doac",
            "oral",
            HadSeverity::High,
            &["renal function", "Hb", "bleeding signs"],
            "Andexanet alfa where available",
            false,
            "Hold if severe bleeding or planned procedure",
        ),
        drug(
            "Rivaroxaban",
            "rivaroxaban",
            "anticoagulant_doac",
            "oral",
            HadSeverity::High,
            &["renal function", "Hb", "bleeding signs"],
            "Andexanet alfa where available",
            false,
            "Hold if severe bleeding or planned procedure",
        ),
        drug(
            "Insulin regular",
            "insulin human regular",
            "insulin",
            "IV or SC",
            HadSeverity::High,
            &["blood glucose", "potassium", "hypoglycemia signs"],
            "Glucose 50% or dextrose",
            true,
            "Hold if glucose below protocol threshold",
        ),
        drug(
            "Insulin glargine",
            "insulin glargine",
            "insulin",
            "SC",
            HadSeverity::High,
            &["blood glucose", "hypoglycemia signs"],
            "Glucose 50% or dextrose",
            true,
            "Hold if glucose below protocol threshold",
        ),
        drug(
            "Morphine injection",
            "morphine sulfate",
            "opioid",
            "IV or SC",
            HadSeverity::High,
            &["respiratory rate", "sedation score", "pain score"],
            "Naloxone",
            true,
            "Hold if respiratory rate is low or sedation is deep",
        ),
        drug(
            "Fentanyl injection",
            "fentanyl",
            "opioid",
            "IV",
            HadSeverity::High,
            &["respiratory rate", "sedation score", "pain score"],
            "Naloxone",
            true,
            "Hold if respiratory rate is low or sedation is deep",
        ),
        drug(
            "Oxycodone oral",
            "oxycodone",
            "opioid",
            "oral",
            HadSeverity::High,
            &["respiratory rate", "sedation score", "pain score"],
            "Naloxone",
            false,
            "Hold if respiratory rate is low or sedation is deep",
        ),
        drug(
            "Potassium chloride concentrate",
            "potassium chloride",
            "electrolyte_potassium",
            "IV",
            HadSeverity::Critical,
            &["serum potassium", "ECG", "infusion site"],
            "Calcium gluconate, insulin with glucose, dialysis",
            true,
            "Never give undiluted; hold if potassium is high",
        ),
        drug(
            "Hypertonic sodium chloride",
            "sodium chloride 3% or more",
            "electrolyte_sodium",
            "IV",
            HadSeverity::High,
            &["serum sodium", "neurologic status", "infusion site"],
            "Free water replacement",
            true,
            "Hold if sodium correction is too rapid",
        ),
        drug(
            "Magnesium sulfate injection",
            "magnesium sulfate",
            "electrolyte_magnesium",
            "IV",
            HadSeverity::High,
            &[
                "magnesium",
                "respiratory rate",
                "deep tendon reflexes",
                "urine output",
            ],
            "Calcium gluconate",
            true,
            "Hold if reflexes are absent or respiratory rate is low",
        ),
        drug(
            "Midazolam injection",
            "midazolam",
            "sedative",
            "IV",
            HadSeverity::High,
            &["respiratory rate", "sedation score", "blood pressure"],
            "Flumazenil",
            true,
            "Hold if sedation is deeper than target",
        ),
        drug(
            "Propofol injection",
            "propofol",
            "sedative",
            "IV",
            HadSeverity::High,
            &["respiratory rate", "sedation score", "blood pressure"],
            "Supportive airway management",
            true,
            "Hold if hypotension or oversedation",
        ),
        drug(
            "Rocuronium injection",
            "rocuronium",
            "neuromuscular_blocker",
            "IV",
            HadSeverity::Critical,
            &["train-of-four", "ventilation", "sedation"],
            "Sugammadex or neostigmine",
            true,
            "Only for trained airway teams",
        ),
        drug(
            "Vecuronium injection",
            "vecuronium",
            "neuromuscular_blocker",
            "IV",
            HadSeverity::Critical,
            &["train-of-four", "ventilation", "sedation"],
            "Neostigmine",
            true,
            "Only for trained airway teams",
        ),
        drug(
            "Norepinephrine infusion",
            "norepinephrine",
            "vasoactive",
            "IV infusion",
            HadSeverity::Critical,
            &["blood pressure", "heart rate", "infusion site"],
            "Fluid resuscitation, vasopressin",
            true,
            "Never give as bolus; use pump and central access when possible",
        ),
        drug(
            "Epinephrine injection",
            "epinephrine",
            "vasoactive",
            "IV or IM",
            HadSeverity::Critical,
            &["blood pressure", "heart rate", "rhythm"],
            "Supportive resuscitation",
            true,
            "Verify concentration and route before administration",
        ),
        drug(
            "Amiodarone injection",
            "amiodarone",
            "antiarrhythmic",
            "IV",
            HadSeverity::High,
            &["blood pressure", "heart rate", "ECG", "infusion site"],
            "Supportive care",
            true,
            "Use pump and maximum rate limit",
        ),
        drug(
            "Digoxin",
            "digoxin",
            "inotropic",
            "oral or IV",
            HadSeverity::High,
            &["digoxin level", "potassium", "renal function", "heart rate"],
            "Digoxin immune Fab",
            false,
            "Hold if bradycardia or toxicity",
        ),
        drug(
            "Vancomycin",
            "vancomycin",
            "antibiotic",
            "IV",
            HadSeverity::High,
            &["trough level", "renal function", "infusion site"],
            "Supportive care",
            false,
            "Hold if trough is high or renal function declines",
        ),
        drug(
            "Methotrexate",
            "methotrexate",
            "chemotherapy",
            "oral or IV",
            HadSeverity::Critical,
            &["CBC", "creatinine", "liver function", "mucosal toxicity"],
            "Leucovorin",
            true,
            "Follow protocol-specific hold criteria",
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn rule(
    name: &str,
    drug_class: &str,
    lab_code: &str,
    lab_name: &str,
    operator: AlertOperator,
    low: f64,
    high: Option<f64>,
    unit: &str,
    severity: HadSeverity,
    action: &str,
) -> AlertRule {
    AlertRule {
        id: new_id(),
        name: name.to_string(),
        description: "Starter template. Map the laboratory item and activate after local approval."
            .to_string(),
        drug_scope_type: "drug_class".to_string(),
        drug_scope_value: Some(drug_class.to_string()),
        lab_items_code: lab_code.to_string(),
        lab_items_name: Some(lab_name.to_string()),
        operator,
        threshold_low: Some(low),
        threshold_high: high,
        unit: Some(unit.to_string()),
        lookback_hours: 72,
        severity,
        action_text: action.to_string(),
        require_active_order: true,
        active: false,
        version: 1,
        created_by: None,
        approved_by: None,
        approved_at: None,
        effective_at: None,
        review_due_at: None,
    }
}

fn default_rules() -> Vec<AlertRule> {
    vec![
        rule(
            "Warfarin INR above 3.5",
            "anticoagulant_warfarin",
            "INR",
            "INR",
            AlertOperator::Gt,
            3.5,
            None,
            "ratio",
            HadSeverity::High,
            "Hold or reduce warfarin, repeat INR, assess bleeding.",
        ),
        rule(
            "Warfarin INR above 5.0",
            "anticoagulant_warfarin",
            "INR",
            "INR",
            AlertOperator::Gt,
            5.0,
            None,
            "ratio",
            HadSeverity::Critical,
            "Hold warfarin, consider vitamin K, notify physician urgently.",
        ),
        rule(
            "Heparin aPTT above 100 seconds",
            "anticoagulant_heparin",
            "APTT",
            "aPTT",
            AlertOperator::Gt,
            100.0,
            None,
            "s",
            HadSeverity::High,
            "Review infusion rate, repeat aPTT, assess bleeding.",
        ),
        rule(
            "Insulin glucose below 70 mg/dL",
            "insulin",
            "GLU",
            "Glucose",
            AlertOperator::Lt,
            70.0,
            None,
            "mg/dL",
            HadSeverity::High,
            "Treat hypoglycemia and review the insulin order.",
        ),
        rule(
            "Potassium chloride with potassium above 5.5",
            "electrolyte_potassium",
            "K",
            "Potassium",
            AlertOperator::Gt,
            5.5,
            None,
            "mmol/L",
            HadSeverity::High,
            "Hold potassium, repeat level, review ECG.",
        ),
        rule(
            "Opioid with respiratory rate below 10",
            "opioid",
            "RR",
            "Respiratory rate",
            AlertOperator::Lt,
            10.0,
            None,
            "/min",
            HadSeverity::Critical,
            "Stop opioid, assess sedation, consider naloxone.",
        ),
        rule(
            "Vancomycin trough above 20",
            "antibiotic",
            "VANCO",
            "Vancomycin trough",
            AlertOperator::Gt,
            20.0,
            None,
            "mg/L",
            HadSeverity::Medium,
            "Review dose and interval, assess nephrotoxicity.",
        ),
        rule(
            "Digoxin level above 2.0",
            "inotropic",
            "DIG",
            "Digoxin level",
            AlertOperator::Gt,
            2.0,
            None,
            "ng/mL",
            HadSeverity::High,
            "Assess toxicity, review dose and renal function.",
        ),
    ]
}

async fn seed_standards(store: &Store) -> Result<()> {
    let refs = [
        (
            "HA6-MED",
            "HA Edition 6",
            "Medication management",
            "Medication management covers selection, prescribing, preparation, dispensing, administration, monitoring, and improvement.",
        ),
        (
            "3P-PATIENT",
            "3P Safety",
            "Patient Safety",
            "Prevent preventable harm to patients from high-alert medication use.",
        ),
        (
            "3P-PERSONNEL",
            "3P Safety",
            "Personnel Safety",
            "Protect staff from hazardous drug exposure, needlestick injury, and unsafe workload.",
        ),
        (
            "3P-PEOPLE",
            "3P Safety",
            "People Safety",
            "Extend medication safety to families, communities, and emergency preparedness.",
        ),
        (
            "NRLS",
            "NRLS",
            "Incident reporting and learning",
            "Report, analyze, and learn from incidents and near misses.",
        ),
        (
            "THIP",
            "THIP",
            "Hospital indicators",
            "Use defined numerators, denominators, and outcome trends.",
        ),
        (
            "PEP",
            "PEP",
            "Patient experience",
            "Measure and improve the patient experience of high-alert medication care.",
        ),
        (
            "SAFETY-CULTURE",
            "Safety Culture Survey",
            "Safety culture",
            "Assess teamwork, communication, staffing, and safety climate.",
        ),
    ];
    for (code, framework, title, requirement) in refs {
        sqlx::query(
            r#"
INSERT OR IGNORE INTO standard_ref (id, framework, code, title, requirement, edition, active)
VALUES (?, ?, ?, ?, ?, ?, 1)
"#,
        )
        .bind(format!("std-{code}"))
        .bind(framework)
        .bind(code)
        .bind(title)
        .bind(requirement)
        .bind("2026")
        .execute(&store.pool)
        .await?;
    }
    Ok(())
}

async fn seed_kpis(store: &Store) -> Result<()> {
    let kpis = [
        (
            "HAD-COVERAGE",
            "HAD catalog coverage",
            "structure",
            "Mapped HAD drugs",
            "All active HAD drugs",
            "App store",
            "monthly",
        ),
        (
            "HAD-DOUBLE-CHECK",
            "Independent double check compliance",
            "process",
            "HAD administrations with documented double check",
            "All HAD administrations",
            "Tracer worksheet",
            "monthly",
        ),
        (
            "HAD-MONITORING",
            "Required monitoring completed on time",
            "process",
            "HAD orders with on-time monitoring",
            "All HAD orders requiring monitoring",
            "HOSxP labs and app store",
            "monthly",
        ),
        (
            "HAD-ADE",
            "Preventable HAD adverse drug events",
            "outcome",
            "Preventable HAD ADEs",
            "1,000 HAD orders",
            "NRLS and incident review",
            "quarterly",
        ),
        (
            "HAD-NEAR-MISS",
            "HAD near-miss reporting rate",
            "outcome",
            "HAD near misses reported",
            "1,000 HAD orders",
            "NRLS",
            "quarterly",
        ),
        (
            "CAPA-CLOSURE",
            "CAPA closed on time",
            "process",
            "CAPA closed by due date",
            "All CAPA due in period",
            "App store",
            "monthly",
        ),
    ];
    for (code, name, domain, numerator, denominator, source, frequency) in kpis {
        sqlx::query(
            r#"
INSERT OR IGNORE INTO kpi_definition (
    id, code, name, domain, numerator, denominator, data_source, frequency, target, owner, active
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, 1)
"#,
        )
        .bind(format!("kpi-{code}"))
        .bind(code)
        .bind(name)
        .bind(domain)
        .bind(numerator)
        .bind(denominator)
        .bind(source)
        .bind(frequency)
        .execute(&store.pool)
        .await?;
    }
    Ok(())
}
