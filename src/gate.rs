//! Versioned policy decisions for firmware CI; descriptor semantics are unchanged.
use crate::{ChangeKind, Layout, compare, json};
/// Explicit review policy; neither variant permits unsupported semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Deny every modeled contract change.
    Strict,
    /// Permit entirely new reports while preserving every existing report.
    AddReports,
}
impl Policy {
    /// Stable serialized policy name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::AddReports => "add-reports",
        }
    }
}
/// Render a deterministic v1 review artifact and return whether policy permits it.
pub fn evaluate(old: &Layout, new: &Layout, policy: Policy) -> (bool, String) {
    let changes = compare(old, new);
    let mut permitted = true;
    let findings: Vec<_> = changes.iter().map(|change| {
        let allowed = policy == Policy::AddReports && change.kind == ChangeKind::ReportAdded;
        permitted &= allowed;
        let category = match change.kind {
            ChangeKind::ReportAdded => "report_added",
            ChangeKind::ReportRemoved => "report_removed",
            ChangeKind::ReportLength => "report_length",
            ChangeKind::FieldAdded => "field_added",
            ChangeKind::FieldRemoved => "field_removed",
            ChangeKind::FieldChanged => "field_changed",
        };
        format!("{{\"category\":{},\"allowed\":{allowed},\"report_kind\":{},\"report_id\":{},\"properties\":[{}]}}",json::string(category),json::string(&change.report.kind.to_string()),change.report.id,change.properties.iter().map(|p|json::string(p)).collect::<Vec<_>>().join(","))
    }).collect();
    (
        permitted,
        format!(
            "{{\"schema_version\":1,\"contract_version\":{},\"policy\":{},\"status\":{},\"error\":null,\"findings\":[{}]}}",
            crate::CONTRACT_VERSION,
            json::string(policy.name()),
            json::string(if permitted { "pass" } else { "deny" }),
            findings.join(",")
        ),
    )
}
/// Invalid/unsupported input is a separate outcome, never an empty passing diff.
pub fn invalid(policy: Policy, message: &str) -> String {
    format!(
        "{{\"schema_version\":1,\"contract_version\":{},\"policy\":{},\"status\":\"invalid\",\"error\":{},\"findings\":[]}}",
        crate::CONTRACT_VERSION,
        json::string(policy.name()),
        json::string(message)
    )
}
