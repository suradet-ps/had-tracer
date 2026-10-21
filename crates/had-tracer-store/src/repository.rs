use had_tracer_core::{
    AlertEvent, AlertOperator, AlertRule, AlertStatus, Capa, CapaStatus, CaseType, CauseCategory,
    Finding, FindingSeverity, FindingStatus, HadDrug, HadSeverity, PhaseCode, SafetyBundle,
    TracerCase, TracerCheck, TracerPhase, TracerStatus, TracerWorkspace, new_id, now_rfc3339,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::Store;
use crate::error::{Result, StoreError};

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

impl Store {
    // ------------------------------------------------------------------
    // HAD catalog
    // ------------------------------------------------------------------

    pub async fn list_had_drugs(&self) -> Result<Vec<HadDrug>> {
        let rows = sqlx::query_as::<_, HadDrugRow>(
            r#"
SELECT id, icode, name, generic_name, drug_class, route, severity,
       safety_bundle_json, active, version, approved_by, approved_at,
       effective_at, review_due_at
FROM had_drug
ORDER BY drug_class ASC, name ASC
"#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn get_had_drug(&self, id: &str) -> Result<HadDrug> {
        let row = sqlx::query_as::<_, HadDrugRow>(
            r#"
SELECT id, icode, name, generic_name, drug_class, route, severity,
       safety_bundle_json, active, version, approved_by, approved_at,
       effective_at, review_due_at
FROM had_drug WHERE id = ?
"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("had_drug {id}")))?;
        row.try_into()
    }

    pub async fn upsert_had_drug(&self, drug: &HadDrug) -> Result<()> {
        let bundle = serde_json::to_string(&drug.safety_bundle)?;
        sqlx::query(
            r#"
INSERT INTO had_drug (
    id, icode, name, generic_name, drug_class, route, severity,
    safety_bundle_json, active, version, approved_by, approved_at,
    effective_at, review_due_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(id) DO UPDATE SET
    icode = excluded.icode,
    name = excluded.name,
    generic_name = excluded.generic_name,
    drug_class = excluded.drug_class,
    route = excluded.route,
    severity = excluded.severity,
    safety_bundle_json = excluded.safety_bundle_json,
    active = excluded.active,
    version = excluded.version,
    approved_by = excluded.approved_by,
    approved_at = excluded.approved_at,
    effective_at = excluded.effective_at,
    review_due_at = excluded.review_due_at
"#,
        )
        .bind(&drug.id)
        .bind(&drug.icode)
        .bind(&drug.name)
        .bind(&drug.generic_name)
        .bind(&drug.drug_class)
        .bind(&drug.route)
        .bind(drug.severity.as_str())
        .bind(bundle)
        .bind(b2i(drug.active))
        .bind(drug.version)
        .bind(&drug.approved_by)
        .bind(&drug.approved_at)
        .bind(&drug.effective_at)
        .bind(&drug.review_due_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn map_had_drug(&self, id: &str, icode: &str) -> Result<()> {
        let result =
            sqlx::query("UPDATE had_drug SET icode = ?, version = version + 1 WHERE id = ?")
                .bind(icode.trim())
                .bind(id)
                .execute(&self.pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(format!("had_drug {id}")));
        }
        Ok(())
    }

    pub async fn set_had_drug_active(&self, id: &str, active: bool) -> Result<()> {
        sqlx::query("UPDATE had_drug SET active = ? WHERE id = ?")
            .bind(b2i(active))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn deactivate_had_drug(&self, id: &str) -> Result<()> {
        self.set_had_drug_active(id, false).await
    }

    // ------------------------------------------------------------------
    // Alert rules and events
    // ------------------------------------------------------------------

    pub async fn list_alert_rules(&self) -> Result<Vec<AlertRule>> {
        let rows = sqlx::query_as::<_, AlertRuleRow>(
            r#"
SELECT id, name, description, drug_scope_type, drug_scope_value,
       lab_items_code, lab_items_name, operator, threshold_low, threshold_high,
       unit, lookback_hours, severity, action_text, require_active_order,
       active, version, created_by, approved_by, approved_at, effective_at,
       review_due_at
FROM alert_rule
ORDER BY active DESC, severity ASC, name ASC
"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_alert_rule(&self, rule: &AlertRule) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO alert_rule (
    id, name, description, drug_scope_type, drug_scope_value,
    lab_items_code, lab_items_name, operator, threshold_low, threshold_high,
    unit, lookback_hours, severity, action_text, require_active_order,
    active, version, created_by, approved_by, approved_at, effective_at,
    review_due_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(id) DO UPDATE SET
    name = excluded.name,
    description = excluded.description,
    drug_scope_type = excluded.drug_scope_type,
    drug_scope_value = excluded.drug_scope_value,
    lab_items_code = excluded.lab_items_code,
    lab_items_name = excluded.lab_items_name,
    operator = excluded.operator,
    threshold_low = excluded.threshold_low,
    threshold_high = excluded.threshold_high,
    unit = excluded.unit,
    lookback_hours = excluded.lookback_hours,
    severity = excluded.severity,
    action_text = excluded.action_text,
    require_active_order = excluded.require_active_order,
    active = excluded.active,
    version = excluded.version,
    created_by = excluded.created_by,
    approved_by = excluded.approved_by,
    approved_at = excluded.approved_at,
    effective_at = excluded.effective_at,
    review_due_at = excluded.review_due_at
"#,
        )
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.drug_scope_type)
        .bind(&rule.drug_scope_value)
        .bind(&rule.lab_items_code)
        .bind(&rule.lab_items_name)
        .bind(rule.operator.as_str())
        .bind(rule.threshold_low)
        .bind(rule.threshold_high)
        .bind(&rule.unit)
        .bind(rule.lookback_hours)
        .bind(rule.severity.as_str())
        .bind(&rule.action_text)
        .bind(b2i(rule.require_active_order))
        .bind(b2i(rule.active))
        .bind(rule.version)
        .bind(&rule.created_by)
        .bind(&rule.approved_by)
        .bind(&rule.approved_at)
        .bind(&rule.effective_at)
        .bind(&rule.review_due_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_alert_rule_active(&self, id: &str, active: bool) -> Result<()> {
        sqlx::query("UPDATE alert_rule SET active = ? WHERE id = ?")
            .bind(b2i(active))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_alert_event(&self, event: &AlertEvent) -> Result<bool> {
        let dedupe_key = format!(
            "{}|{}|{}|{}",
            event.hn,
            event.rule_id,
            event.drug_icode.as_deref().unwrap_or("-"),
            event.lab_order_number.as_deref().unwrap_or("-")
        );
        let result = sqlx::query(
            r#"
INSERT OR IGNORE INTO alert_event (
    id, dedupe_key, rule_id, rule_name, hn, an, vn, drug_icode, drug_name,
    lab_order_number, lab_items_code, lab_items_name, raw_value, numeric_value,
    comparator, unit, observed_at, severity, status, action_text,
    acknowledged_by, acknowledged_at, override_reason, resolved_by, resolved_at,
    resolve_note, created_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&event.id)
        .bind(dedupe_key)
        .bind(&event.rule_id)
        .bind(&event.rule_name)
        .bind(&event.hn)
        .bind(&event.an)
        .bind(&event.vn)
        .bind(&event.drug_icode)
        .bind(&event.drug_name)
        .bind(&event.lab_order_number)
        .bind(&event.lab_items_code)
        .bind(&event.lab_items_name)
        .bind(&event.raw_value)
        .bind(event.numeric_value)
        .bind(&event.comparator)
        .bind(&event.unit)
        .bind(&event.observed_at)
        .bind(event.severity.as_str())
        .bind(event.status.as_str())
        .bind(&event.action_text)
        .bind(&event.acknowledged_by)
        .bind(&event.acknowledged_at)
        .bind(&event.override_reason)
        .bind(&event.resolved_by)
        .bind(&event.resolved_at)
        .bind(&event.resolve_note)
        .bind(&event.created_at)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_alert_events(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> Result<Vec<AlertEvent>> {
        let rows = sqlx::query_as::<_, AlertEventRow>(
            r#"
SELECT id, rule_id, rule_name, hn, an, vn, drug_icode, drug_name,
       lab_order_number, lab_items_code, lab_items_name, raw_value,
       numeric_value, comparator, unit, observed_at, severity, status,
       action_text, acknowledged_by, acknowledged_at, override_reason,
       resolved_by, resolved_at, resolve_note, created_at
FROM alert_event
WHERE (? IS NULL OR status = ?)
ORDER BY CASE severity
    WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2
    WHEN 'low' THEN 3 ELSE 4 END,
    created_at DESC
LIMIT ?
"#,
        )
        .bind(status)
        .bind(status)
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn alert_counts(&self) -> Result<AlertCounts> {
        let new = count_where(&self.pool, "alert_event", "status = 'new'").await?;
        let acknowledged =
            count_where(&self.pool, "alert_event", "status = 'acknowledged'").await?;
        let overridden = count_where(&self.pool, "alert_event", "status = 'overridden'").await?;
        let resolved = count_where(&self.pool, "alert_event", "status = 'resolved'").await?;
        let critical = count_where(
            &self.pool,
            "alert_event",
            "severity = 'critical' AND status = 'new'",
        )
        .await?;
        Ok(AlertCounts {
            new,
            acknowledged,
            overridden,
            resolved,
            critical,
        })
    }

    pub async fn acknowledge_alert(&self, id: &str, actor: &str) -> Result<()> {
        let now = now_rfc3339();
        sqlx::query(
            "UPDATE alert_event SET status = 'acknowledged', acknowledged_by = ?, acknowledged_at = ? WHERE id = ?",
        )
        .bind(actor)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn override_alert(&self, id: &str, actor: &str, reason: &str) -> Result<()> {
        if reason.trim().is_empty() {
            return Err(StoreError::Invalid(
                "override reason must not be empty".to_string(),
            ));
        }
        let now = now_rfc3339();
        sqlx::query(
            "UPDATE alert_event SET status = 'overridden', acknowledged_by = ?, acknowledged_at = ?, override_reason = ? WHERE id = ?",
        )
        .bind(actor)
        .bind(now)
        .bind(reason.trim())
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn resolve_alert(&self, id: &str, actor: &str, note: &str) -> Result<()> {
        let now = now_rfc3339();
        sqlx::query(
            "UPDATE alert_event SET status = 'resolved', resolved_by = ?, resolved_at = ?, resolve_note = ? WHERE id = ?",
        )
        .bind(actor)
        .bind(now)
        .bind(note.trim())
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Tracer cases
    // ------------------------------------------------------------------

    pub async fn create_tracer_case(&self, case: &TracerCase) -> Result<TracerWorkspace> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
INSERT INTO tracer_case (
    id, case_type, title, purpose, hn, an, vn, trigger_type, trigger_ref,
    lead_user, status, opened_at, closed_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&case.id)
        .bind(case.case_type.as_str())
        .bind(&case.title)
        .bind(&case.purpose)
        .bind(&case.hn)
        .bind(&case.an)
        .bind(&case.vn)
        .bind(&case.trigger_type)
        .bind(&case.trigger_ref)
        .bind(&case.lead_user)
        .bind(case.status.as_str())
        .bind(&case.opened_at)
        .bind(&case.closed_at)
        .execute(&mut *tx)
        .await?;

        for phase in PhaseCode::ALL {
            let phase_id = new_id();
            sqlx::query(
                "INSERT INTO tracer_phase (id, case_id, phase_code, status, summary) VALUES (?, ?, ?, 'pending', '')",
            )
            .bind(&phase_id)
            .bind(&case.id)
            .bind(phase.as_str())
            .execute(&mut *tx)
            .await?;
            for question in phase.questions() {
                sqlx::query(
                    r#"
INSERT INTO tracer_check (
    id, case_id, phase_id, standard_code, question, expected, observed,
    gap, evidence_summary, finding_id
) VALUES (?, ?, ?, NULL, ?, '', '', '', '', NULL)
"#,
                )
                .bind(new_id())
                .bind(&case.id)
                .bind(&phase_id)
                .bind(*question)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        self.get_tracer_workspace(&case.id).await
    }

    pub async fn list_tracer_cases(&self, limit: u32) -> Result<Vec<TracerCase>> {
        let rows = sqlx::query_as::<_, TracerCaseRow>(
            r#"
SELECT id, case_type, title, purpose, hn, an, vn, trigger_type, trigger_ref,
       lead_user, status, opened_at, closed_at
FROM tracer_case
ORDER BY opened_at DESC
LIMIT ?
"#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_tracer_workspace(&self, id: &str) -> Result<TracerWorkspace> {
        let case = sqlx::query_as::<_, TracerCaseRow>(
            r#"
SELECT id, case_type, title, purpose, hn, an, vn, trigger_type, trigger_ref,
       lead_user, status, opened_at, closed_at
FROM tracer_case WHERE id = ?
"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("tracer_case {id}")))?;

        let phases = sqlx::query_as::<_, TracerPhaseRow>(
            r#"
SELECT id, case_id, phase_code, status, summary
FROM tracer_phase
WHERE case_id = ?
ORDER BY CASE phase_code
    WHEN 'pre_encounter' THEN 0 WHEN 'prescribing' THEN 1
    WHEN 'verification' THEN 2 WHEN 'preparation' THEN 3
    WHEN 'administration' THEN 4 WHEN 'monitoring' THEN 5
    WHEN 'handoff' THEN 6 ELSE 7 END
"#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let checks = sqlx::query_as::<_, TracerCheckRow>(
            r#"
SELECT id, case_id, phase_id, standard_code, question, expected, observed,
       gap, evidence_summary, finding_id
FROM tracer_check
WHERE case_id = ?
ORDER BY rowid ASC
"#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let findings = sqlx::query_as::<_, FindingRow>(
            r#"
SELECT id, case_id, phase_id, title, description, standard_code, severity,
       likelihood, risk_score, cause_category, immediate_action,
       recommended_action, status, created_at
FROM finding
WHERE case_id = ?
ORDER BY created_at DESC
"#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let capas = sqlx::query_as::<_, CapaRow>(
            r#"
SELECT c.id, c.finding_id, c.action, c.owner, c.due_date, c.status,
       c.verification_method, c.verification_date, c.effectiveness_result,
       c.evidence, c.created_at, c.closed_at
FROM capa c
JOIN finding f ON f.id = c.finding_id
WHERE f.case_id = ?
ORDER BY c.created_at DESC
"#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        Ok(TracerWorkspace {
            case: case.into(),
            phases: phases.into_iter().map(Into::into).collect(),
            checks: checks.into_iter().map(Into::into).collect(),
            findings: findings.into_iter().map(Into::into).collect(),
            capas: capas.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn update_tracer_phase(
        &self,
        phase_id: &str,
        status: &str,
        summary: &str,
    ) -> Result<()> {
        let result = sqlx::query("UPDATE tracer_phase SET status = ?, summary = ? WHERE id = ?")
            .bind(status)
            .bind(summary)
            .bind(phase_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(format!("tracer_phase {phase_id}")));
        }
        Ok(())
    }

    pub async fn update_tracer_check(&self, check: &TracerCheck) -> Result<()> {
        let result = sqlx::query(
            r#"
UPDATE tracer_check
SET standard_code = ?, expected = ?, observed = ?, gap = ?,
    evidence_summary = ?, finding_id = ?
WHERE id = ?
"#,
        )
        .bind(&check.standard_code)
        .bind(&check.expected)
        .bind(&check.observed)
        .bind(&check.gap)
        .bind(&check.evidence_summary)
        .bind(&check.finding_id)
        .bind(&check.id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(format!("tracer_check {}", check.id)));
        }
        Ok(())
    }

    pub async fn close_tracer_case(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE tracer_case SET status = 'closed', closed_at = ? WHERE id = ?")
            .bind(now_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Findings and CAPA
    // ------------------------------------------------------------------

    pub async fn create_finding(&self, finding: &Finding) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO finding (
    id, case_id, phase_id, title, description, standard_code, severity,
    likelihood, risk_score, cause_category, immediate_action,
    recommended_action, status, created_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&finding.id)
        .bind(&finding.case_id)
        .bind(&finding.phase_id)
        .bind(&finding.title)
        .bind(&finding.description)
        .bind(&finding.standard_code)
        .bind(finding.severity.as_str())
        .bind(finding.likelihood)
        .bind(finding.risk_score)
        .bind(finding.cause_category.as_str())
        .bind(&finding.immediate_action)
        .bind(&finding.recommended_action)
        .bind(finding.status.as_str())
        .bind(&finding.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_findings(&self, case_id: Option<&str>) -> Result<Vec<Finding>> {
        let rows = sqlx::query_as::<_, FindingRow>(
            r#"
SELECT id, case_id, phase_id, title, description, standard_code, severity,
       likelihood, risk_score, cause_category, immediate_action,
       recommended_action, status, created_at
FROM finding
WHERE (? IS NULL OR case_id = ?)
ORDER BY created_at DESC
"#,
        )
        .bind(case_id)
        .bind(case_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn update_finding_status(&self, id: &str, status: &str) -> Result<()> {
        sqlx::query("UPDATE finding SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_capa(&self, capa: &Capa) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO capa (
    id, finding_id, action, owner, due_date, status, verification_method,
    verification_date, effectiveness_result, evidence, created_at, closed_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&capa.id)
        .bind(&capa.finding_id)
        .bind(&capa.action)
        .bind(&capa.owner)
        .bind(&capa.due_date)
        .bind(capa.status.as_str())
        .bind(&capa.verification_method)
        .bind(&capa.verification_date)
        .bind(&capa.effectiveness_result)
        .bind(&capa.evidence)
        .bind(&capa.created_at)
        .bind(&capa.closed_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_capas(&self, finding_id: Option<&str>) -> Result<Vec<Capa>> {
        let rows = sqlx::query_as::<_, CapaRow>(
            r#"
SELECT id, finding_id, action, owner, due_date, status, verification_method,
       verification_date, effectiveness_result, evidence, created_at, closed_at
FROM capa
WHERE (? IS NULL OR finding_id = ?)
ORDER BY created_at DESC
"#,
        )
        .bind(finding_id)
        .bind(finding_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn update_capa_status(
        &self,
        id: &str,
        status: &str,
        effectiveness_result: &str,
    ) -> Result<()> {
        let closed_at = if status == "closed" || status == "verified" {
            Some(now_rfc3339())
        } else {
            None
        };
        sqlx::query(
            "UPDATE capa SET status = ?, effectiveness_result = ?, closed_at = ? WHERE id = ?",
        )
        .bind(status)
        .bind(effectiveness_result)
        .bind(closed_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Dashboard, references, audit
    // ------------------------------------------------------------------

    pub async fn dashboard_metrics(&self) -> Result<DashboardMetrics> {
        Ok(DashboardMetrics {
            had_total: count(&self.pool, "had_drug").await?,
            had_mapped: count_where(&self.pool, "had_drug", "icode IS NOT NULL AND icode <> ''")
                .await?,
            active_rules: count_where(&self.pool, "alert_rule", "active = 1").await?,
            open_alerts: count_where(&self.pool, "alert_event", "status = 'new'").await?,
            critical_alerts: count_where(
                &self.pool,
                "alert_event",
                "severity = 'critical' AND status = 'new'",
            )
            .await?,
            tracer_open: count_where(&self.pool, "tracer_case", "status <> 'closed'").await?,
            findings_open: count_where(
                &self.pool,
                "finding",
                "status NOT IN ('closed','verified')",
            )
            .await?,
            capa_overdue: count_where(
                &self.pool,
                "capa",
                "due_date IS NOT NULL AND date(due_date) < date('now') AND status NOT IN ('closed','verified')",
            )
            .await?,
            audit_today: count_where(&self.pool, "audit_log", "date(created_at) = date('now')")
                .await?,
        })
    }

    pub async fn list_standard_refs(&self) -> Result<Vec<StandardRef>> {
        let rows = sqlx::query_as::<_, StandardRefRow>(
            r#"
SELECT id, framework, code, title, requirement, edition, active
FROM standard_ref WHERE active = 1 ORDER BY framework, code
"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn list_kpi_definitions(&self) -> Result<Vec<KpiDefinition>> {
        let rows = sqlx::query_as::<_, KpiDefinitionRow>(
            r#"
SELECT id, code, name, domain, numerator, denominator, data_source,
       frequency, target, owner, active
FROM kpi_definition WHERE active = 1 ORDER BY domain, code
"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn audit(
        &self,
        actor: &str,
        action: &str,
        entity_type: &str,
        entity_id: Option<&str>,
        detail: serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO audit_log (id, actor, action, entity_type, entity_id, detail_json, created_at)
VALUES (?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(new_id())
        .bind(actor)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(detail.to_string())
        .bind(now_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_audit(&self, limit: u32) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query_as::<_, AuditRow>(
            r#"
SELECT id, actor, action, entity_type, entity_id, detail_json, created_at
FROM audit_log ORDER BY created_at DESC LIMIT ?
"#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn table_counts(&self) -> Result<Vec<(String, i64)>> {
        let mut counts = Vec::new();
        for table in crate::schema::EXPORT_TABLES {
            counts.push(((*table).to_string(), count(&self.pool, table).await?));
        }
        Ok(counts)
    }
}

fn b2i(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

async fn count(pool: &sqlx::SqlitePool, table: &str) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) AS count FROM {table}");
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await?;
    Ok(row.try_get("count")?)
}

async fn count_where(pool: &sqlx::SqlitePool, table: &str, predicate: &str) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) AS count FROM {table} WHERE {predicate}");
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await?;
    Ok(row.try_get("count")?)
}

// ----------------------------------------------------------------------
// Row types
// ----------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct HadDrugRow {
    id: String,
    icode: Option<String>,
    name: String,
    generic_name: Option<String>,
    drug_class: String,
    route: Option<String>,
    severity: String,
    safety_bundle_json: String,
    active: i64,
    version: i64,
    approved_by: Option<String>,
    approved_at: Option<String>,
    effective_at: Option<String>,
    review_due_at: Option<String>,
}

impl TryFrom<HadDrugRow> for HadDrug {
    type Error = StoreError;

    fn try_from(row: HadDrugRow) -> Result<Self> {
        let safety_bundle: SafetyBundle = serde_json::from_str(&row.safety_bundle_json)
            .map_err(|e| StoreError::Invalid(format!("safety bundle JSON: {e}")))?;
        Ok(Self {
            id: row.id,
            icode: row.icode,
            name: row.name,
            generic_name: row.generic_name,
            drug_class: row.drug_class,
            route: row.route,
            severity: HadSeverity::from_str(&row.severity),
            safety_bundle,
            active: row.active != 0,
            version: row.version,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            effective_at: row.effective_at,
            review_due_at: row.review_due_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct AlertRuleRow {
    id: String,
    name: String,
    description: String,
    drug_scope_type: String,
    drug_scope_value: Option<String>,
    lab_items_code: String,
    lab_items_name: Option<String>,
    operator: String,
    threshold_low: Option<f64>,
    threshold_high: Option<f64>,
    unit: Option<String>,
    lookback_hours: i64,
    severity: String,
    action_text: String,
    require_active_order: i64,
    active: i64,
    version: i64,
    created_by: Option<String>,
    approved_by: Option<String>,
    approved_at: Option<String>,
    effective_at: Option<String>,
    review_due_at: Option<String>,
}

impl From<AlertRuleRow> for AlertRule {
    fn from(row: AlertRuleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            drug_scope_type: row.drug_scope_type,
            drug_scope_value: row.drug_scope_value,
            lab_items_code: row.lab_items_code,
            lab_items_name: row.lab_items_name,
            operator: AlertOperator::from_str(&row.operator),
            threshold_low: row.threshold_low,
            threshold_high: row.threshold_high,
            unit: row.unit,
            lookback_hours: row.lookback_hours,
            severity: HadSeverity::from_str(&row.severity),
            action_text: row.action_text,
            require_active_order: row.require_active_order != 0,
            active: row.active != 0,
            version: row.version,
            created_by: row.created_by,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            effective_at: row.effective_at,
            review_due_at: row.review_due_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AlertEventRow {
    id: String,
    rule_id: String,
    rule_name: String,
    hn: String,
    an: Option<String>,
    vn: Option<String>,
    drug_icode: Option<String>,
    drug_name: Option<String>,
    lab_order_number: Option<String>,
    lab_items_code: String,
    lab_items_name: Option<String>,
    raw_value: String,
    numeric_value: Option<f64>,
    comparator: Option<String>,
    unit: Option<String>,
    observed_at: Option<String>,
    severity: String,
    status: String,
    action_text: String,
    acknowledged_by: Option<String>,
    acknowledged_at: Option<String>,
    override_reason: Option<String>,
    resolved_by: Option<String>,
    resolved_at: Option<String>,
    resolve_note: Option<String>,
    created_at: String,
}

impl From<AlertEventRow> for AlertEvent {
    fn from(row: AlertEventRow) -> Self {
        Self {
            id: row.id,
            rule_id: row.rule_id,
            rule_name: row.rule_name,
            hn: row.hn,
            an: row.an,
            vn: row.vn,
            drug_icode: row.drug_icode,
            drug_name: row.drug_name,
            lab_order_number: row.lab_order_number,
            lab_items_code: row.lab_items_code,
            lab_items_name: row.lab_items_name,
            raw_value: row.raw_value,
            numeric_value: row.numeric_value,
            comparator: row.comparator,
            unit: row.unit,
            observed_at: row.observed_at,
            severity: HadSeverity::from_str(&row.severity),
            status: AlertStatus::from_str(&row.status),
            action_text: row.action_text,
            acknowledged_by: row.acknowledged_by,
            acknowledged_at: row.acknowledged_at,
            override_reason: row.override_reason,
            resolved_by: row.resolved_by,
            resolved_at: row.resolved_at,
            resolve_note: row.resolve_note,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TracerCaseRow {
    id: String,
    case_type: String,
    title: String,
    purpose: String,
    hn: String,
    an: Option<String>,
    vn: Option<String>,
    trigger_type: Option<String>,
    trigger_ref: Option<String>,
    lead_user: Option<String>,
    status: String,
    opened_at: String,
    closed_at: Option<String>,
}

impl From<TracerCaseRow> for TracerCase {
    fn from(row: TracerCaseRow) -> Self {
        Self {
            id: row.id,
            case_type: CaseType::from_str(&row.case_type),
            title: row.title,
            purpose: row.purpose,
            hn: row.hn,
            an: row.an,
            vn: row.vn,
            trigger_type: row.trigger_type,
            trigger_ref: row.trigger_ref,
            lead_user: row.lead_user,
            status: TracerStatus::from_str(&row.status),
            opened_at: row.opened_at,
            closed_at: row.closed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TracerPhaseRow {
    id: String,
    case_id: String,
    phase_code: String,
    status: String,
    summary: String,
}

impl From<TracerPhaseRow> for TracerPhase {
    fn from(row: TracerPhaseRow) -> Self {
        Self {
            id: row.id,
            case_id: row.case_id,
            phase_code: PhaseCode::from_str(&row.phase_code),
            status: row.status,
            summary: row.summary,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TracerCheckRow {
    id: String,
    case_id: String,
    phase_id: String,
    standard_code: Option<String>,
    question: String,
    expected: String,
    observed: String,
    gap: String,
    evidence_summary: String,
    finding_id: Option<String>,
}

impl From<TracerCheckRow> for TracerCheck {
    fn from(row: TracerCheckRow) -> Self {
        Self {
            id: row.id,
            case_id: row.case_id,
            phase_id: row.phase_id,
            standard_code: row.standard_code,
            question: row.question,
            expected: row.expected,
            observed: row.observed,
            gap: row.gap,
            evidence_summary: row.evidence_summary,
            finding_id: row.finding_id,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FindingRow {
    id: String,
    case_id: String,
    phase_id: Option<String>,
    title: String,
    description: String,
    standard_code: Option<String>,
    severity: String,
    likelihood: i64,
    risk_score: i64,
    cause_category: String,
    immediate_action: String,
    recommended_action: String,
    status: String,
    created_at: String,
}

impl From<FindingRow> for Finding {
    fn from(row: FindingRow) -> Self {
        Self {
            id: row.id,
            case_id: row.case_id,
            phase_id: row.phase_id,
            title: row.title,
            description: row.description,
            standard_code: row.standard_code,
            severity: FindingSeverity::from_str(&row.severity),
            likelihood: row.likelihood,
            risk_score: row.risk_score,
            cause_category: CauseCategory::from_str(&row.cause_category),
            immediate_action: row.immediate_action,
            recommended_action: row.recommended_action,
            status: FindingStatus::from_str(&row.status),
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CapaRow {
    id: String,
    finding_id: String,
    action: String,
    owner: String,
    due_date: Option<String>,
    status: String,
    verification_method: String,
    verification_date: Option<String>,
    effectiveness_result: String,
    evidence: String,
    created_at: String,
    closed_at: Option<String>,
}

impl From<CapaRow> for Capa {
    fn from(row: CapaRow) -> Self {
        Self {
            id: row.id,
            finding_id: row.finding_id,
            action: row.action,
            owner: row.owner,
            due_date: row.due_date,
            status: CapaStatus::from_str(&row.status),
            verification_method: row.verification_method,
            verification_date: row.verification_date,
            effectiveness_result: row.effectiveness_result,
            evidence: row.evidence,
            created_at: row.created_at,
            closed_at: row.closed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct StandardRefRow {
    id: String,
    framework: String,
    code: String,
    title: String,
    requirement: String,
    edition: Option<String>,
    active: i64,
}

impl From<StandardRefRow> for StandardRef {
    fn from(row: StandardRefRow) -> Self {
        Self {
            id: row.id,
            framework: row.framework,
            code: row.code,
            title: row.title,
            requirement: row.requirement,
            edition: row.edition,
            active: row.active != 0,
        }
    }
}

#[derive(sqlx::FromRow)]
struct KpiDefinitionRow {
    id: String,
    code: String,
    name: String,
    domain: String,
    numerator: String,
    denominator: String,
    data_source: String,
    frequency: String,
    target: Option<String>,
    owner: Option<String>,
    active: i64,
}

impl From<KpiDefinitionRow> for KpiDefinition {
    fn from(row: KpiDefinitionRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            name: row.name,
            domain: row.domain,
            numerator: row.numerator,
            denominator: row.denominator,
            data_source: row.data_source,
            frequency: row.frequency,
            target: row.target,
            owner: row.owner,
            active: row.active != 0,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: String,
    actor: String,
    action: String,
    entity_type: String,
    entity_id: Option<String>,
    detail_json: Option<String>,
    created_at: String,
}

impl From<AuditRow> for AuditEntry {
    fn from(row: AuditRow) -> Self {
        Self {
            id: row.id,
            actor: row.actor,
            action: row.action,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            detail_json: row.detail_json,
            created_at: row.created_at,
        }
    }
}
