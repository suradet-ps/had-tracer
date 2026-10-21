use serde::{Deserialize, Serialize};

use crate::model::HadSeverity;

/// Comparison operator for an alert rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertOperator {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Between,
    Outside,
}

impl AlertOperator {
    /// Stable storage value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gt => "gt",
            Self::Ge => "ge",
            Self::Lt => "lt",
            Self::Le => "le",
            Self::Eq => "eq",
            Self::Between => "between",
            Self::Outside => "outside",
        }
    }

    /// Parse from storage.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "ge" => Self::Ge,
            "lt" => Self::Lt,
            "le" => Self::Le,
            "eq" => Self::Eq,
            "between" => Self::Between,
            "outside" => Self::Outside,
            _ => Self::Gt,
        }
    }

    /// Thai label for the UI.
    pub fn label_th(self) -> &'static str {
        match self {
            Self::Gt => "มากกว่า",
            Self::Ge => "มากกว่าหรือเท่ากับ",
            Self::Lt => "น้อยกว่า",
            Self::Le => "น้อยกว่าหรือเท่ากับ",
            Self::Eq => "เท่ากับ",
            Self::Between => "อยู่ในช่วง",
            Self::Outside => "อยู่นอกช่วง",
        }
    }
}

/// A versioned laboratory alert rule with drug context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub drug_scope_type: String,
    pub drug_scope_value: Option<String>,
    pub lab_items_code: String,
    pub lab_items_name: Option<String>,
    pub operator: AlertOperator,
    pub threshold_low: Option<f64>,
    pub threshold_high: Option<f64>,
    pub unit: Option<String>,
    pub lookback_hours: i64,
    pub severity: HadSeverity,
    pub action_text: String,
    pub require_active_order: bool,
    pub active: bool,
    pub version: i64,
    pub created_by: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub effective_at: Option<String>,
    pub review_due_at: Option<String>,
}

impl AlertRule {
    /// A compact threshold description for alert evidence.
    pub fn threshold_label(&self) -> String {
        let unit = self.unit.as_deref().unwrap_or("");
        match self.operator {
            AlertOperator::Between | AlertOperator::Outside => format!(
                "{} to {}{}",
                self.threshold_low.unwrap_or_default(),
                self.threshold_high.unwrap_or_default(),
                if unit.is_empty() {
                    String::new()
                } else {
                    format!(" {unit}")
                }
            ),
            _ => format!(
                "{} {}{}",
                self.operator.label_th(),
                self.threshold_low.unwrap_or_default(),
                if unit.is_empty() {
                    String::new()
                } else {
                    format!(" {unit}")
                }
            ),
        }
    }
}

/// Comparator found in a raw laboratory result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparator {
    Lt,
    Le,
    Gt,
    Ge,
}

impl Comparator {
    /// Symbol used in the UI.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }
}

/// A parsed laboratory result, retaining the raw text for auditability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLabValue {
    pub raw: String,
    pub numeric: Option<f64>,
    pub comparator: Option<Comparator>,
    pub unit: Option<String>,
}

impl ParsedLabValue {
    /// Whether the value is non-numeric or qualified in a way that makes
    /// exact comparison unsafe.
    pub fn is_uncertain(&self) -> bool {
        self.numeric.is_none() || self.comparator.is_some()
    }
}

/// Result of evaluating one alert rule against one laboratory value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvaluation {
    pub triggered: bool,
    pub uncertain: bool,
    pub severity: HadSeverity,
    pub reason: String,
    pub observed_value: Option<f64>,
    pub threshold_label: String,
}

/// Lifecycle status of an alert event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    New,
    Acknowledged,
    Overridden,
    Resolved,
    Expired,
}

impl AlertStatus {
    /// Stable storage value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Acknowledged => "acknowledged",
            Self::Overridden => "overridden",
            Self::Resolved => "resolved",
            Self::Expired => "expired",
        }
    }

    /// Parse from storage.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "acknowledged" => Self::Acknowledged,
            "overridden" => Self::Overridden,
            "resolved" => Self::Resolved,
            "expired" => Self::Expired,
            _ => Self::New,
        }
    }

    /// Thai label for the UI.
    pub fn label_th(self) -> &'static str {
        match self {
            Self::New => "ใหม่",
            Self::Acknowledged => "รับทราบ",
            Self::Overridden => "Override",
            Self::Resolved => "แก้ไขแล้ว",
            Self::Expired => "หมดอายุ",
        }
    }
}

/// Persisted alert event shown on the alert board.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub hn: String,
    pub an: Option<String>,
    pub vn: Option<String>,
    pub drug_icode: Option<String>,
    pub drug_name: Option<String>,
    pub lab_order_number: Option<String>,
    pub lab_items_code: String,
    pub lab_items_name: Option<String>,
    pub raw_value: String,
    pub numeric_value: Option<f64>,
    pub comparator: Option<String>,
    pub unit: Option<String>,
    pub observed_at: Option<String>,
    pub severity: HadSeverity,
    pub status: AlertStatus,
    pub action_text: String,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<String>,
    pub override_reason: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub resolve_note: Option<String>,
    pub created_at: String,
}

/// Parse a HOSxP laboratory result string.
///
/// HOSxP stores results as text. Values can be numeric, qualified with a
/// comparator, carry a unit, or be descriptive. This parser never panics and
/// never guesses when the value is not numeric.
pub fn parse_lab_value(raw: &str) -> ParsedLabValue {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return ParsedLabValue {
            raw: raw.to_string(),
            numeric: None,
            comparator: None,
            unit: None,
        };
    }

    let (comparator, rest) = if let Some(value) = trimmed.strip_prefix(">=") {
        (Some(Comparator::Ge), value)
    } else if let Some(value) = trimmed.strip_prefix("<=") {
        (Some(Comparator::Le), value)
    } else if let Some(value) = trimmed.strip_prefix('>') {
        (Some(Comparator::Gt), value)
    } else if let Some(value) = trimmed.strip_prefix('<') {
        (Some(Comparator::Lt), value)
    } else if let Some(value) = trimmed.strip_prefix('≥') {
        (Some(Comparator::Ge), value)
    } else if let Some(value) = trimmed.strip_prefix('≤') {
        (Some(Comparator::Le), value)
    } else {
        (None, trimmed)
    };

    let rest = rest.trim();
    let mut number = String::new();
    let mut remainder = String::new();
    let mut in_number = true;
    for (index, ch) in rest.char_indices() {
        let is_number_char = ch.is_ascii_digit()
            || ch == '.'
            || ch == ','
            || (ch == '-' && number.is_empty())
            || (ch == '+' && number.is_empty());
        if in_number && is_number_char {
            if ch != ',' {
                number.push(ch);
            }
        } else {
            if in_number {
                remainder = rest[index..].to_string();
            }
            in_number = false;
            if !in_number {
                break;
            }
        }
    }

    let numeric = number.parse::<f64>().ok();
    let unit = if remainder.trim().is_empty() {
        None
    } else {
        Some(normalize_unit(remainder.trim()))
    };

    ParsedLabValue {
        raw: raw.to_string(),
        numeric,
        comparator,
        unit,
    }
}

/// Normalize common HOSxP laboratory unit spellings for comparison.
pub fn normalize_unit(unit: &str) -> String {
    let compact = unit.trim().to_ascii_lowercase().replace(' ', "");
    match compact.as_str() {
        "mg/dl" | "mg/100ml" => "mg/dL".to_string(),
        "mmol/l" => "mmol/L".to_string(),
        "meq/l" => "mEq/L".to_string(),
        "u/l" | "iu/l" => "U/L".to_string(),
        "ng/ml" => "ng/mL".to_string(),
        "pg/ml" => "pg/mL".to_string(),
        "sec" | "secs" | "second" | "seconds" | "s" => "s".to_string(),
        "ratio" => "ratio".to_string(),
        "x10^3/ul" | "10^3/ul" | "k/ul" | "cells/mm3" => "10^3/uL".to_string(),
        other => other.to_string(),
    }
}

fn units_compatible(rule_unit: Option<&str>, result_unit: Option<&str>) -> bool {
    match (rule_unit, result_unit) {
        (None, _) | (_, None) => true,
        (Some(rule), Some(result)) => normalize_unit(rule) == normalize_unit(result),
    }
}

/// Evaluate one rule against one parsed laboratory value.
///
/// The evaluator is deliberately conservative. Missing thresholds, missing
/// numeric values, unit mismatch, or qualified values produce an explicit
/// uncertainty flag instead of a silent normal verdict.
pub fn evaluate_rule(rule: &AlertRule, lab: &ParsedLabValue) -> AlertEvaluation {
    let threshold_label = rule.threshold_label();
    let base = AlertEvaluation {
        triggered: false,
        uncertain: false,
        severity: rule.severity,
        reason: String::new(),
        observed_value: lab.numeric,
        threshold_label,
    };

    if !rule.active {
        return AlertEvaluation {
            reason: "rule is inactive".to_string(),
            ..base
        };
    }

    if !units_compatible(rule.unit.as_deref(), lab.unit.as_deref()) {
        return AlertEvaluation {
            uncertain: true,
            reason: format!("unit mismatch: rule {:?}, result {:?}", rule.unit, lab.unit),
            ..base
        };
    }

    let Some(value) = lab.numeric else {
        return AlertEvaluation {
            uncertain: true,
            reason: "laboratory value is not numeric".to_string(),
            ..base
        };
    };

    let Some(low) = rule.threshold_low else {
        return AlertEvaluation {
            uncertain: true,
            reason: "rule has no threshold".to_string(),
            ..base
        };
    };

    if matches!(
        rule.operator,
        AlertOperator::Between | AlertOperator::Outside
    ) && rule.threshold_high.is_none()
    {
        return AlertEvaluation {
            uncertain: true,
            reason: "range rule has no upper threshold".to_string(),
            ..base
        };
    }

    if lab.comparator.is_some()
        && matches!(
            rule.operator,
            AlertOperator::Eq | AlertOperator::Between | AlertOperator::Outside
        )
    {
        return AlertEvaluation {
            uncertain: true,
            reason: "qualified value cannot be compared exactly".to_string(),
            ..base
        };
    }

    let triggered = match rule.operator {
        AlertOperator::Gt => value > low,
        AlertOperator::Ge => value >= low,
        AlertOperator::Lt => value < low,
        AlertOperator::Le => value <= low,
        AlertOperator::Eq => (value - low).abs() < 0.000_001,
        AlertOperator::Between => {
            let high = rule.threshold_high.unwrap_or(low);
            value >= low && value <= high
        }
        AlertOperator::Outside => {
            let high = rule.threshold_high.unwrap_or(low);
            value < low || value > high
        }
    };

    let uncertain = lab.comparator.is_some();
    AlertEvaluation {
        triggered,
        uncertain,
        reason: if triggered {
            format!(
                "{} {} {}",
                lab.raw,
                if uncertain { "qualifies" } else { "meets" },
                rule.threshold_label()
            )
        } else {
            format!("{} does not meet {}", lab.raw, rule.threshold_label())
        },
        ..base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(operator: AlertOperator, low: f64, high: Option<f64>, unit: Option<&str>) -> AlertRule {
        AlertRule {
            id: "r1".into(),
            name: "Test rule".into(),
            description: String::new(),
            drug_scope_type: "drug_icode".into(),
            drug_scope_value: Some("WARF".into()),
            lab_items_code: "INR".into(),
            lab_items_name: Some("INR".into()),
            operator,
            threshold_low: Some(low),
            threshold_high: high,
            unit: unit.map(str::to_string),
            lookback_hours: 72,
            severity: HadSeverity::High,
            action_text: "Review".into(),
            require_active_order: true,
            active: true,
            version: 1,
            created_by: None,
            approved_by: None,
            approved_at: None,
            effective_at: None,
            review_due_at: None,
        }
    }

    #[test]
    fn parses_plain_numeric_value() {
        let parsed = parse_lab_value("3.5");
        assert_eq!(parsed.numeric, Some(3.5));
        assert!(parsed.comparator.is_none());
    }

    #[test]
    fn parses_comparator_and_unit() {
        let parsed = parse_lab_value("<0.5 mg/dL");
        assert_eq!(parsed.numeric, Some(0.5));
        assert_eq!(parsed.comparator, Some(Comparator::Lt));
        assert_eq!(parsed.unit.as_deref(), Some("mg/dL"));
    }

    #[test]
    fn rejects_non_numeric_value() {
        let parsed = parse_lab_value("Negative");
        assert!(parsed.numeric.is_none());
        assert!(parsed.is_uncertain());
    }

    #[test]
    fn gt_rule_triggers_above_threshold() {
        let parsed = parse_lab_value("4.2");
        let evaluation = evaluate_rule(&rule(AlertOperator::Gt, 3.5, None, Some("ratio")), &parsed);
        assert!(evaluation.triggered);
        assert!(!evaluation.uncertain);
    }

    #[test]
    fn lt_rule_does_not_trigger_above_threshold() {
        let parsed = parse_lab_value("70");
        let evaluation =
            evaluate_rule(&rule(AlertOperator::Lt, 70.0, None, Some("mg/dL")), &parsed);
        assert!(!evaluation.triggered);
    }

    #[test]
    fn qualified_value_is_uncertain() {
        let parsed = parse_lab_value("<0.5");
        let evaluation = evaluate_rule(&rule(AlertOperator::Lt, 1.0, None, None), &parsed);
        assert!(evaluation.triggered);
        assert!(evaluation.uncertain);
    }

    #[test]
    fn unit_mismatch_is_uncertain() {
        let parsed = parse_lab_value("5.5 mg/dL");
        let evaluation =
            evaluate_rule(&rule(AlertOperator::Gt, 3.5, None, Some("mmol/L")), &parsed);
        assert!(!evaluation.triggered);
        assert!(evaluation.uncertain);
    }

    #[test]
    fn between_rule_uses_both_thresholds() {
        let parsed = parse_lab_value("5.0");
        let evaluation = evaluate_rule(
            &rule(AlertOperator::Between, 4.0, Some(6.0), Some("mg/dL")),
            &parsed,
        );
        assert!(evaluation.triggered);
    }

    #[test]
    fn missing_threshold_is_uncertain() {
        let mut r = rule(AlertOperator::Gt, 3.5, None, None);
        r.threshold_low = None;
        let evaluation = evaluate_rule(&r, &parse_lab_value("4.2"));
        assert!(!evaluation.triggered);
        assert!(evaluation.uncertain);
    }
}
