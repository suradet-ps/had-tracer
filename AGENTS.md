# AGENTS.md - HAD Tracer

## 1. Project Overview

HAD Tracer is a read-only desktop application for clinical tracer review of high-alert drugs (HAD). It connects to HOSxP (MySQL/MariaDB), detects HAD orders and related laboratory alerts, and guides a multidisciplinary team through tracer review, finding management, corrective action, and outcome reporting.

The product supports Thai hospitals that work under HA Edition 6 and the 3P Safety framework:

- Patient Safety
- Personnel Safety
- People Safety

HAD Tracer is not a CPOE, eMAR, or replacement for the hospital information system. It reads HOSxP, stores tracer knowledge locally, and produces evidence for learning and improvement.

## 2. Product Goal

Enable a hospital to answer four questions with evidence:

1. Which high-alert drugs are being used, where, and for whom?
2. Which HAD orders have a laboratory result that requires review or action?
3. Where does the real care process break down across departments and handoffs?
4. Did the corrective action improve the system, and can we prove it on re-trace?

The product must produce data that is useful for HA survey preparation, NRLS learning, patient experience review, and THIP indicator tracking. Accreditation is an outcome, not the product goal.

## 3. Scope

### 3.1 In Scope

- Read-only connection to HOSxP MySQL/MariaDB with encrypted credentials.
- Schema probe to verify tables and columns before use.
- Patient search by HN, name, or visit identifiers.
- HAD medication summary for a patient across OPD and IPD.
- HAD catalog curation mapped to `drugitems.icode`.
- Per-drug safety bundle with dosing, preparation, administration, monitoring, antidote, and storage rules.
- Configurable laboratory alert rules with drug context.
- Alert review workflow with acknowledge, override, and audit.
- Automatic trigger to tracer case creation from an alert.
- Individual tracer and system tracer workflows.
- Tracer timeline built from HOSxP data plus manual evidence.
- Seven-phase tracer worksheet.
- Finding register with severity, standard mapping, and root cause notes.
- Corrective and preventive action (CAPA) tracking.
- Re-trace verification after CAPA.
- One-page tracer report and full report export to PDF.
- Dashboard for structure, process, outcome, and balancing indicators.
- Mapping fields for NRLS, Safety Culture Survey, PEP, and THIP.
- Local SQLite store for all app-owned data.
- Local backup, export, import, and restore with integrity checks and audit records.
- Thai UI in the MVP. English identifiers and English documentation.

### 3.2 Out of Scope for MVP

- Any write, update, delete, or DDL against HOSxP.
- Replacing CPOE, eMAR, pharmacy dispensing, or nursing documentation.
- Automatic dose adjustment or autonomous clinical decisions.
- Real-time bedside administration capture unless an eMAR source exists at the site.
- Barcode medication administration and smart pump integration.
- Full incident management system replacement.
- Multi-tenant SaaS deployment.
- Predictive analytics or machine learning.
- Billing, inventory, procurement, and stock management.
- Patient-facing mobile application.
- Cloud synchronization by default.

## 4. Standards and References

### 4.1 HA Edition 6

The primary accreditation reference is the Thai Hospital and Healthcare Standards, Edition 6. The system must support evidence collection for medication management, clinical tracer, risk management, patient safety, personnel safety, people safety, and outcome measurement.

Exact clause numbers must be verified against the official Edition 6 document before they are hardcoded in the application. Standard references must be stored as data, not embedded in source code.

### 4.2 3P Safety

The system must map findings and indicators to the three 3P domains:

- Patient Safety: preventable harm to patients from HAD use.
- Personnel Safety: hazardous drug handling, needlestick injury, workload, alert fatigue, and staff competency.
- People Safety: community and outpatient medication safety, emergency preparedness, and public health literacy.

### 4.3 National Learning and Measurement Tools

The system should be able to export or link data to:

- NRLS: incident and near-miss reporting, harm score, and root cause analysis.
- Safety Culture Survey: culture dimensions related to teamwork, communication, staffing, and safety climate.
- PEP: patient experience and patient-reported feedback on HAD education and care.
- THIP: hospital indicator definitions, numerators, denominators, and outcome trends.

### 4.4 HAD Clinical References

- ISMP List of High-Alert Medications in Acute Care Settings, 2024 edition.
- Thai national professional policy standards for high-alert drug management, as adopted by the hospital.
- Hospital Pharmacy and Therapeutics Committee (PTC) approved HAD list.
- Hospital-specific policies for anticoagulants, insulin, opioids, concentrated electrolytes, chemotherapy, sedation, and neuromuscular blockers.
- Joint Commission NPSG medication labeling and anticoagulant therapy requirements as an international benchmark.

## 5. Users and Roles

| Role | Main responsibilities |
|---|---|
| Pharmacist tracer lead | Select tracer cases, review HAD orders, verify alerts, lead findings |
| Nurse tracer | Observe preparation and administration, capture bedside evidence |
| Physician reviewer | Review clinical appropriateness and monitoring decisions |
| Quality and risk manager | Own CAPA, standard mapping, NRLS linkage, and survey evidence |
| Tracer team member | Complete worksheet phases and attach evidence |
| Executive viewer | Read dashboards and outcome summaries |
| Administrator | Manage connection settings, HAD catalog, rules, users, and audit |

MVP may run as a single-user desktop app with a local admin role. Role-based access control must be designed into the data model even if the UI exposes only one role in the first release.

## 6. Domain Glossary

- Clinical tracer: a method to follow a real patient journey or a real system process from start to finish, compare it with standards, and identify system failures and interfaces that break.
- Individual tracer: follows one patient through the care process.
- System tracer: follows one system, such as medication management, across departments and disciplines.
- High-alert drug (HAD): a drug that carries a heightened risk of significant patient harm when used in error, even if the error is not common.
- HAD safety bundle: the set of prescribing, preparation, administration, monitoring, and governance controls required for one drug or drug class.
- Alert rule: a versioned rule that combines a drug or drug class, a laboratory item, a threshold, a time window, a severity, and an action.
- Alert event: one evaluated instance of an alert rule for one patient and one laboratory result.
- Tracer case: one tracer review, either individual or system level.
- Tracer phase: one stage of the patient journey used in the worksheet.
- Finding: a gap, risk, or system weakness discovered during tracer review.
- CAPA: corrective and preventive action linked to a finding.
- Re-trace: a second tracer review used to verify that CAPA changed practice and outcomes.
- Trigger to tracer: automatic creation of a tracer case from a laboratory alert or HAD order signal.

## 7. Data Sources and Integration Boundaries

### 7.1 HOSxP Read-Only Access

HOSxP is the source of truth for patient, visit, drug order, dispensing, laboratory, allergy, diagnosis, and OPD and IPD vital sign data. All access is SELECT-only.

The schema below was verified against the live HOSxP database at the pilot site on 2026-10-21 using `information_schema` and live test queries. All 14 core table names are correct. The column corrections and supporting tables are mandatory.

| Data | Table | Verified columns and notes |
|---|---|---|
| Patient | `patient` | `hn`, `cid` (masked), `pname`, `fname`, `lname`, `birthday` (date) |
| OPD visit | `ovst` | `vn`, `hn`, `vstdate`, `vsttime`, `main_dep` (`char(3)`), `an` (bridge to IPD when present). Join `kskdepartment.depcode` for the department name. |
| IPD admission | `ipt` | `an`, `hn`, `vn`, `regdate`, `dchdate`, `ward` (`varchar(4)`). Join `ward.ward` for the ward name. |
| Drug and non-drug charges | `opitemrece` | `vn`, `hn`, `an`, `icode`, `qty` (`int(11)`, no decimals), `vstdate`, `drugusage`, `sp_use`, `income`. This is not a drug-only table. Join `drugitems` to restrict to drug lines. |
| Drug master | `drugitems` | `icode`, `name`, `strength`, `units`, `generic_name`, `drugaccount`, `therapeuticgroup`. `therapeutic` is free Thai text, not a coded class. Do not use it for standard grouping. |
| Sig | `drugusage` | `drugusage` (PK), `name1`, `name2`, `name3`, `shortlist`. The column is `name1`, not `name`. The PK column is `drugusage`, not `code`. |
| Special use | `sp_use` | `sp_use` (PK), `sp_name`, `name1`, `name2`, `name3`. The column is `sp_name`, not `name`. Use `drugusage` as the primary sig source; `sp_use` is used far less often. |
| Laboratory header | `lab_head` | `lab_order_number`, `hn`, `vn`, `order_date`, `report_date`. There is no `an` column. At the pilot site, `vn` is populated on every `lab_head` row. Join `lab_head.vn` to `ovst.vn`, then resolve the admission through `ovst.an`. |
| Laboratory result | `lab_order` | `lab_order_number`, `lab_items_code`, `lab_order_result`. The primary key is the pair. |
| Laboratory master | `lab_items` | `lab_items_code`, `lab_items_name`, `lab_items_unit`, `lab_items_normal_value`. The unit column is `lab_items_unit`, not `unit`. |
| Allergy | `opd_allergy` | `hn`, `agent`, `symptom`, `reporter`, `report_date`, `note`. PK is `(hn, agent)`. No visit key and no drug code. `agent` is free text (`varchar(250)`), so match by name, not by `icode`. |
| OPD diagnosis | `ovstdiag` | `vn`, `hn`, `icd10`, `diagtype` (`char(2)`). Join `diagtype.diagtype` and `icd101.code`. |
| IPD diagnosis | `iptdiag` | `an`, `hn`, `icd10`, `diagtype` (`char(2)`), `icd11`. Same lookup joins as `ovstdiag`. |
| OPD screening and vitals | `opdscreen` | `hn`, `vn`, `vstdate`, `bps`, `bpd`, `pulse`, `rr`, `temperature`, `bw`, `height`, `bmi`, `fbs`, `hba1c`, `egfr`, `creatinine`, `ldl`. OPD only, no `an`. The lab-like fields are sparse and must not be used as the primary lab source. |
| IPD vital signs | `ipt_vital_sign` | `an`, `vital_sign_id`, `vital_sign_value`, `vital_sign_datetime` |
| Vital sign master | `vital_sign` | `vital_sign_id`, `vital_sign_name`, `vital_sign_unit`, `std_min`, `std_max` |

Supporting lookup tables:

| Lookup | Table | Join |
|---|---|---|
| Department | `kskdepartment` | `ovst.main_dep` -> `kskdepartment.depcode`, display `department` |
| Ward | `ward` | `ipt.ward` -> `ward.ward`, display `name` |
| Income | `income` | `opitemrece.income` -> `income.income`, display `name` (21 rows) |
| Drug account | `drugaccount` | `drugitems.drugaccount` -> `drugaccount.drugaccount`, display `name` (7 rows) |
| Diagnosis type | `diagtype` | `ovstdiag.diagtype` or `iptdiag.diagtype` -> `diagtype.diagtype`, display `name` (5 rows) |
| ICD-10 | `icd101` | `icd10` -> `icd101.code`, display `name` or `tname`. 42,076 codes, stored without dots. |

Schema verification result:

- All 14 core table names exist at the pilot site. No table needs to be renamed.
- Three column names from the earlier draft were wrong: `drugusage.name` must be `drugusage.name1`, `sp_use.name` must be `sp_use.sp_name`, and `lab_items.unit` must be `lab_items.lab_items_unit`.
- Four supporting tables must be added: `ipt_vital_sign`, `vital_sign`, `kskdepartment`, and `ward`.
- `lab_head` has no `an`. A query that uses `lab_head.an` fails with MySQL error 42S22.
- `opdscreen` has no `an` and is OPD only. IPD vital signs come from `ipt_vital_sign` joined to `vital_sign`.
- `opitemrece` contains both drugs and non-drug charges. At the pilot site, roughly 46% of rows are drug lines and 54% are non-drug lines. Always join `drugitems` when counting or listing drugs, otherwise drug volume is inflated by roughly 2.2 times.
- `drugitems.therapeutic` is free Thai text with about 300 distinct values. Use `therapeuticgroup`, `generic_name`, or `drugaccount` for grouping instead.
- `opd_allergy.agent` has no drug code. Match allergy names as text and never assume an `icode` link.
- `icd101` does not contain every code used in `ovstdiag` or `iptdiag`. In a 30-day sample there were 2,618 orphan rows, including codes such as 9357 and 9929. Use `LEFT JOIN` and `IFNULL(name, icd10)` so patients are not dropped from reports.
- `sp_use` is used far less often than `drugusage`. At the pilot site, a 90-day sample showed `sp_use` used far less often than `drugusage`. Do not use `sp_use` as the primary sig source.
- `opdscreen` laboratory-like fields are sparse. In a one-year sample, only a few hundred to a few thousand values were present for `hba1c`, `egfr`, `ldl`, and `creatinine`. Read laboratory results from `lab_head` and `lab_order`, not from `opdscreen`.
- `opitemrece.qty` is an integer. Verify the dose unit, dosage form, and package unit before calculating days supply.

Query guardrails:

- Never select from `lab_head` with an `an` column.
- Always filter `opitemrece` through `drugitems` for drug counts and HAD detection.
- Always join lookup tables for department, ward, income, drug account, diagnosis type, and ICD-10 labels.
- Use `LEFT JOIN` for all diagnosis and ICD lookups.
- Use `lab_head.vn` and `ovst.vn` for both OPD and IPD laboratory linkage.
- Use `opdscreen` for OPD vitals only, and `ipt_vital_sign` plus `vital_sign` for IPD vitals.
- Keep a per-site schema capability report. The live schema must be verified again before a new site is enabled. Missing tables or columns must degrade gracefully and must not crash the application.

### 7.2 What HOSxP Cannot Provide

HOSxP does provide OPD vital signs in `opdscreen` and IPD vital signs in `ipt_vital_sign` joined to `vital_sign`. The gap is administration-level data, not vital sign data.

The following data is usually not available from the confirmed HOSxP order and dispensing tables:

- Bedside administration time.
- Independent double check and witness identity.
- Smart pump library entry and programmed rate.
- Barcode scan confirmation.
- Patient education delivery and understanding.
- Direct observation of preparation technique.
- Handoff communication quality.

These items must be captured as tracer evidence in the app, or imported later from an eMAR or BCMA source if the site has one. The UI must never imply that HOSxP confirms bedside administration.

### 7.3 App-Owned Data

The following data is owned by HAD Tracer and stored outside HOSxP:

- HAD catalog and safety bundles.
- Alert rules and alert events.
- Tracer cases, phases, checks, evidence, findings, CAPA, and re-traces.
- Standard references and mappings.
- KPI definitions and values.
- Audit log.
- User and role settings.
- Connection settings with encrypted credentials.

MVP stores app data in SQLite on the local machine. Turso is not part of the MVP. Any future store must implement the `TracerStore` trait so the domain layer does not depend on a specific database engine. A later multi-user deployment may use a separate MySQL schema or a self-hosted sync service with separate credentials and must never reuse the HOSxP read-only connection.

### 7.4 App Store, Backup, Export, Import, and Restore

The app store is a local SQLite database. It must be usable without a network connection and must never write to HOSxP.

Store requirements:

- Store the database file under the platform application data directory.
- Enable WAL mode, foreign keys, and a busy timeout.
- Store schema version, app version, site identifier, and install identifier in `app_meta`.
- Keep all app data local by default. No cloud sync in the MVP.
- Use TEXT UUID primary keys so export, import, and future synchronization do not depend on autoincrement values.

Backup requirements:

- Support manual backup and optional scheduled backup to a user-selected local path.
- Use a consistent SQLite backup method, such as the online backup API or `VACUUM INTO` after a WAL checkpoint.
- Include app version, schema version, site identifier, backup timestamp, and SHA-256 checksum in the backup manifest.
- Run `PRAGMA integrity_check` before and after the backup.
- Provide a configurable retention policy, for example daily, weekly, and monthly copies.
- Encrypt backup files that contain PHI, or require OS disk encryption plus access control at minimum.
- Never include database credentials or API tokens in a backup.
- Record every backup in the audit log.

Export requirements:

- Support a full logical export and a single tracer case export.
- Use a documented format such as SQL, JSON, or CSV with a manifest.
- Include app version, schema version, site identifier, export timestamp, row counts, and SHA-256 checksum.
- Exclude credentials, secrets, and connection strings.
- Apply the same PHI controls as the live application.
- Add a confidentiality notice to PDF and CSV exports.

Import requirements:

- Support merge and replace modes.
- Validate the manifest, schema version, app version compatibility, and checksum before writing.
- Provide a dry-run preview with row counts, duplicate identifiers, and conflicts.
- Merge by UUID. Resolve duplicates by skip, overwrite, or create a copy, and require an explicit choice.
- Never import into HOSxP.
- Record every import in the audit log.

Restore requirements:

- Restrict restore to an administrator.
- Stop writes and create a safety backup of the current database before restore.
- Verify integrity, schema version, and checksum before replacing the database.
- Apply pending migrations after restore.
- Reopen the database, verify core tables and row counts, and record the result in the audit log.
- Fail closed. On any error, keep the current database intact and report the failure.

Backup is not synchronization. A Turso sync or any future replication feature must not be described as a backup.

Test a full restore at least quarterly and before each release.

## 8. Tech Stack

- Language: Rust, edition 2024.
- Minimum supported Rust version: 1.94 or newer, required by sqlx 0.9.
- Shell: Tauri 2.
- Frontend: Leptos 0.8, client-side rendered, compiled to WebAssembly with trunk.
- HIS database access: sqlx 0.9 with MySQL/MariaDB, read-only connection only.
- App data store: sqlx 0.9 with a local SQLite file. Turso is a future option only and is not part of the MVP.
- Credential encryption: encryptman with encryptman-keyring for the OS keychain.
- Serialization: serde and serde_json.
- Errors: thiserror in library crates, typed command errors in the Tauri app.
- Logging: tracing and tracing-subscriber. No PHI in logs.
- Dates: chrono. HOSxP dates must be normalized from Buddhist Era to Common Era per value.
- PDF: pdf-writer with harfrust and skrifa for Thai text shaping, following the med-recon implementation.
- UI language: Thai in MVP. Code, identifiers, schema, and documentation are English.

sqlx feature guidance for 0.9:

- Use `runtime-tokio`.
- Use a TLS feature such as `tls-rustls-ring-native-roots`.
- Do not use removed combined runtime and TLS features such as `runtime-tokio-native-tls`.
- Use runtime `query_as` and `FromRow`. Do not use compile-time macros that require `DATABASE_URL` at build time.

## 9. Repository Layout

```text
had-tracer/
  Cargo.toml
  rust-toolchain.toml
  AGENTS.md
  docs/
  crates/
    had-tracer-core/
    had-tracer-hosxp/
    had-tracer-store/
    had-tracer-config/
    had-tracer-bridge/
  apps/
    had-tracer-app/
      src/
      frontend/
```

### 9.1 Crate Responsibilities

| Crate | Responsibility | Must not contain |
|---|---|---|
| `had-tracer-core` | Domain types, HAD catalog model, rule model, parser, evaluator, tracer model, pure business logic | I/O, sqlx, Tauri, filesystem, network |
| `had-tracer-hosxp` | SELECT-only HOSxP queries, row mapping, schema probe, schema variation handling | Writes, business rules, UI strings |
| `had-tracer-store` | SQLite schema and migrations, CRUD for app-owned data, audit log | HOSxP access, Tauri commands |
| `had-tracer-config` | Connection settings, encrypted credential storage, keyring integration, app settings | Clinical logic |
| `had-tracer-bridge` | Serde DTOs and Tauri invoke transport for the WASM frontend | SQL, database access, clinical logic |
| `had-tracer-app` | Tauri 2 shell, command adapters, window management, PDF export wiring | SQL, clinical rules, business logic |
| `frontend` | Leptos 0.8 UI, routing, state, presentation | Direct database access, secrets |

Dependency direction:

```text
frontend -> bridge -> core
app -> core, hosxp, store, config, bridge
hosxp -> core, config
store -> core, config
config -> no clinical crates
core -> no I/O crates
```

No SQL may exist outside `had-tracer-hosxp` and `had-tracer-store`. No business logic may exist in `had-tracer-app`.

## 10. Functional Modules

### 10.1 Connection and Schema Probe

- Configure host, port, database, username, password, TLS mode, and site label.
- Encrypt credentials at rest.
- Test connection with a short timeout.
- Run a schema probe for required tables and columns.
- Detect MySQL or MariaDB version and window function support.
- Report missing schema features as capability flags, not fatal errors.
- Enforce a SELECT-only guard and a read-only session.
- Apply a statement timeout to every query.

### 10.2 Patient Search and HAD Cockpit

- Search by HN, first name, last name, or visit number.
- Show patient identity with PHI redaction controls.
- Show current or recent HAD orders with drug name, strength, route, sig, start date, and source visit or admission.
- Show latest relevant laboratory results with value, unit, reference range if available, and result date.
- Read laboratory values from `lab_head` and `lab_order`, not from the laboratory-like fields in `opdscreen`.
- Show latest OPD vital signs from `opdscreen` and IPD vital signs from `ipt_vital_sign` joined to `vital_sign`.
- Show allergy history relevant to HAD selection.
- Show active alerts sorted by severity.
- Show whether the patient is a candidate for tracer review.

### 10.3 HAD Catalog and Governance

- Curate HAD entries by mapping to `drugitems.icode`.
- Support class-level HAD entries and drug-specific entries.
- Store the safety bundle for each entry.
- Version every change with effective date, author, approver, and review due date.
- Flag unmapped HAD orders for review.
- Never depend on drug name alone as the identity key.
- Support annual review and re-approval workflow.

### 10.4 Alert Rule Engine

- Create, edit, version, activate, and retire rules.
- Combine drug or drug class with laboratory item and threshold.
- Support operators: greater than, greater than or equal, less than, less than or equal, equal, between, and outside range.
- Support lookback window in hours.
- Support severity: critical, high, medium, low, info.
- Support action text and suggested next step.
- Support optional conditions such as age, renal function, or active admission.
- Provide a test mode against a selected patient before activation.
- Evaluate on patient open, manual refresh, and optional background refresh.
- Create alert events only when the rule is active and the drug context is present.
- Deduplicate by patient, rule, drug, and laboratory order number.
- Require a reason for override.
- Record every acknowledge, override, resolve, and expire action in the audit log.

### 10.5 Failure Modes the Alert Engine Must Handle

- Non-numeric results such as negative, positive, trace, or free text.
- Results with comparators such as `<0.5` or `>100`.
- Results with units embedded in the value.
- Missing units in `lab_items`.
- Unit mismatch between rule and result.
- Duplicate laboratory orders for the same item.
- Corrected or amended laboratory results.
- Laboratory result dates stored in Buddhist Era or Common Era.
- Orders that have been discontinued but remain in `opitemrece`.
- OPD orders that are old but still appear active.
- `opitemrece` rows that are non-drug charges because `drugitems` was not joined.
- Queries that reference `lab_head.an`, which does not exist.
- `opdscreen` used as the laboratory source for sparse fields such as `hba1c`, `egfr`, `ldl`, or `creatinine`.
- `sp_use` used as the primary sig source when `drugusage` is present.
- `icd101` orphans that cause inner joins to drop patients from reports.
- `opd_allergy.agent` treated as a coded drug field.
- Integer `qty` interpreted as a measured dose without checking the dose unit and package unit.
- Schema differences across HOSxP versions and sites.

Unparseable or ambiguous results must be surfaced as data quality issues. They must not be silently treated as normal.

### 10.6 Trigger to Tracer

- Allow an alert event or a HAD order to create a tracer case.
- Pre-fill the case with patient, encounter, HAD drugs, laboratory results, and allergy data.
- Pre-select the tracer type: individual, system, or program-specific.
- Require a tracer lead and a purpose statement.
- Record the trigger source and the reason for selection.
- Avoid duplicate open tracer cases for the same patient and encounter unless explicitly allowed.

### 10.7 Clinical Tracer Workflow

- Create an individual tracer or system tracer.
- Build a timeline from HOSxP visits, admissions, HAD orders, laboratory results, allergies, and diagnoses.
- Allow manual events, observations, interviews, and document evidence.
- Complete a seven-phase worksheet.
- Record findings with severity, standard reference, and system cause category.
- Link findings to CAPA.
- Re-trace after CAPA and compare before and after evidence.
- Generate a one-page tracer report and a full report.
- Export PDF with Thai text shaping.

### 10.8 Finding and CAPA

- Finding fields: title, description, phase, standard reference, evidence links, severity, likelihood, risk score, system cause category, immediate action, recommended action.
- System cause categories: patient factors, task factors, individual factors, team factors, environment, equipment, organizational, policy, information technology.
- CAPA fields: action, owner, due date, status, verification method, verification date, effectiveness result.
- Status flow: open, assigned, in progress, implemented, verified, closed, reopened.
- Overdue CAPA must appear on the dashboard.
- Closure requires evidence of implementation and, where required, evidence of effectiveness from re-trace.

### 10.9 Reporting and Dashboards

- One-page tracer report: context, concern, critical to quality, what was seen, system gaps, actions, results, next re-trace.
- Full tracer report with evidence appendix.
- HAD catalog coverage report.
- Alert volume and alert burden report.
- Alert override report with reasons.
- CAPA status and overdue report.
- Unit-level comparison report.
- Standard coverage heat map.
- THIP-style indicator summary.
- NRLS linkage summary.
- Safety Culture, PEP, and People Safety mapping summary.

### 10.10 Administration and Audit

- Manage connection settings and site profile.
- Manage HAD catalog versions and approvals.
- Manage alert rules and activation.
- Manage standard reference library.
- Manage users, roles, and permissions.
- Export and import configuration without secrets.
- Manage backup, export, import, and restore workflows.
- Record audit events for PHI access, rule changes, alert actions, tracer changes, CAPA changes, backups, exports, imports, and restores.
- Provide data retention and purge controls.

## 11. Clinical Tracer Workflow

The worksheet uses seven phases. Each phase contains questions mapped to HA Edition 6, 3P Safety, and hospital policy.

| Phase | Focus | Example evidence |
|---|---|---|
| 1. Pre-encounter | History, prior HAD use, allergies, baseline laboratory values, medication reconciliation | OPD records, prior labs, allergy list, reconciliation form |
| 2. Prescribing | Indication, dose, route, frequency, duration, renal or hepatic adjustment, interaction check, target range | Order record, diagnosis, guideline, pharmacist note |
| 3. Verification | Pharmacist review, clarification, duplication check, look-alike and sound-alike check, high-risk population check | Verification queue, intervention note, order clarification |
| 4. Preparation and dispensing | Dilution, concentration, compatibility, labeling, segregation, storage, access control, double check | Label, preparation record, ADC log, double check record |
| 5. Administration | Right patient, right drug, right dose, right route, right time, right documentation, pump settings, line safety, monitoring during administration | eMAR if available, pump record, observation checklist, witness signature |
| 6. Monitoring and follow-up | Laboratory monitoring, vital signs, adverse effect screening, antidote readiness, dose adjustment, escalation | Lab trend, vital trend, monitoring note, escalation record |
| 7. Handoff and discharge | Transfer communication, discharge medication list, patient education, follow-up appointment, primary care handoff | Handoff note, discharge summary, education record, appointment |

For each phase, the tracer team records:

- What should happen according to policy and standard.
- What actually happened.
- Where the process differed.
- Which interface or system factor contributed.
- What evidence supports the observation.
- Whether the gap is isolated or systemic.

The tracer must not be reduced to a scoring checklist. Narrative evidence and system learning are required outputs.

## 12. HAD Safety Bundle

Each HAD entry should contain the following fields where applicable:

- Drug identity: `icode`, generic name, brand names, class, route, dosage form, strength.
- Clinical use: approved indications and common off-label uses.
- Dosing: usual dose, maximum dose, weight-based dose, renal adjustment, hepatic adjustment, pediatric note, geriatric note.
- Administration: maximum rate, concentration limit, diluent, compatibility, line requirement, filter requirement, infusion device requirement.
- Monitoring: laboratory parameters, vital signs, clinical signs, frequency, baseline requirements, target range.
- Hold and stop criteria: thresholds for holding, reducing, or stopping the drug.
- Antidote or reversal agent: name, dose, location, availability check.
- Double check: required or not, who can perform it, what must be verified.
- Storage and access: segregated storage, locked storage, limited stock, access restriction.
- Look-alike and sound-alike warnings: confused drug names and tall-man lettering.
- High-risk populations: renal impairment, hepatic impairment, pediatric, neonatal, pregnant, older adults, low body weight.
- Patient education: key messages, warning signs, follow-up requirements.
- References: guideline, policy, product label, and internal standard.
- Governance: version, author, approver, approval date, effective date, review due date.

MVP HAD classes:

- Anticoagulants: warfarin, unfractionated heparin, low molecular weight heparin, direct oral anticoagulants.
- Insulins: subcutaneous and intravenous.
- Opioids: all routes.
- Concentrated electrolytes: potassium chloride concentrate, hypertonic sodium chloride, magnesium sulfate injection.
- Sedation and anesthesia agents: midazolam, propofol, ketamine.
- Neuromuscular blocking agents: rocuronium, vecuronium, succinylcholine.
- Vasoactive and inotropic infusions: norepinephrine, epinephrine, dopamine, dobutamine, amiodarone.
- Chemotherapy and hazardous drugs, if the site includes them in the first release.
- Other site-defined HAD entries approved by the PTC.

## 13. Lab Alert Model

### 13.1 Rule Fields

```text
alert_rule
  id
  name
  description
  drug_scope_type        drug_icode | drug_class | any_had
  drug_scope_value
  lab_items_code
  lab_items_name_snapshot
  operator               gt | ge | lt | le | eq | between | outside
  threshold_low
  threshold_high
  unit
  lookback_hours
  severity               critical | high | medium | low | info
  action_text
  require_active_order
  condition_json         optional age, renal, ward, or population conditions
  version
  active
  created_by
  approved_by
  approved_at
  effective_at
  review_due_at
```

### 13.2 Evaluation Semantics

1. Resolve active HAD orders for the patient.
2. Resolve the latest valid laboratory result for the rule item within the lookback window.
3. If the rule uses a vital sign, resolve the latest OPD value from `opdscreen` or the latest IPD value from `ipt_vital_sign` joined to `vital_sign`, within the same lookback window.
4. Parse the raw result into a numeric value, comparator, and unit.
5. Normalize the unit to the rule unit.
6. Evaluate the condition.
7. Check for an existing alert event with the same patient, rule, drug, and laboratory order number or vital sign timestamp.
8. Create a new alert event only if no duplicate exists.
9. Present the alert with evidence and recommended action.
10. Never modify a HOSxP order or write a clinical decision back to HOSxP.

### 13.3 Alert Lifecycle

```text
new -> acknowledged -> resolved
new -> overridden -> resolved
new -> expired
acknowledged -> expired
```

Override requires a reason and a user identity. Resolve requires a note. Expire is automatic when the lookback window passes or the order is no longer active.

### 13.4 Example MVP Rules

| Drug or class | Lab | Condition | Severity | Suggested action |
|---|---|---|---|---|
| Warfarin | INR | greater than 3.5 | high | Hold or reduce dose, repeat INR, assess bleeding |
| Warfarin | INR | greater than 5.0 | critical | Hold, consider vitamin K, notify physician urgently |
| Heparin | aPTT | greater than 100 seconds | high | Review infusion rate, repeat aPTT, assess bleeding |
| Heparin | Platelet count | less than 100,000 | high | Assess for heparin-induced thrombocytopenia |
| Insulin | Glucose | less than 70 mg/dL | high | Treat hypoglycemia, review insulin order |
| Potassium chloride | Potassium | greater than 5.5 mmol/L | high | Hold potassium, repeat level, review ECG |
| Vancomycin | Trough | greater than 20 mg/L | medium | Review dose and interval, assess nephrotoxicity |
| Digoxin | Digoxin level | greater than 2.0 ng/mL | high | Assess toxicity, review dose and renal function |
| Opioid | Respiratory rate (OPD or IPD vital) | less than 10 per minute | critical | Stop opioid, assess sedation, consider naloxone |
| Methotrexate | Creatinine | rising from baseline | high | Hold, assess renal function, review hydration |

Rules are site-configurable. The table is a starting point, not a fixed clinical protocol.

## 14. App Data Model

The app store uses a local SQLite file in the MVP. Turso is not part of the MVP. Tables are logical and may be adjusted during implementation. Use TEXT UUID primary keys for all entities that can be exported, imported, or synchronized in the future.

```text
app_meta
  key, value

had_drug
  id, icode, name, generic_name, drug_class, route, severity,
  safety_bundle_json, version, active, approved_by, approved_at,
  effective_at, review_due_at

alert_rule
  id, name, drug_scope_type, drug_scope_value, lab_items_code,
  operator, threshold_low, threshold_high, unit, lookback_hours,
  severity, action_text, condition_json, version, active,
  created_by, approved_by, approved_at, effective_at, review_due_at

alert_event
  id, alert_rule_id, hn, an, vn, drug_icode, lab_order_number,
  lab_items_code, raw_value, numeric_value, comparator, unit,
  observed_at, severity, status, acknowledged_by, acknowledged_at,
  override_reason, resolved_by, resolved_at, resolve_note, created_at

tracer_case
  id, case_type, title, purpose, hn, an, vn, trigger_type,
  trigger_ref, lead_user, status, opened_at, closed_at

tracer_phase
  id, tracer_case_id, phase_code, status, summary

tracer_check
  id, tracer_case_id, tracer_phase_id, standard_code, question,
  expected, observed, gap, evidence_summary, finding_id

tracer_evidence
  id, tracer_case_id, tracer_phase_id, evidence_type, title,
  source, captured_at, captured_by, content, file_path

finding
  id, tracer_case_id, tracer_phase_id, title, description,
  standard_code, severity, likelihood, risk_score, cause_category,
  immediate_action, recommended_action, status, created_by, created_at

capa
  id, finding_id, action, owner, due_date, status, verification_method,
  verification_date, effectiveness_result, evidence, created_at, closed_at

re_trace
  id, tracer_case_id, previous_case_id, capa_id, summary,
  result, verified_at, verified_by

standard_ref
  id, framework, code, title, requirement, edition, active

kpi_definition
  id, code, name, domain, numerator, denominator, data_source,
  frequency, target, owner, active

kpi_value
  id, kpi_definition_id, period_start, period_end, numerator_value,
  denominator_value, result_value, note, created_at

audit_log
  id, actor, action, entity_type, entity_id, detail_json, created_at
```

PHI must be minimized in the app store. If a field is not needed for tracer evidence, do not persist it.

## 15. Quality Dimensions and Indicators

### 15.1 Donabedian Model

Structure indicators:

- HAD list approved and reviewed within 12 months.
- Percentage of HAD entries with an approved safety bundle.
- Percentage of HAD drugs with a standard order set or order template.
- Percentage of HAD drugs with segregation and access controls.
- Percentage of nurses and pharmacists with current HAD competency.
- Availability of antidotes and reversal agents.
- Availability of smart pump libraries or infusion devices.
- Coverage of read-only schema probe and data quality checks.

Process indicators:

- Percentage of HAD orders reviewed by a pharmacist before first dose.
- Percentage of HAD administrations with documented independent double check.
- Percentage of HAD infusions with correct pump library entry.
- Percentage of required laboratory monitoring completed on time.
- Percentage of HAD patients with documented education.
- Percentage of alerts acknowledged within the defined time.
- Percentage of tracer cases completed per quarter.
- Percentage of findings with an owner and due date.
- Percentage of CAPA closed on time.
- Percentage of CAPA verified by re-trace.

Outcome indicators:

- Preventable adverse drug events per 1,000 HAD orders.
- Preventable adverse drug events per 1,000 HAD administrations where administration data exists.
- Near-miss reports per 1,000 HAD orders.
- Percentage of HAD errors with harm severity E to I.
- Anticoagulant-related bleeding events.
- Insulin-related severe hypoglycemia.
- Opioid-related respiratory depression or naloxone use.
- Potassium chloride-related hyperkalemia or arrhythmia.
- Chemotherapy extravasation events.
- Unplanned ICU transfer, length of stay, mortality, and readmission where attribution is feasible.
- Patient experience and complaint themes related to HAD care.

Balancing indicators:

- Alert override rate.
- Alert burden per patient per day.
- Time from order to first dose for time-critical HAD.
- Staff-reported double check workload.
- False positive trigger rate.
- Time spent per tracer case.

### 15.2 IOM Quality Dimensions

Every indicator set should be reviewed against:

- Safety.
- Effectiveness.
- Patient-centeredness.
- Timeliness.
- Efficiency.
- Equity.

### 15.3 3P Mapping

Patient Safety:

- Alert rules, double check, monitoring, ADE prevention, patient education.

Personnel Safety:

- Hazardous drug handling, personal protective equipment, needlestick prevention, workload, alert fatigue, competency.

People Safety:

- Outpatient and community HAD safety, health literacy, emergency preparedness, access equity.

### 15.4 Four Data Tools Mapping

NRLS:

- Incident and near-miss reporting, harm score, RCA, learning summary.

Safety Culture Survey:

- Teamwork, communication, staffing, safety climate, reporting culture.

PEP:

- Patient experience of HAD education, monitoring, and follow-up.

THIP:

- Indicator definitions, numerators, denominators, trends, and targets.

## 16. Security, Privacy, and Compliance

- HOSxP connection must use a dedicated read-only database user.
- Every HOSxP statement must be SELECT-only and guarded by a keyword check.
- Set the session to read-only where supported.
- Apply a statement timeout to every query.
- Encrypt database credentials at rest with encryptman.
- Store the master key in the OS keychain with encryptman-keyring.
- Never log passwords, connection strings, HN, CID, patient names, or raw clinical records.
- Redact PHI in error messages and crash reports.
- Provide role-based access to PHI and tracer data.
- Record audit events for PHI access, exports, rule changes, and alert actions.
- Support data retention and secure purge.
- Do not enable telemetry or cloud sync by default.
- Treat all patient data as PDPA-protected personal data.
- Export files must be written only to a user-selected path.
- PDF and CSV exports must carry a confidentiality notice.
- Encrypt backup and export files that contain PHI, or require OS disk encryption and access control at minimum.
- Verify backup and restore integrity before replacing the live database.
- Never include database credentials, API tokens, or connection strings in a backup or export.

## 17. Non-Functional Requirements

- Patient search should return within 2 seconds on a typical hospital local network.
- Alert evaluation for one patient should return within 5 seconds.
- The UI must remain responsive during database queries.
- Tracer editing must work offline once the case is open.
- Local data must survive an application crash without corruption.
- Missing tables or columns must degrade gracefully with a visible capability warning.
- All dates must be normalized to Common Era before display or comparison.
- Date and time display uses the Asia/Bangkok timezone.
- The application must handle HOSxP schema variation across sites.
- The application must support MySQL 8 and MariaDB 10.2 or newer for window functions.
- If the site runs MySQL 5.7, queries must fall back to correlated subqueries or grouped joins.
- The application must not block the Tauri main thread.
- Logs must be structured and redact PHI.
- The application must start with a clear connection state: not configured, connected, degraded, or unreachable.
- Backup and restore must preserve database integrity, schema version, and audit records.
- Restore must be atomic from the user perspective. A failed restore must leave the current database usable.

## 18. Build and Development Workflow

Expected commands after scaffold:

```text
cargo check --workspace
cargo test --workspace
cargo check -p had-tracer-frontend --target wasm32-unknown-unknown
cargo tauri dev
cargo tauri build
```

Development rules:

- Confirm schema assumptions against a live read-only connection before writing a query.
- Keep `cargo test` at the workspace root fast and database-free.
- Gate integration tests behind an environment variable and run them only against a staging database.
- Never point tests at a production HOSxP database.
- Keep HAD rules and standard references in data, not in source code.
- Add a schema probe test for every new table or column dependency.

## 19. Coding Rules

- Use Rust edition 2024.
- Use thiserror in library crates. Do not use anyhow.
- The Tauri app uses a typed command error with a Thai message for the UI.
- Do not use unwrap or expect in production code without an invariant message.
- Do not use blocking filesystem or sleep calls inside async functions.
- Apply tokio timeouts to all external I/O.
- Keep SQL inside `had-tracer-hosxp` and `had-tracer-store` only.
- Keep business logic out of the Tauri shell.
- Keep PHI out of logs, panic messages, and analytics.
- Use serde DTOs for every frontend boundary.
- Use English identifiers, schema names, and comments.
- Use Thai for user-facing strings in the MVP.
- Do not hardcode drug names, laboratory codes, or standard clause numbers.
- Do not introduce a new dependency without checking the workspace license and maintenance status.
- Do not add comments that merely restate the code.
- Do not use em dashes in documentation. Use plain ASCII punctuation.

## 20. Testing Strategy

Unit tests:

- Laboratory result parser: numeric, comparator, unit, malformed, empty.
- Unit normalization and conversion.
- Rule evaluator: each operator, boundary values, lookback windows, duplicates.
- Alert lifecycle transitions.
- Risk score calculation.
- Tracer phase completion rules.
- Date normalization from Buddhist Era to Common Era.
- PHI redaction.

Query tests:

- Query string shape for schema variants.
- Read-only guard rejects non-SELECT statements.
- Missing table and missing column handling.
- Pagination and result limits.

Integration tests:

- Gated by an environment variable.
- Use a staging HOSxP database only.
- Verify patient search, HAD order detection, latest lab retrieval, and schema probe.
- Never run against production.

UI tests:

- Connection setup validation.
- Patient search and HAD cockpit rendering.
- Alert acknowledge and override.
- Tracer case creation and worksheet completion.
- PDF export with Thai text.

## 21. Roadmap

### Phase 0: Discovery and Foundation

- Confirm MySQL version and read-only user.
- Freeze the pilot-site schema contract documented in section 7.1.
- Probe the live schema for the confirmed tables, columns, lookup joins, and laboratory and vital sign bridges.
- Confirm eMAR, BCMA, and smart pump availability. IPD vital tables are confirmed as `ipt_vital_sign` and `vital_sign`.
- Freeze HAD list v0.1 with the PTC or designated owner.
- Freeze five alert rules for MVP.
- Scaffold the workspace and read-only connection.
- Build the schema probe and capability report.

### Phase 1: MVP

- Patient search.
- HAD cockpit with active orders and latest labs.
- HAD catalog with mapping and safety bundle.
- Alert rule engine with five rules.
- Alert board with acknowledge and override.
- Trigger to tracer case.
- Auto-generated timeline.
- Seven-phase worksheet.
- Finding register.
- One-page tracer report and PDF export.
- Local SQLite store.
- Local backup, export, import, and restore workflows with integrity checks.
- Audit log for critical actions.

### Phase 2: Improvement Loop

- CAPA workflow.
- Re-trace and effectiveness verification.
- KPI definitions and dashboard.
- NRLS, Safety Culture, PEP, and THIP export fields.
- Standard coverage heat map.
- Multi-user app store option.
- Scheduled backup and retention policy.
- Import and export of configuration and findings.

### Phase 3: Advanced Integration

- eMAR and BCMA integration where available.
- Smart pump library verification.
- Real-time or scheduled background alert refresh.
- Multi-site benchmarking.
- Predictive risk scoring with explainability.
- Population-level and area-based quality views.

## 22. MVP Definition of Done

The MVP is complete when all of the following are true:

- The app connects to a staging HOSxP database with a read-only user.
- The schema probe validates the confirmed pilot-site schema contract, including lookup joins and the `lab_head` `vn` bridge.
- The schema probe reports required tables and columns and identifies missing capabilities.
- A user can search for a patient by HN and open the HAD cockpit.
- The cockpit shows active or recent HAD orders and the latest relevant laboratory results.
- The cockpit shows OPD vital signs from `opdscreen` and IPD vital signs from `ipt_vital_sign` joined to `vital_sign`.
- A curator can map at least 15 HAD drugs to `drugitems.icode` and save a safety bundle.
- At least five alert rules can be configured, tested, and activated.
- An alert event is created for a matching patient, with evidence and no duplicate.
- A user can acknowledge or override an alert with an audit record.
- A user can create a tracer case from an alert or a patient.
- The tracer timeline includes HOSxP visits, admissions, HAD orders, labs, allergies, and diagnoses.
- A user can complete all seven worksheet phases and attach evidence.
- A user can record a finding with severity and standard reference.
- A user can export a one-page tracer report as a PDF with Thai text.
- A user can back up the local database, export it, import it, and restore it with integrity checks and audit records.
- No statement writes to HOSxP.
- Unit tests and query tests pass without a database.
- Integration tests pass against staging when enabled.
- PHI does not appear in logs or error messages.

## 23. Open Questions

- Which HOSxP version and MySQL or MariaDB version are used at the target site?
- Is a dedicated read-only database user available?
- Does the site have eMAR, BCMA, or smart pump integration? IPD vital tables are confirmed.
- Is the MVP single-user or multi-user?
- Will the MVP support only the pilot site or also other HOSxP sites with schema variants?
- After the MVP, should app data remain local SQLite or move to a shared store for multi-user tracer teams?
- Should backup files be encrypted with an app-managed key or rely on OS disk encryption?
- Who owns and approves the HAD list?
- What is the first scope: OPD, IPD, ER, or all three?
- Which five alert rules are mandatory for the first release?
- Which HA Edition 6 clause numbers map to each tracer question?
- Which NRLS and THIP fields are required by the hospital quality team?
- What data retention period is required?
- Is offline tracer editing required at the bedside?
- What is the preferred report format and hospital branding?

## 24. Current Implementation Status

Implemented in the repository:

- Rust workspace with core, config, HOSxP, store, bridge, Tauri, and Leptos crates.
- Read-only HOSxP MySQL connector with schema probe, statement timeout, and read-only session guards.
- Patient search, HAD cockpit, HAD orders, laboratory results, OPD and IPD vitals, allergies, and diagnoses.
- HAD catalog with icode mapping, safety bundle editing, version increment, and active toggle.
- Alert rule editor, laboratory item search, rule activation, alert evaluation, deduplication, acknowledge, override, and resolve.
- Tracer creation with seven phases and checklist questions, phase summaries, check editing, findings, and CAPA.
- Dashboard, findings and CAPA register, standard and KPI reference view, and settings.
- Local SQLite store with migrations, seed data, audit log, backup, JSON export, JSON import, and file restore.
- Tauri bundle build with embedded Leptos frontend.

Not yet implemented:

- BCMA, eMAR, and smart pump integration.
- Scheduled background HOSxP polling.
- Dedicated PDF writer. The report screen currently uses the webview print dialog.
- Multi-user store and role-based authentication.
- KPI value entry and trend charts.
- Automated integration tests against a staging HOSxP database.

## 25. Documentation Style

- Write documentation in English.
- Use plain ASCII punctuation.
- Do not use em dashes.
- Use short headings and direct sentences.
- Prefer tables for schema, roles, phases, rules, and indicators.
- Keep clinical claims traceable to a reference or a hospital policy.
- Do not present the HAD list or alert thresholds as universal clinical truth. They are site-configurable and require local approval.
