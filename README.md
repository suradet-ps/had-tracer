# HAD Tracer

A local-first clinical tracer workspace for high-alert drugs (HAD). It reads HOSxP, surfaces drug-specific laboratory alerts, and guides a multidisciplinary team through a seven-phase tracer review, findings, CAPA, and re-trace planning.

Built for Thai hospitals working under HA Edition 6 and 3P Safety: Patient Safety, Personnel Safety, and People Safety.

## Design DNA

HAD Tracer borrows the clarity of a developer platform and the restraint of a clinical record.

- Deep ember anchors the sidebar, hero band, and footer.
- Safety red and ember orange are reserved for primary actions and critical safety cues.
- Warm white surfaces, thin hairlines, and 12px cards keep clinical content readable.
- Euclid Circular A drives display type, with Sarabun for Thai text and Source Code Pro for clinical codes.
- Buttons are pills. Cards are calm. Color is used to encode severity, not decoration.
- Every screen answers one question: what happened, what should have happened, and what do we do next.

## Capabilities

- Read-only HOSxP connection with encrypted credentials.
- Schema probe before use, with a capability report.
- Patient search by HN or name.
- HAD cockpit with orders, laboratory results, OPD and IPD vitals, allergies, and diagnoses.
- HAD catalog mapped to `drugitems.icode`, with a per-drug safety bundle.
- Versioned laboratory alert rules with drug context.
- Alert board with acknowledge, override, resolve, and audit.
- Seven-phase tracer worksheet with evidence notes.
- Findings, risk score, CAPA, and effectiveness notes.
- Dashboard, standard reference, and KPI reference views.
- Local SQLite store with backup, JSON export, JSON import, and restore.
- Audit log for critical actions.

## Boundaries

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
- Report output: browser Print to PDF from the report screen.

## Requirements

- Rust 1.94 or newer. sqlx 0.9 requires Rust 1.94.
- `wasm32-unknown-unknown` target.
- `trunk` 0.21 or newer.
- Tauri CLI 2.
- Microsoft Edge WebView2 Runtime on Windows.
- A HOSxP database user with SELECT-only grants.

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

```text
cargo tauri dev
```

Production build without installer:

```text
cargo tauri build --no-bundle
```

The executable is written under `target/release/`.

Frontend-only build:

```text
trunk build apps/had-tracer-app/frontend/index.html --release
```

## First Run

1. Open Settings and enter the HOSxP host, port, database, user, password, and TLS mode.
2. Use Test Connection. Confirm the schema probe passes.
3. Open HAD Catalog and map each seeded HAD entry to a real `drugitems.icode`.
4. Open Rules, map each rule to a real `lab_items.lab_items_code`, set the unit, and activate it.
5. Search for a patient and open the Patient HAD Cockpit.
6. Use Evaluate Alerts, then create a tracer case.
7. Complete the seven-phase worksheet, record findings, and add CAPA.
8. Use Settings to create a backup and verify the backup manifest.

## HOSxP Schema Guardrails

The app is built around the verified HOSxP schema at the pilot site. Site-specific details stay in `AGENTS.md`; the public README keeps the reusable rules:

- `opitemrece` is not a drug-only table. Always join `drugitems`.
- `drugusage` uses `name1`, not `name`.
- `sp_use` uses `sp_name`, not `name`.
- `lab_items` uses `lab_items_unit`, not `unit`.
- `lab_head` has no `an`. Join through `vn`.
- `opdscreen` is OPD only. IPD vitals use `ipt_vital_sign` and `vital_sign`.
- `opd_allergy.agent` has no drug code and must be matched as text.
- ICD-10 joins need `LEFT JOIN` and a fallback label.

## Data and Safety

- Read-only HOSxP access.
- PHI stays local and is not logged.
- Credentials are encrypted at rest.
- Every critical action is written to the audit log.
- Backup integrity is checked with `PRAGMA integrity_check`.
- Backup is not synchronization.
- Test a restore before relying on a backup in production.
- Clinical thresholds are local policy and require hospital approval.

## Checks

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p had-tracer-frontend --target wasm32-unknown-unknown
```

## MVP Limits

- Single-user local store. Multi-user collaboration is not implemented.
- No BCMA, eMAR, or smart pump integration.
- No scheduled background HOSxP polling. Alerts are evaluated on demand.
- The printable report uses the webview print dialog. A dedicated PDF writer is not included yet.
- KPI definitions are seeded, but KPI values are entered outside the app until the value-entry workflow is added.
