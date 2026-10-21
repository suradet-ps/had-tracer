//! SQL used by the read-only HOSxP connector.
//!
//! Every statement is SELECT-only and every dynamic list is built from
//! placeholders, never from user-supplied text.

/// Required tables and columns for the verified pilot-site schema contract.
pub const REQUIRED_SCHEMA: &[(&str, &[&str])] = &[
    ("patient", &["hn", "pname", "fname", "lname", "birthday"]),
    ("ovst", &["vn", "hn", "vstdate", "main_dep"]),
    ("ipt", &["an", "hn", "vn", "regdate", "dchdate", "ward"]),
    (
        "opitemrece",
        &[
            "vn",
            "hn",
            "an",
            "icode",
            "qty",
            "vstdate",
            "drugusage",
            "sp_use",
            "income",
        ],
    ),
    ("drugitems", &["icode", "name", "strength", "units"]),
    ("drugusage", &["drugusage", "name1"]),
    ("sp_use", &["sp_use", "sp_name"]),
    ("lab_head", &["lab_order_number", "hn", "vn", "order_date"]),
    (
        "lab_order",
        &["lab_order_number", "lab_items_code", "lab_order_result"],
    ),
    (
        "lab_items",
        &["lab_items_code", "lab_items_name", "lab_items_unit"],
    ),
    (
        "opd_allergy",
        &["hn", "agent", "symptom", "reporter", "report_date", "note"],
    ),
    ("ovstdiag", &["vn", "hn", "icd10", "diagtype"]),
    ("iptdiag", &["an", "hn", "icd10", "diagtype"]),
    (
        "opdscreen",
        &[
            "hn",
            "vn",
            "vstdate",
            "bps",
            "bpd",
            "pulse",
            "rr",
            "temperature",
        ],
    ),
    (
        "ipt_vital_sign",
        &[
            "an",
            "vital_sign_id",
            "vital_sign_value",
            "vital_sign_datetime",
        ],
    ),
    (
        "vital_sign",
        &["vital_sign_id", "vital_sign_name", "vital_sign_unit"],
    ),
    ("kskdepartment", &["depcode", "department"]),
    ("ward", &["ward", "name"]),
    ("income", &["income", "name"]),
    ("drugaccount", &["drugaccount", "name"]),
    ("diagtype", &["diagtype", "name"]),
    ("icd101", &["code", "name"]),
];

/// Columns that are useful but not fatal when absent.
pub const OPTIONAL_SCHEMA: &[(&str, &str)] = &[
    ("lab_head", "report_date"),
    ("iptdiag", "icd11"),
    ("opdscreen", "bw"),
    ("opdscreen", "height"),
    ("opdscreen", "bmi"),
    ("opdscreen", "fbs"),
    ("opdscreen", "hba1c"),
    ("opdscreen", "egfr"),
    ("opdscreen", "creatinine"),
    ("opdscreen", "ldl"),
    ("vital_sign", "std_min"),
    ("vital_sign", "std_max"),
    ("patient", "cid"),
];

pub const SQL_SERVER_VERSION: &str = "SELECT VERSION() AS version";
pub const SQL_DATABASE_NAME: &str = "SELECT DATABASE() AS database_name";

pub const SQL_SCHEMA_COLUMNS: &str = r#"
SELECT TABLE_NAME AS table_name, COLUMN_NAME AS column_name
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
"#;

pub const SQL_PATIENT_SEARCH: &str = r#"
SELECT
    hn,
    TRIM(CONCAT_WS(' ', NULLIF(pname, ''), NULLIF(fname, ''), NULLIF(lname, ''))) AS name,
    CAST(birthday AS CHAR) AS birthday,
    cid
FROM patient
WHERE hn = ? OR hn LIKE ? OR fname LIKE ? OR lname LIKE ?
ORDER BY (hn = ?) DESC, hn ASC
LIMIT ?
"#;

pub const SQL_PATIENT_BY_HN: &str = r#"
SELECT
    hn,
    TRIM(CONCAT_WS(' ', NULLIF(pname, ''), NULLIF(fname, ''), NULLIF(lname, ''))) AS name,
    CAST(birthday AS CHAR) AS birthday,
    cid
FROM patient
WHERE hn = ?
LIMIT 1
"#;

pub const SQL_ACTIVE_ADMISSION: &str = r#"
SELECT
    i.an,
    CAST(i.regdate AS CHAR) AS regdate,
    i.ward,
    w.name AS ward_name
FROM ipt i
LEFT JOIN ward w ON w.ward = i.ward
WHERE i.hn = ?
  AND (i.dchdate IS NULL OR i.dchdate = '0000-00-00')
ORDER BY i.regdate DESC
LIMIT 1
"#;

pub const SQL_OPD_VITALS: &str = r#"
SELECT
    CAST(vstdate AS CHAR) AS vstdate,
    CAST(bps AS CHAR) AS bps,
    CAST(bpd AS CHAR) AS bpd,
    CAST(pulse AS CHAR) AS pulse,
    CAST(rr AS CHAR) AS rr,
    CAST(temperature AS CHAR) AS temperature,
    CAST(bw AS CHAR) AS bw,
    CAST(height AS CHAR) AS height,
    CAST(bmi AS CHAR) AS bmi
FROM opdscreen
WHERE hn = ?
ORDER BY vstdate DESC, vn DESC
LIMIT 1
"#;

pub const SQL_IPD_VITALS: &str = r#"
SELECT
    CAST(v.vital_sign_datetime AS CHAR) AS observed_at,
    s.vital_sign_name AS name,
    CAST(v.vital_sign_value AS CHAR) AS value,
    s.vital_sign_unit AS unit,
    CAST(s.std_min AS CHAR) AS std_min,
    CAST(s.std_max AS CHAR) AS std_max
FROM ipt_vital_sign v
JOIN vital_sign s ON s.vital_sign_id = v.vital_sign_id
WHERE v.an = ?
ORDER BY v.vital_sign_datetime DESC
LIMIT 200
"#;

pub const SQL_ALLERGIES: &str = r#"
SELECT
    agent,
    symptom,
    reporter,
    CAST(report_date AS CHAR) AS report_date,
    note
FROM opd_allergy
WHERE hn = ?
ORDER BY report_date DESC
LIMIT 50
"#;

pub const SQL_DIAGNOSES: &str = r#"
SELECT * FROM (
    SELECT
        d.icd10,
        i.name AS name,
        dt.name AS diag_type,
        'OPD' AS source,
        d.vn AS encounter
    FROM ovstdiag d
    LEFT JOIN icd101 i ON i.code = d.icd10
    LEFT JOIN diagtype dt ON dt.diagtype = d.diagtype
    WHERE d.hn = ?
    UNION ALL
    SELECT
        d.icd10,
        i.name AS name,
        dt.name AS diag_type,
        'IPD' AS source,
        d.an AS encounter
    FROM iptdiag d
    LEFT JOIN icd101 i ON i.code = d.icd10
    LEFT JOIN diagtype dt ON dt.diagtype = d.diagtype
    WHERE d.hn = ?
) u
ORDER BY source ASC, encounter DESC
LIMIT 100
"#;

pub const SQL_LAB_ITEMS_SEARCH: &str = r#"
SELECT lab_items_code, lab_items_name, lab_items_unit
FROM lab_items
WHERE lab_items_code = ? OR lab_items_name LIKE ?
ORDER BY lab_items_name ASC
LIMIT 50
"#;

pub const SQL_DRUGITEMS_SEARCH: &str = r#"
SELECT icode, name, strength, units
FROM drugitems
WHERE icode = ? OR name LIKE ?
ORDER BY name ASC
LIMIT 50
"#;

/// Build `?, ?, ?` for an IN list.
pub fn placeholders(count: usize) -> String {
    std::iter::repeat_n("?", count)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_parameterised() {
        assert_eq!(placeholders(3), "?, ?, ?");
        assert!(!placeholders(3).contains('\''));
    }

    #[test]
    fn required_schema_covers_core_tables() {
        let tables: Vec<&str> = REQUIRED_SCHEMA.iter().map(|(table, _)| *table).collect();
        for table in [
            "patient",
            "opitemrece",
            "lab_head",
            "lab_order",
            "ipt_vital_sign",
            "vital_sign",
        ] {
            assert!(tables.contains(&table), "missing {table}");
        }
    }
}
