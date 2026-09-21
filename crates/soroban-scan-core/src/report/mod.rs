//! Reporting: terminal, JSON, SARIF, and Markdown renderers.
//!
//! Machine-readable formats are stable and versioned by the tool version. No
//! renderer ever emits environment variables, absolute user paths, or secrets.

use serde::Serialize;

use crate::engine::ScanOutcome;
use crate::finding::Finding;
use crate::model::Diagnostic;
use crate::registry::RuleRegistry;
use crate::rule::RuleMetadata;
use crate::severity::Severity;

/// Tool name used in reports.
pub const TOOL_NAME: &str = "soroban-scan";
/// Tool homepage used in reports.
pub const TOOL_URL: &str = "https://github.com/AyinkxLab/soroban-security-scanner";
/// SARIF schema URI.
pub const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";

/// Output format for scan results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable terminal text.
    Terminal,
    /// Stable JSON.
    Json,
    /// SARIF 2.1.0 for code scanning.
    Sarif,
    /// Markdown report.
    Markdown,
}

impl OutputFormat {
    /// Parses a format name.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "terminal" | "text" => Ok(OutputFormat::Terminal),
            "json" => Ok(OutputFormat::Json),
            "sarif" => Ok(OutputFormat::Sarif),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            other => Err(format!("unknown output format: {other}")),
        }
    }

    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            OutputFormat::Terminal => "terminal",
            OutputFormat::Json => "json",
            OutputFormat::Sarif => "sarif",
            OutputFormat::Markdown => "markdown",
        }
    }
}

/// Renders a scan outcome in the requested format.
pub fn render(outcome: &ScanOutcome, registry: &RuleRegistry, format: OutputFormat) -> String {
    match format {
        OutputFormat::Terminal => render_terminal(outcome),
        OutputFormat::Json => render_json(outcome)
            .unwrap_or_else(|e| format!("{{\"error\":\"failed to serialize report: {e}\"}}")),
        OutputFormat::Sarif => render_sarif(outcome, registry),
        OutputFormat::Markdown => render_markdown(outcome),
    }
}

fn severity_counts(findings: &[Finding]) -> [usize; 5] {
    let mut counts = [0usize; 5];
    for finding in findings {
        counts[finding.severity.rank() as usize] += 1;
    }
    counts
}

/// Renders a concise human-readable summary line.
pub fn summary_line(findings: &[Finding]) -> String {
    let counts = severity_counts(findings);
    format!(
        "{} finding(s) — critical: {}, high: {}, medium: {}, low: {}, info: {}",
        findings.len(),
        counts[4],
        counts[3],
        counts[2],
        counts[1],
        counts[0]
    )
}

/// Renders the terminal report.
pub fn render_terminal(outcome: &ScanOutcome) -> String {
    let mut out = String::new();
    out.push_str(&format!("{TOOL_NAME} {}\n", crate::VERSION));
    out.push_str(&format!(
        "Project: {} ({})\n",
        outcome.project.root.display(),
        outcome.project.kind
    ));
    out.push_str(&format!(
        "Files: {}  Rules run: {}  Skipped: {}\n\n",
        outcome.stats.files_analyzed, outcome.stats.rules_run, outcome.stats.rules_skipped
    ));

    if outcome.findings.is_empty() {
        out.push_str("No security findings.\n");
    } else {
        for finding in &outcome.findings {
            out.push_str(&format!(
                "{:<8} {:<7} {}\n",
                finding.severity.as_str().to_ascii_uppercase(),
                finding.rule_id,
                finding.location.display_label()
            ));
            out.push_str(&format!("         {}\n", finding.title));
            out.push_str(&format!("         confidence: {}\n", finding.confidence));
            out.push_str(&format!("         evidence: {}\n", finding.evidence));
            out.push_str(&format!("         fix: {}\n\n", finding.remediation));
        }
        out.push_str(&summary_line(&outcome.findings));
        out.push('\n');
    }

    if !outcome.diagnostics.is_empty() {
        out.push_str(&format!("\n{} diagnostic(s):\n", outcome.diagnostics.len()));
        for diagnostic in &outcome.diagnostics {
            out.push_str(&format!("  - {}\n", diagnostic.message));
        }
    }

    out
}

/// Renders a compact terminal report (one line per finding).
pub fn render_terminal_compact(outcome: &ScanOutcome) -> String {
    let mut out = String::new();
    for finding in &outcome.findings {
        out.push_str(&format!(
            "{} {} {}\n",
            finding.severity.as_str(),
            finding.rule_id,
            finding.location.display_label()
        ));
    }
    out.push_str(&summary_line(&outcome.findings));
    out.push('\n');
    out
}

/// A serializable JSON report.
#[derive(Debug, Serialize)]
struct JsonReport<'a> {
    tool: ToolInfo,
    project: ProjectSummary,
    stats: StatsSummary,
    findings: &'a [Finding],
    diagnostics: &'a [Diagnostic],
}

/// Tool identity in JSON reports.
#[derive(Debug, Serialize)]
struct ToolInfo {
    name: &'static str,
    version: &'static str,
    information_uri: &'static str,
}

/// Project summary in JSON reports.
#[derive(Debug, Serialize)]
struct ProjectSummary {
    root: String,
    kind: String,
    source_files: usize,
    soroban_sdk_versions: Vec<String>,
}

/// Scan statistics in JSON reports.
#[derive(Debug, Serialize)]
struct StatsSummary {
    files_analyzed: usize,
    rules_run: usize,
    rules_skipped: usize,
    findings_raw: usize,
    findings_rejected: usize,
}

/// Renders the JSON report.
pub fn render_json(outcome: &ScanOutcome) -> Result<String, serde_json::Error> {
    let report = JsonReport {
        tool: ToolInfo {
            name: TOOL_NAME,
            version: crate::VERSION,
            information_uri: TOOL_URL,
        },
        project: ProjectSummary {
            root: outcome.project.root.display().to_string(),
            kind: outcome.project.kind.to_string(),
            source_files: outcome.project.source_files,
            soroban_sdk_versions: outcome.project.soroban.sdk_versions.clone(),
        },
        stats: StatsSummary {
            files_analyzed: outcome.stats.files_analyzed,
            rules_run: outcome.stats.rules_run,
            rules_skipped: outcome.stats.rules_skipped,
            findings_raw: outcome.stats.findings_raw,
            findings_rejected: outcome.stats.findings_rejected,
        },
        findings: &outcome.findings,
        diagnostics: &outcome.diagnostics,
    };
    serde_json::to_string_pretty(&report)
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low | Severity::Info => "note",
    }
}

/// Renders the SARIF 2.1.0 report.
pub fn render_sarif(outcome: &ScanOutcome, registry: &RuleRegistry) -> String {
    use serde_json::{json, Value};

    let rules: Vec<Value> = registry
        .metadata_sorted()
        .iter()
        .map(|meta| {
            json!({
                "id": meta.id,
                "name": meta.title,
                "shortDescription": { "text": meta.title },
                "fullDescription": { "text": meta.description },
                "helpUri": meta.references.first().copied().unwrap_or(TOOL_URL),
                "defaultConfiguration": { "level": sarif_level(meta.default_severity) },
                "properties": {
                    "category": meta.category.as_str(),
                    "confidence": meta.default_confidence.as_str(),
                    "severity": meta.default_severity.as_str()
                }
            })
        })
        .collect();

    let results: Vec<Value> = outcome
        .findings
        .iter()
        .map(|finding| {
            let mut region = json!({ "startLine": finding.location.line });
            if let Some(column) = finding.location.column {
                region["startColumn"] = json!(column);
            }
            if let Some(end_line) = finding.location.end_line {
                region["endLine"] = json!(end_line);
            }
            json!({
                "ruleId": finding.rule_id,
                "level": sarif_level(finding.severity),
                "message": { "text": format!("{} — {}", finding.title, finding.evidence) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": finding.location.file.to_string_lossy().replace('\\', "/")
                        },
                        "region": region
                    }
                }],
                "partialFingerprints": {
                    "sorobanScanFingerprint": finding.fingerprint
                },
                "properties": {
                    "severity": finding.severity.as_str(),
                    "confidence": finding.confidence.as_str(),
                    "category": finding.category.as_str(),
                    "remediation": finding.remediation
                }
            })
        })
        .collect();

    let document = json!({
        "$schema": SARIF_SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": TOOL_NAME,
                    "version": crate::VERSION,
                    "informationUri": TOOL_URL,
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&document).unwrap_or_else(|_| "{}".to_string())
}

/// Renders the Markdown report.
pub fn render_markdown(outcome: &ScanOutcome) -> String {
    let counts = severity_counts(&outcome.findings);
    let mut out = String::new();
    out.push_str("# Soroban Security Scan\n\n");
    out.push_str(&format!("- **Tool:** {TOOL_NAME} {}\n", crate::VERSION));
    out.push_str(&format!(
        "- **Project:** `{}` ({})\n",
        outcome.project.root.display(),
        outcome.project.kind
    ));
    out.push_str(&format!(
        "- **Files analyzed:** {}\n",
        outcome.stats.files_analyzed
    ));
    out.push_str(&format!("- **Findings:** {}\n\n", outcome.findings.len()));

    out.push_str("## Summary\n\n");
    out.push_str("| Severity | Count |\n| -------- | ----- |\n");
    for severity in [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ] {
        out.push_str(&format!(
            "| {} | {} |\n",
            severity,
            counts[severity.rank() as usize]
        ));
    }
    out.push('\n');

    if outcome.findings.is_empty() {
        out.push_str("No security findings.\n");
        return out;
    }

    out.push_str("## Findings\n\n");
    for finding in &outcome.findings {
        out.push_str(&format!("### {} — {}\n\n", finding.rule_id, finding.title));
        out.push_str(&format!("- **Severity:** {}\n", finding.severity));
        out.push_str(&format!("- **Confidence:** {}\n", finding.confidence));
        out.push_str(&format!("- **Category:** {}\n", finding.category));
        out.push_str(&format!(
            "- **Location:** `{}`\n",
            finding.location.display_label()
        ));
        out.push_str(&format!("- **Evidence:** {}\n", finding.evidence));
        out.push_str(&format!("- **Remediation:** {}\n", finding.remediation));
        if !finding.references.is_empty() {
            out.push_str("- **References:**\n");
            for reference in &finding.references {
                out.push_str(&format!("  - <{reference}>\n"));
            }
        }
        out.push('\n');
    }
    out
}

/// Renders the rule catalog for `soroban-scan rules`.
pub fn render_rule_list(metas: &[&RuleMetadata], format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            let items: Vec<_> = metas
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "title": m.title,
                        "category": m.category.as_str(),
                        "severity": m.default_severity.as_str(),
                        "confidence": m.default_confidence.as_str(),
                    })
                })
                .collect();
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        }
        _ => {
            let mut out = String::new();
            out.push_str(&format!(
                "{:<8} {:<8} {:<10} {}\n",
                "RULE", "SEVERITY", "CONFIDENCE", "TITLE"
            ));
            for meta in metas {
                out.push_str(&format!(
                    "{:<8} {:<8} {:<10} {}\n",
                    meta.id,
                    meta.default_severity.as_str(),
                    meta.default_confidence.as_str(),
                    meta.title
                ));
            }
            out
        }
    }
}

/// Renders a single rule for `soroban-scan explain`.
pub fn render_rule_detail(meta: &RuleMetadata, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "id": meta.id,
            "title": meta.title,
            "description": meta.description,
            "category": meta.category.as_str(),
            "severity": meta.default_severity.as_str(),
            "confidence": meta.default_confidence.as_str(),
            "remediation": meta.remediation,
            "references": meta.references,
        }))
        .unwrap_or_else(|_| "{}".to_string()),
        OutputFormat::Markdown => format!(
            "# {} — {}\n\n- **Severity:** {}\n- **Confidence:** {}\n- **Category:** {}\n\n\
## Description\n\n{}\n\n## Remediation\n\n{}\n\n## References\n\n{}\n",
            meta.id,
            meta.title,
            meta.default_severity,
            meta.default_confidence,
            meta.category,
            meta.description,
            meta.remediation,
            meta.references
                .iter()
                .map(|r| format!("- <{r}>"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        _ => {
            let mut out = String::new();
            out.push_str(&format!("{} — {}\n\n", meta.id, meta.title));
            out.push_str(&format!("Severity:   {}\n", meta.default_severity));
            out.push_str(&format!("Confidence: {}\n", meta.default_confidence));
            out.push_str(&format!("Category:   {}\n\n", meta.category));
            out.push_str(&format!("{}\n\n", meta.description));
            out.push_str(&format!("Remediation:\n{}\n\n", meta.remediation));
            out.push_str("References:\n");
            for reference in meta.references {
                out.push_str(&format!("  - {reference}\n"));
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScanConfig;
    use crate::engine;

    fn sample() -> (ScanOutcome, RuleRegistry) {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/corpus/SS-001/positive.rs"
        ))
        .unwrap();
        let registry = RuleRegistry::with_default_rules();
        let outcome =
            engine::scan_source_str(&src, "positive.rs", &ScanConfig::default(), &registry);
        (outcome, registry)
    }

    #[test]
    fn terminal_report_contains_findings() {
        let (outcome, _) = sample();
        let text = render_terminal(&outcome);
        assert!(text.contains("SS-001"));
        assert!(text.contains("finding(s)"));
    }

    #[test]
    fn json_report_is_valid_and_has_findings() {
        let (outcome, _) = sample();
        let text = render_json(&outcome).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["tool"]["name"], "soroban-scan");
        assert!(!value["findings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn sarif_report_is_valid_sarif() {
        let (outcome, registry) = sample();
        let text = render_sarif(&outcome, &registry);
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["version"], "2.1.0");
        assert_eq!(value["runs"][0]["tool"]["driver"]["name"], "soroban-scan");
        assert!(!value["runs"][0]["results"].as_array().unwrap().is_empty());
        // Every result must have a rule id and a location.
        for result in value["runs"][0]["results"].as_array().unwrap() {
            assert!(result["ruleId"].is_string());
            assert!(
                result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"].is_string()
            );
        }
    }

    #[test]
    fn markdown_report_contains_table() {
        let (outcome, _) = sample();
        let text = render_markdown(&outcome);
        assert!(text.contains("| Severity | Count |"));
        assert!(text.contains("SS-001"));
    }

    #[test]
    fn output_format_parsing() {
        assert_eq!(OutputFormat::parse("SARIF").unwrap(), OutputFormat::Sarif);
        assert_eq!(OutputFormat::parse("md").unwrap(), OutputFormat::Markdown);
        assert!(OutputFormat::parse("xml").is_err());
    }
}
