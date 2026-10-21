# HAD Tracer

HAD Tracer is a read-only clinical tracer desktop application for high-alert drugs (HAD). It reads HOSxP (MySQL/MariaDB), evaluates drug-specific laboratory alerts, and guides a multidisciplinary team through a seven-phase tracer review, findings, CAPA, and re-trace planning.

The product is designed around HA Edition 6 and 3P Safety: Patient Safety, Personnel Safety, and People Safety.

## What It Does

- Connects to HOSxP with a read-only MySQL user.
- Probes the live schema before use and reports missing capabilities.
- Searches patients by HN or name.
- Shows HAD orders, laboratory results, OPD and IPD vital signs, allergies, and diagnoses.
- Curates a hospital HAD catalog mapped to `drugitems.icode`.
- Evaluates configurable laboratory alert rules with versioning.
- Creates tracer cases from a patient or an alert.
- Provides a seven-phase tracer worksheet with evidence notes.
- Tracks findings, risk score, CAPA, and effectiveness notes.
- Exports a printable one-page tracer narrative and KPI reference view.
- Backs up, exports, imports, and restores the local SQLite store.
- Keeps an audit log for critical actions.

## What It Does Not Do

- It never writes to HOSxP.
- It is not a CPOE, eMAR, or dispensing system.
- It does not replace bedside documentation.
- It does not send patient data to the cloud.
- It does not make autonomous clinical decisions.

## Architecture

```text
had-tracer/
  crates/
    had-tracer-core/      Pure domain: HAD catalog, alert parser/evaluator, tracer model
    had-tracer-config/    Encrypted HOSxP connection settings and site settings
    had-tracer-hosxp/     SELECT-only HOSxP connector and schema probe
    had-tracer-store/     Local SQLite store, migrations, seed, backup/export/import
    had-tracer-bridge/    Tauri invoke transport for WASM
  apps/had-tracer-app/    Tauri 2 shell, commands, and Leptos 0.8 frontend
```

- HIS data source: HOSxP MySQL/MariaDB, read-only.
- App data: local SQLite file under the platform data directory.
- Frontend: Leptos 0.8, client-side rendered to WASM.
- Database client: sqlx 0.9.
- Credential encryption: encryptman with OS keychain.
- Print view: browser Print to PDF from the report screen.

## Requirements

- Rust 1.94 or newer. sqlx 0.9 requires Rust 1.94.
- `wasm32-unknown-unknown` target.
- `trunk` 0.21 or newer.
- Tauri CLI 2.
- Microsoft Edge WebView2 Runtime on Windows.
- A HOSxP database user with SELECT-only grants.

Install the build tools:

```text
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli --version "^2"
```

## HOSxP Read-Only User

Create a dedicated account and grant only SELECT:

```sql
CREATE USER 'had_ro'@'%' IDENTIFIED BY 'replace-with-a-strong-password';
GRANT SELECT ON hos.* TO 'had_ro'@'%';
FLUSH PRIVILEGES;
```

The app also sets the session to read-only and applies a statement timeout, but the database grant is the primary control.

## Run

Development:

```text
cargo tauri dev
```

Production build without installer:

```text
cargo tauri build --no-bundle
```

The executable is written under `target/release/`.

Frontend-only check:

```text
trunk build apps/had-tracer-app/frontend/index.html --release
```

## First Run

1. Open Settings and enter the HOSxP host, port, database, user, password, and TLS mode.
2. Use Test Connection. Confirm the schema probe is green.
3. Open HAD Catalog and map each seeded HAD entry to a real `drugitems.icode`.
4. Open Rules, map each rule to a real `lab_items.lab_items_code`, set the unit, and activate it.
5. Search for a patient and open the Patient HAD Cockpit.
6. Use Evaluate Alerts, then create a tracer case.
7. Complete the seven-phase worksheet, record findings, and add CAPA.
8. Use Settings to create a backup and verify the backup manifest.

## Schema Contract

The verified schema contract at Sa Bophospital is documented in `AGENTS.md`. Important points:

- `opitemrece` is not a drug-only table. Always join `drugitems`.
- `drugusage` uses `name1`, not `name`.
- `sp_use` uses `sp_name`, not `name`.
- `lab_items` uses `lab_items_unit`, not `unit`.
- `lab_head` has no `an`. Join through `vn`.
- `opdscreen` is OPD only. IPD vitals use `ipt_vital_sign` and `vital_sign`.
- `opd_allergy.agent` has no drug code and must be matched as text.
- ICD-10 joins need `LEFT JOIN` and a fallback label.

## Quality and Safety

- Read-only HOSxP access.
- PHI stays local and is not logged.
- Credentials are encrypted at rest.
- Every critical action is written to the audit log.
- Backup integrity is checked with `PRAGMA integrity_check`.
- Backup is not synchronization.
- Test a restore before relying on a backup in production.
- Clinical thresholds are local policy. They require hospital approval.

## Checks

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p had-tracer-frontend --target wasm32-unknown-unknown
```

## Known MVP Limits

- Single-user local store. Multi-user collaboration is not implemented.
- No BCMA, eMAR, or smart pump integration.
- No scheduled background HOSxP polling. Alerts are evaluated on demand.
- The printable report uses the webview print dialog. A dedicated PDF writer is not included yet.
- KPI definitions are seeded, but KPI values are entered outside the app until the value-entry workflow is added.
