# HAD Tracer

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Leptos 0.8](https://img.shields.io/badge/Leptos-0.8-blue.svg)](https://leptos.dev)
[![Rust 1.94+](https://img.shields.io/badge/Rust-1.94%2B-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![sqlx 0.9](https://img.shields.io/badge/sqlx-0.9-336791.svg)](https://github.com/launchbadge/sqlx)

---

## ◆ PULSE

One question, answered with evidence: **which high-alert drug is running,
what does the laboratory say, and where did the process break?**

The moment an INR returns at 6.2, a potassium at 6.0, or a glucose at 48,
someone has to act. HAD Tracer reads the hospital's HOSxP database,
strictly read-only, evaluates drug-specific laboratory rules, and walks
the team through a seven-phase clinical tracer. It does not guess. It does
not write. It shows the order, the result, the gap, and the next move.

The frame is HA Edition 6 and 3P Safety: Patient Safety, Personnel
Safety, and People Safety.

| P0-P4 ▣ | P5 ▣ | P6 ▣ | P7 ◐ | P8 ☐ |
|---|---|---|---|---|
| Foundation, HOSxP, store, alerts, tracer | Red-orange safety interface | Docs and build | Pilot | v1.0 |

*Core, schema probe, local store, alert engine, tracer workflow, findings,
CAPA, backup, and the red-orange interface are sealed. The desktop build
is clean. The pilot is the next honest step.*

> Built with Tauri 2 + Leptos 0.8, answered by `had-tracer-core`, read from
> a verified HOSxP schema through `sqlx`, never a write, never a hidden
> clinical decision.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

Two installs, one test, one launch.

```text
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --locked
cargo test
cargo tauri dev
```

The release build is a single executable at
`target/release/had-tracer-app.exe`.

<details>
<summary>Notes for hospital PCs</summary>

- Windows needs the WebView2 Runtime. Updated Windows 10 and 11 normally
  have it already.
- The build is unsigned for v0.x. Expect the SmartScreen prompt, then
  coordinate allow-listing with hospital IT.
- HOSxP access is SELECT-only. The database grant is the primary control;
  the app also opens a read-only session and applies a statement timeout.
- Updates are manual by design. Hospital IT controls what runs on clinical
  workstations.

</details>

---

## ◆ ANATOMY

Five crates, one hard law: HOSxP is read-only, without exceptions.

- **Decides** - `had-tracer-core` is pure Rust. It owns the HAD catalog,
  the laboratory result parser, the rule evaluator, the tracer phases, the
  findings, and the CAPA model. No SQL, no filesystem, no Tauri, no network.
- **Reads** - `had-tracer-hosxp` talks to MySQL/MariaDB through `sqlx`.
  Parameterized `SELECT`s only, explicit columns, a schema probe before
  use, a read-only session, a statement timeout, and a guard that rejects
  anything that is not a single read statement.
- **Remembers** - `had-tracer-store` is a local SQLite file. It owns the
  HAD catalog, rules, alert events, tracer cases, findings, CAPA, audit
  log, backup, JSON export, JSON import, and file restore. WAL mode,
  foreign keys, integrity checks, UUID keys.
- **Guards secrets** - `had-tracer-config` encrypts HOSxP credentials with
  AES-256-GCM. The master key lives in the OS keychain. The password is a
  `secrecy::SecretString` and never touches a log.
- **Asks and shows** - `had-tracer-bridge` carries typed payloads over the
  Tauri IPC boundary. `had-tracer-app` is the Tauri 2 shell. The frontend
  is Leptos 0.8, compiled to WASM, and built with `trunk`.

**The interface is deliberately warm.** HAD is a risk, so the product does
not wear a calm green wellness palette. It uses soft ember for the anchor
surfaces, muted safety red for primary actions, warm orange for secondary
signals, warm white for clinical content, 12px cards, and pill buttons.
Severity is encoded in color, but color never replaces the evidence.

**The schema is not assumed.** The connector is built against a verified
HOSxP schema contract. It knows that `opitemrece` is not a drug-only table,
that `drugusage` uses `name1`, that `sp_use` uses `sp_name`, that
`lab_items` uses `lab_items_unit`, that `lab_head` has no `an`, and that
IPD vitals live in `ipt_vital_sign` plus `vital_sign`. The site-specific
detail lives in `AGENTS.md`. Missing tables and columns degrade into a
visible capability report, never a silent wrong answer.

---

## ◆ RITUALS

**The core ceremony** - from a patient to a finding:

1. Open Settings. Enter the HOSxP connection and run the schema probe.
2. Open HAD Catalog. Map each seeded HAD entry to a real
   `drugitems.icode`. Add or revise the safety bundle.
3. Open Rules. Map each rule to a real `lab_items.lab_items_code`, set the
   unit, and activate it after local approval.
4. Search for a patient. Read the HAD cockpit: orders, labs, OPD and IPD
   vitals, allergies, diagnoses.
5. Evaluate alerts. Acknowledge, override with a reason, or resolve with a
   note. Every action lands in the audit log.
6. Create a tracer case. Complete the seven phases. Record what should
   have happened, what actually happened, and where the system failed.
7. Turn the gap into a finding. Give it a severity, a risk score, a cause
   category, an owner, and a CAPA.
8. Back up the local store. Verify the manifest. Test a restore before
   trusting it.

**The ceremony of evidence** - a non-numeric result, a qualified value, a
unit mismatch, or a missing threshold is surfaced as uncertainty. It is
never collapsed into a normal verdict. The screen shows the raw value and
the rule it met.

**The ceremony of silence** - patient data stays local. No cloud sync, no
telemetry, no PHI in logs. The app reads HOSxP and writes only to its own
SQLite store. A backup is not synchronization.

**The ceremony of restraint** - the tool does not adjust doses, choose
therapy, or replace the clinician. It presents the order, the laboratory
result, the policy reference, and the system gap. Clinical judgment
belongs to the clinical team.

---

## ◆ ECHOES

**Where this artifact is heading**

```text
P0-P4 ▸ foundation, HOSxP, store, alerts, tracer ▸ sealed
P5    ▸ red-orange safety interface                ▸ sealed
P6    ▸ docs, build, local binary                  ▸ sealed
P7    ▸ pilot at the hospital                      ▸ ahead
P8    ▸ v1.0 hardening                             ▸ ahead
```

**Raising the artifact** - read `AGENTS.md` first. The hard rules are
there: HOSxP read-only, parameterized queries only, no PHI in logs,
credentials encrypted at rest, backups verified with
`PRAGMA integrity_check`, and clinical thresholds approved locally.

**Known edges** - single-user local store, no BCMA/eMAR/smart pump
integration, no scheduled HOSxP polling, a webview print dialog instead of
a dedicated PDF writer, and no KPI value-entry workflow yet. These are
written down because an honest artifact names its edges.

**Status** - the local gate is `cargo fmt`, `cargo clippy -- -D warnings`,
`cargo test`, the WASM frontend check, `trunk build`, and
`cargo tauri build --no-bundle`. All pass on the current tree.

> ⚠️ HAD Tracer reports what the record contains and what the local rules
> say. Clinical interpretation belongs to the clinician. This software is
> not medical advice.

---

```
  ─────────────────────────────────────────
   A missed high-alert drug is not a chart error.
   It is a patient-safety event waiting.
  ─────────────────────────────────────────
```

HAD Tracer is distributed under the [MIT License](LICENSE).
