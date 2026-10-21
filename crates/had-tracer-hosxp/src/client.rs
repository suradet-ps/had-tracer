use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chrono::{Days, Local};
use had_tracer_core::{
    AllergyView, DiagnosisView, HadDrug, HadOrderView, HadSeverity, LabResultView, PatientCockpit,
    PatientSummary, SchemaCapability, SchemaProbeReport, VitalSignView, parse_lab_value,
};
use secrecy::ExposeSecret;
use sqlx::Row;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};

use crate::config::HosxpConfig;
use crate::error::{Error, Result};
use crate::queries::{
    OPTIONAL_SCHEMA, REQUIRED_SCHEMA, SQL_ACTIVE_ADMISSION, SQL_ALLERGIES, SQL_DATABASE_NAME,
    SQL_DIAGNOSES, SQL_DRUGITEMS_SEARCH, SQL_IPD_VITALS, SQL_LAB_ITEMS_SEARCH, SQL_OPD_VITALS,
    SQL_PATIENT_BY_HN, SQL_PATIENT_SEARCH, SQL_SCHEMA_COLUMNS, SQL_SERVER_VERSION, placeholders,
};
use crate::readonly::assert_read_only;

/// A read-only HOSxP client. Cloning is cheap because the pool is shared.
#[derive(Clone)]
pub struct HosxpClient {
    pool: MySqlPool,
}

impl HosxpClient {
    /// Open a read-only pool and verify it with a smoke test.
    pub async fn connect(config: &HosxpConfig) -> Result<Self> {
        let ssl_mode = match config.tls_mode.as_str() {
            "disabled" => MySqlSslMode::Disabled,
            "required" => MySqlSslMode::Required,
            "verify_ca" => MySqlSslMode::VerifyCa,
            "verify_identity" => MySqlSslMode::VerifyIdentity,
            _ => MySqlSslMode::Preferred,
        };
        let options = MySqlConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(config.password.expose_secret())
            .database(&config.database)
            .ssl_mode(ssl_mode);

        let statement_timeout = config.statement_timeout_ms.max(1_000);
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .after_connect(move |conn, _meta| {
                Box::pin(async move {
                    let _ = sqlx::query("SET SESSION TRANSACTION READ ONLY")
                        .execute(&mut *conn)
                        .await;
                    let _ = sqlx::query("SET NAMES utf8mb4").execute(&mut *conn).await;
                    let sql = format!("SET SESSION MAX_EXECUTION_TIME={statement_timeout}");
                    let _ = sqlx::query(sqlx::AssertSqlSafe(sql))
                        .execute(&mut *conn)
                        .await;
                    Ok(())
                })
            })
            .connect_with(options)
            .await
            .map_err(|e| Error::Connect(e.to_string()))?;

        let client = Self { pool };
        client
            .ping()
            .await
            .map_err(|e| Error::Connect(e.to_string()))?;
        Ok(client)
    }

    /// Measure a round trip to the server.
    pub async fn ping(&self) -> Result<u64> {
        assert_read_only("SELECT 1")?;
        let started = Instant::now();
        let row = sqlx::query("SELECT 1 AS ok").fetch_one(&self.pool).await?;
        let _: i32 = row.try_get("ok")?;
        Ok(started.elapsed().as_millis() as u64)
    }

    /// Verify the live schema contract before any feature is used.
    pub async fn schema_probe(&self) -> Result<SchemaProbeReport> {
        assert_read_only(SQL_SCHEMA_COLUMNS)?;
        let version_row = sqlx::query(SQL_SERVER_VERSION)
            .fetch_one(&self.pool)
            .await?;
        let version: String = version_row.try_get("version")?;
        let database_row = sqlx::query(SQL_DATABASE_NAME).fetch_one(&self.pool).await?;
        let database: Option<String> = database_row.try_get::<String, _>("database_name").ok();

        let rows = sqlx::query(SQL_SCHEMA_COLUMNS)
            .fetch_all(&self.pool)
            .await?;
        let mut columns: HashMap<String, HashSet<String>> = HashMap::new();
        for row in rows {
            let table: String = row.try_get("table_name")?;
            let column: String = row.try_get("column_name")?;
            columns.entry(table).or_default().insert(column);
        }

        let mut tables = Vec::with_capacity(REQUIRED_SCHEMA.len());
        for (table, required_columns) in REQUIRED_SCHEMA {
            let present = columns.contains_key(*table);
            let missing_columns = if present {
                required_columns
                    .iter()
                    .filter(|column| !columns[*table].contains(**column))
                    .map(|column| (*column).to_string())
                    .collect::<Vec<_>>()
            } else {
                required_columns
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect()
            };
            tables.push(SchemaCapability {
                table: (*table).to_string(),
                present,
                missing_columns,
            });
        }

        let mut warnings = Vec::new();
        for (table, column) in OPTIONAL_SCHEMA {
            let present = columns.get(*table).is_some_and(|set| set.contains(*column));
            if !present {
                warnings.push(format!("optional column {table}.{column} is absent"));
            }
        }

        let ok = tables
            .iter()
            .all(|table| table.present && table.missing_columns.is_empty());
        Ok(SchemaProbeReport {
            server_version: version,
            database: database.unwrap_or_default(),
            checked_at: had_tracer_core::now_rfc3339(),
            tables,
            ok,
            warnings,
        })
    }

    /// Search patients by HN, first name, or last name.
    pub async fn search_patients(&self, query: &str, limit: u32) -> Result<Vec<PatientSummary>> {
        assert_read_only(SQL_PATIENT_SEARCH)?;
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let like = format!("%{query}%");
        let limit = limit.clamp(1, 100) as i64;
        let rows = sqlx::query_as::<_, PatientRow>(SQL_PATIENT_SEARCH)
            .bind(query)
            .bind(&like)
            .bind(&like)
            .bind(&like)
            .bind(query)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Fetch one patient by exact HN.
    pub async fn patient_by_hn(&self, hn: &str) -> Result<Option<PatientSummary>> {
        assert_read_only(SQL_PATIENT_BY_HN)?;
        let row = sqlx::query_as::<_, PatientRow>(SQL_PATIENT_BY_HN)
            .bind(hn)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    /// Return the open IPD admission, if any.
    pub async fn active_admission(
        &self,
        hn: &str,
    ) -> Result<Option<(String, Option<String>, Option<String>)>> {
        assert_read_only(SQL_ACTIVE_ADMISSION)?;
        let row = sqlx::query_as::<_, AdmissionRow>(SQL_ACTIVE_ADMISSION)
            .bind(hn)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|row| {
            (
                row.an,
                normalize_date_text(row.regdate),
                row.ward_name.or(row.ward),
            )
        }))
    }

    /// Fetch recent HAD order lines from `opitemrece`, filtered to mapped
    /// HAD icodes.
    pub async fn had_orders(
        &self,
        hn: &str,
        had_drugs: &[HadDrug],
        days: u32,
    ) -> Result<Vec<HadOrderView>> {
        let mapped: Vec<&HadDrug> = had_drugs.iter().filter(|drug| drug.is_mapped()).collect();
        if mapped.is_empty() {
            return Ok(Vec::new());
        }
        let by_icode: HashMap<&str, &HadDrug> = mapped
            .iter()
            .filter_map(|drug| drug.icode.as_deref().map(|icode| (icode, *drug)))
            .collect();
        let icodes: Vec<String> = by_icode.keys().map(|icode| (*icode).to_string()).collect();
        let cutoff = Local::now().date_naive() - Days::new(days.max(1) as u64);
        let sql = format!(
            r#"
SELECT
    o.vn,
    o.an,
    o.icode,
    d.name AS drug_name,
    d.strength,
    d.units,
    CAST(o.vstdate AS CHAR) AS vstdate,
    o.qty,
    du.name1 AS sig,
    su.sp_name AS special_use,
    inc.name AS income_name
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
LEFT JOIN drugusage du ON du.drugusage = o.drugusage
LEFT JOIN sp_use su ON su.sp_use = o.sp_use
LEFT JOIN income inc ON inc.income = o.income
WHERE o.hn = ? AND o.icode IN ({})
  AND o.vstdate >= ?
ORDER BY o.vstdate DESC, o.vn DESC
LIMIT 500
"#,
            placeholders(icodes.len())
        );
        assert_read_only(&sql)?;
        let mut query = sqlx::query_as::<_, HadOrderRow>(sqlx::AssertSqlSafe(sql)).bind(hn);
        for icode in &icodes {
            query = query.bind(icode);
        }
        let rows = query.bind(cutoff).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let drug = by_icode.get(row.icode.as_str()).copied();
                HadOrderView {
                    vn: row.vn,
                    an: row.an,
                    icode: row.icode,
                    drug_name: row.drug_name.unwrap_or_default(),
                    strength: row.strength,
                    units: row.units,
                    vstdate: normalize_date_text(row.vstdate),
                    qty: row.qty,
                    sig: row.sig,
                    special_use: row.special_use,
                    income_name: row.income_name,
                    had_drug_id: drug.map(|drug| drug.id.clone()),
                    had_drug_class: drug.map(|drug| drug.drug_class.clone()),
                    severity: drug
                        .map(|drug| drug.severity)
                        .unwrap_or(HadSeverity::Medium),
                }
            })
            .collect())
    }

    /// Latest result per laboratory item for one patient.
    pub async fn latest_labs(&self, hn: &str, codes: &[String]) -> Result<Vec<LabResultView>> {
        let mut unique: Vec<String> = codes
            .iter()
            .map(|code| code.trim().to_string())
            .filter(|code| !code.is_empty())
            .collect();
        unique.sort();
        unique.dedup();
        if unique.is_empty() {
            return Ok(Vec::new());
        }
        let sql = format!(
            r#"
SELECT
    CAST(h.lab_order_number AS CHAR) AS lab_order_number,
    CAST(o.lab_items_code AS CHAR) AS lab_items_code,
    i.lab_items_name,
    i.lab_items_unit,
    CAST(o.lab_order_result AS CHAR) AS lab_order_result,
    CAST(h.order_date AS CHAR) AS order_date,
    CAST(h.report_date AS CHAR) AS report_date
FROM lab_order o
JOIN lab_head h ON h.lab_order_number = o.lab_order_number
JOIN lab_items i ON i.lab_items_code = o.lab_items_code
WHERE h.hn = ?
  AND o.lab_items_code IN ({})
  AND o.lab_order_result IS NOT NULL
  AND o.lab_order_result <> ''
  AND h.lab_order_number = (
      SELECT h2.lab_order_number
      FROM lab_head h2
      JOIN lab_order o2 ON o2.lab_order_number = h2.lab_order_number
      WHERE h2.hn = h.hn
        AND o2.lab_items_code = o.lab_items_code
        AND o2.lab_order_result IS NOT NULL
        AND o2.lab_order_result <> ''
      ORDER BY h2.order_date DESC, h2.lab_order_number DESC
      LIMIT 1
  )
ORDER BY i.lab_items_name ASC
"#,
            placeholders(unique.len())
        );
        assert_read_only(&sql)?;
        let mut query = sqlx::query_as::<_, LabRow>(sqlx::AssertSqlSafe(sql)).bind(hn);
        for code in &unique {
            query = query.bind(code);
        }
        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let parsed = parse_lab_value(row.lab_order_result.as_deref().unwrap_or_default());
                LabResultView {
                    lab_order_number: row.lab_order_number,
                    lab_items_code: row.lab_items_code,
                    lab_items_name: row.lab_items_name.unwrap_or_default(),
                    unit: row.lab_items_unit,
                    raw_value: parsed.raw,
                    numeric_value: parsed.numeric,
                    comparator: parsed.comparator.map(|value| value.as_str().to_string()),
                    observed_at: normalize_date_text(row.report_date.or(row.order_date)),
                }
            })
            .collect())
    }

    /// Latest OPD vital signs from `opdscreen`.
    pub async fn opd_vitals(&self, hn: &str) -> Result<Vec<VitalSignView>> {
        assert_read_only(SQL_OPD_VITALS)?;
        let row = sqlx::query_as::<_, OpdVitalRow>(SQL_OPD_VITALS)
            .bind(hn)
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Ok(Vec::new());
        };
        let observed_at = normalize_date_text(row.vstdate);
        let mut values = Vec::new();
        if let (Some(systolic), Some(diastolic)) = (row.bps, row.bpd.clone()) {
            values.push(vital(
                "OPD",
                "Blood pressure",
                format!("{systolic}/{diastolic}"),
                Some("mmHg"),
                observed_at.clone(),
            ));
        }
        push_optional(&mut values, "OPD", "Pulse", row.pulse, "bpm", &observed_at);
        push_optional(
            &mut values,
            "OPD",
            "Respiratory rate",
            row.rr,
            "/min",
            &observed_at,
        );
        push_optional(
            &mut values,
            "OPD",
            "Temperature",
            row.temperature,
            "C",
            &observed_at,
        );
        push_optional(
            &mut values,
            "OPD",
            "Body weight",
            row.bw,
            "kg",
            &observed_at,
        );
        push_optional(&mut values, "OPD", "Height", row.height, "cm", &observed_at);
        push_optional(&mut values, "OPD", "BMI", row.bmi, "kg/m2", &observed_at);
        Ok(values)
    }

    /// Latest value per IPD vital sign.
    pub async fn ipd_vitals(&self, an: &str) -> Result<Vec<VitalSignView>> {
        assert_read_only(SQL_IPD_VITALS)?;
        let rows = sqlx::query_as::<_, IpdVitalRow>(SQL_IPD_VITALS)
            .bind(an)
            .fetch_all(&self.pool)
            .await?;
        let mut seen = HashSet::new();
        let mut values = Vec::new();
        for row in rows {
            let name = row.name.unwrap_or_default();
            if name.trim().is_empty() || !seen.insert(name.clone()) {
                continue;
            }
            let value = row.value.unwrap_or_default();
            if value.trim().is_empty() {
                continue;
            }
            values.push(VitalSignView {
                source: "IPD".to_string(),
                name,
                value,
                unit: row.unit,
                observed_at: normalize_date_text(row.observed_at),
                std_min: row.std_min.and_then(|value| value.parse::<f64>().ok()),
                std_max: row.std_max.and_then(|value| value.parse::<f64>().ok()),
            });
            if values.len() >= 20 {
                break;
            }
        }
        Ok(values)
    }

    /// Allergy list from `opd_allergy`.
    pub async fn allergies(&self, hn: &str) -> Result<Vec<AllergyView>> {
        assert_read_only(SQL_ALLERGIES)?;
        let rows = sqlx::query_as::<_, AllergyRow>(SQL_ALLERGIES)
            .bind(hn)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| AllergyView {
                agent: row.agent.unwrap_or_default(),
                symptom: row.symptom,
                reporter: row.reporter,
                report_date: normalize_date_text(row.report_date),
                note: row.note,
            })
            .collect())
    }

    /// Diagnoses from OPD and IPD with LEFT JOIN labels.
    pub async fn diagnoses(&self, hn: &str) -> Result<Vec<DiagnosisView>> {
        assert_read_only(SQL_DIAGNOSES)?;
        let rows = sqlx::query_as::<_, DiagnosisRow>(SQL_DIAGNOSES)
            .bind(hn)
            .bind(hn)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| DiagnosisView {
                icd10: row.icd10,
                name: row.name,
                diag_type: row.diag_type,
                source: row.source,
                encounter: row.encounter,
            })
            .collect())
    }

    /// Search laboratory items for rule configuration.
    pub async fn search_lab_items(&self, query: &str) -> Result<Vec<LabItemRow>> {
        assert_read_only(SQL_LAB_ITEMS_SEARCH)?;
        let like = format!("%{}%", query.trim());
        let rows = sqlx::query_as::<_, LabItemRow>(SQL_LAB_ITEMS_SEARCH)
            .bind(query.trim())
            .bind(like)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    /// Search the drug master for HAD mapping.
    pub async fn search_drugitems(&self, query: &str) -> Result<Vec<DrugItemRow>> {
        assert_read_only(SQL_DRUGITEMS_SEARCH)?;
        let like = format!("%{}%", query.trim());
        let rows = sqlx::query_as::<_, DrugItemRow>(SQL_DRUGITEMS_SEARCH)
            .bind(query.trim())
            .bind(like)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    /// Build the patient cockpit from all read-only sections.
    pub async fn patient_cockpit(
        &self,
        hn: &str,
        had_drugs: &[HadDrug],
        lab_codes: &[String],
        days: u32,
    ) -> Result<PatientCockpit> {
        let patient = self
            .patient_by_hn(hn)
            .await?
            .ok_or_else(|| Error::Schema(format!("patient {hn} not found")))?;
        let admission = self.active_admission(hn).await?;
        let active_an = admission.as_ref().map(|(an, _, _)| an.clone());
        let active_ward = admission.as_ref().and_then(|(_, _, ward)| ward.clone());
        let active_admission_date = admission.as_ref().and_then(|(_, date, _)| date.clone());

        let had_orders = self.had_orders(hn, had_drugs, days).await?;
        let labs = self.latest_labs(hn, lab_codes).await?;
        let mut vitals = self.opd_vitals(hn).await?;
        if let Some(an) = &active_an {
            vitals.extend(self.ipd_vitals(an).await?);
        }
        let allergies = self.allergies(hn).await?;
        let diagnoses = self.diagnoses(hn).await?;

        Ok(PatientCockpit {
            patient,
            active_an,
            active_ward,
            active_admission_date,
            had_orders,
            labs,
            vitals,
            allergies,
            diagnoses,
            warnings: Vec::new(),
        })
    }
}

fn vital(
    source: &str,
    name: &str,
    value: String,
    unit: Option<&str>,
    observed_at: Option<String>,
) -> VitalSignView {
    VitalSignView {
        source: source.to_string(),
        name: name.to_string(),
        value,
        unit: unit.map(str::to_string),
        observed_at,
        std_min: None,
        std_max: None,
    }
}

fn push_optional(
    values: &mut Vec<VitalSignView>,
    source: &str,
    name: &str,
    value: Option<String>,
    unit: &str,
    observed_at: &Option<String>,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        values.push(vital(source, name, value, Some(unit), observed_at.clone()));
    }
}

/// HOSxP can store Buddhist Era years. Normalize any YYYY prefix >= 2500.
pub fn normalize_date_text(raw: Option<String>) -> Option<String> {
    let raw = raw?;
    let trimmed = raw.trim();
    if trimmed.len() < 4 {
        return Some(trimmed.to_string());
    }
    let Ok(year) = trimmed[..4].parse::<i32>() else {
        return Some(trimmed.to_string());
    };
    if year >= 2500 {
        Some(format!("{}{}", year - 543, &trimmed[4..]))
    } else {
        Some(trimmed.to_string())
    }
}

fn mask_cid(cid: Option<String>) -> Option<String> {
    let cid = cid?;
    let digits: String = cid.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let tail: String = digits
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    Some(format!("*********{tail}"))
}

#[derive(sqlx::FromRow)]
struct PatientRow {
    hn: String,
    name: Option<String>,
    birthday: Option<String>,
    cid: Option<String>,
}

impl From<PatientRow> for PatientSummary {
    fn from(row: PatientRow) -> Self {
        Self {
            hn: row.hn,
            name: row.name.unwrap_or_default().trim().to_string(),
            birthday: normalize_date_text(row.birthday),
            cid_masked: mask_cid(row.cid),
        }
    }
}

#[derive(sqlx::FromRow)]
struct AdmissionRow {
    an: String,
    regdate: Option<String>,
    ward: Option<String>,
    ward_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct HadOrderRow {
    vn: Option<String>,
    an: Option<String>,
    icode: String,
    drug_name: Option<String>,
    strength: Option<String>,
    units: Option<String>,
    vstdate: Option<String>,
    qty: Option<i64>,
    sig: Option<String>,
    special_use: Option<String>,
    income_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct LabRow {
    lab_order_number: String,
    lab_items_code: String,
    lab_items_name: Option<String>,
    lab_items_unit: Option<String>,
    lab_order_result: Option<String>,
    order_date: Option<String>,
    report_date: Option<String>,
}

#[derive(sqlx::FromRow)]
struct OpdVitalRow {
    vstdate: Option<String>,
    bps: Option<String>,
    bpd: Option<String>,
    pulse: Option<String>,
    rr: Option<String>,
    temperature: Option<String>,
    bw: Option<String>,
    height: Option<String>,
    bmi: Option<String>,
}

#[derive(sqlx::FromRow)]
struct IpdVitalRow {
    observed_at: Option<String>,
    name: Option<String>,
    value: Option<String>,
    unit: Option<String>,
    std_min: Option<String>,
    std_max: Option<String>,
}

#[derive(sqlx::FromRow)]
struct AllergyRow {
    agent: Option<String>,
    symptom: Option<String>,
    reporter: Option<String>,
    report_date: Option<String>,
    note: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DiagnosisRow {
    icd10: String,
    name: Option<String>,
    diag_type: Option<String>,
    source: String,
    encounter: Option<String>,
}

/// A laboratory item for the rule editor.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabItemRow {
    pub lab_items_code: String,
    pub lab_items_name: String,
    pub lab_items_unit: Option<String>,
}

/// A drug master item for HAD mapping.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrugItemRow {
    pub icode: String,
    pub name: String,
    pub strength: Option<String>,
    pub units: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_buddhist_era_dates() {
        assert_eq!(
            normalize_date_text(Some("2568-10-21".into())),
            Some("2025-10-21".into())
        );
        assert_eq!(
            normalize_date_text(Some("2025-10-21".into())),
            Some("2025-10-21".into())
        );
    }

    #[test]
    fn masks_cid_to_last_four_digits() {
        assert_eq!(
            mask_cid(Some("1234567890123".into())),
            Some("*********0123".into())
        );
    }
}
