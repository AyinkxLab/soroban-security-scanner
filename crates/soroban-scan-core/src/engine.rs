//! The scanner pipeline.
//!
//! The engine runs enabled rules over an immutable [`AnalysisContext`],
//! validates every finding against the scan root and known source files,
//! applies configuration overrides and filters, de-duplicates, and sorts the
//! result deterministically.

use std::collections::BTreeSet;
use std::path::Path;

use crate::config::ScanConfig;
use crate::context::AnalysisContext;
use crate::error::ScanError;
use crate::finding::Finding;
use crate::model::{Diagnostic, Project};
use crate::project::{self, LoadedProject};
use crate::registry::RuleRegistry;

/// Statistics about a scan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanStats {
    /// Number of rules executed.
    pub rules_run: usize,
    /// Number of rules skipped by configuration.
    pub rules_skipped: usize,
    /// Number of source files analyzed.
    pub files_analyzed: usize,
    /// Findings produced before validation/filtering.
    pub findings_raw: usize,
    /// Findings dropped because they referenced an unknown file.
    pub findings_rejected: usize,
}

/// The full result of a scan.
#[derive(Debug, Clone)]
pub struct ScanOutcome {
    /// Project metadata.
    pub project: Project,
    /// Final findings, sorted and de-duplicated.
    pub findings: Vec<Finding>,
    /// Diagnostics (project + pipeline).
    pub diagnostics: Vec<Diagnostic>,
    /// Scan statistics.
    pub stats: ScanStats,
}

/// Normalizes a relative path to forward slashes without a leading `./`.
fn normalize_rel(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    s.strip_prefix("./").unwrap_or(&s).to_string()
}

/// Returns true if `path` is a safe, relative path with no traversal.
fn is_safe_relative(path: &Path) -> bool {
    if path.is_absolute() {
        return false;
    }
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => return false,
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return false,
            _ => {}
        }
    }
    !path.as_os_str().is_empty()
}

/// Runs the scanner over a loaded project.
pub fn scan(loaded: &LoadedProject, registry: &RuleRegistry, config: &ScanConfig) -> ScanOutcome {
    let ctx = AnalysisContext::new(&loaded.project, &loaded.sources);
    let known: BTreeSet<String> = loaded
        .sources
        .iter()
        .map(|s| normalize_rel(&ctx.relative_path(&s.path)))
        .collect();

    let mut diagnostics = loaded.project.diagnostics.clone();
    let mut findings: Vec<Finding> = Vec::new();
    let mut stats = ScanStats {
        files_analyzed: loaded.sources.len(),
        ..ScanStats::default()
    };

    for rule in registry.rules() {
        let meta = rule.metadata();
        if !rule.enabled_by_default() || !config.rules.is_enabled(meta.id) {
            stats.rules_skipped += 1;
            continue;
        }
        stats.rules_run += 1;
        rule.analyze(&ctx, &mut findings);
    }

    stats.findings_raw = findings.len();
    let mut rejected = 0usize;

    findings.retain_mut(|finding| {
        if let Some(severity) = config.rules.severity_overrides.get(&finding.rule_id) {
            finding.severity = *severity;
        }
        if let Some(confidence) = config.rules.confidence_overrides.get(&finding.rule_id) {
            finding.confidence = *confidence;
        }

        // Relativize and validate the reported location.
        finding.location.file = ctx.relative_path(&finding.location.file);
        if !is_safe_relative(&finding.location.file) {
            diagnostics.push(Diagnostic::error(
                format!(
                    "rule {} produced an unsafe location: {}",
                    finding.rule_id,
                    finding.location.file.display()
                ),
                None,
            ));
            rejected += 1;
            return false;
        }
        let rel = normalize_rel(&finding.location.file);
        if !known.contains(&rel) {
            diagnostics.push(Diagnostic::error(
                format!(
                    "rule {} reported a file outside the analyzed set: {rel}",
                    finding.rule_id
                ),
                None,
            ));
            rejected += 1;
            return false;
        }
        if finding.evidence.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                format!(
                    "rule {} produced a finding without evidence",
                    finding.rule_id
                ),
                None,
            ));
            rejected += 1;
            return false;
        }

        finding.refresh_fingerprint();
        finding.severity.meets(config.min_severity) && finding.confidence >= config.min_confidence
    });

    stats.findings_rejected = rejected;

    // Deterministic ordering, then exact de-duplication.
    findings.sort_by_key(Finding::sort_key);
    let mut seen = BTreeSet::new();
    findings.retain(|f| seen.insert(f.identity()));

    ScanOutcome {
        project: loaded.project.clone(),
        findings,
        diagnostics,
        stats,
    }
}

/// Loads and scans a directory in one step.
pub fn run_scan(
    root: &Path,
    config: &ScanConfig,
    registry: &RuleRegistry,
) -> Result<ScanOutcome, ScanError> {
    let loaded = project::load(root, &config.discovery_options())?;
    Ok(scan(&loaded, registry, config))
}

/// Builds a synthetic single-file project from an in-memory source string.
///
/// This is useful for embedding and for regression tests. The synthetic project
/// is treated as Soroban when the source contains a Soroban SDK dependency
/// marker (assumed) or a contract definition.
pub fn synthetic_loaded(src: &str, file_name: &str) -> LoadedProject {
    let root = std::path::PathBuf::from(".");
    let path = std::path::PathBuf::from(file_name);
    let parsed = crate::source::parse_source(&path, &root, src);

    let mut diagnostics = Vec::new();
    if let Some(err) = &parsed.parse_error {
        diagnostics.push(Diagnostic::warning(
            format!("failed to parse source: {err}"),
            Some(path.clone()),
        ));
    }

    let mut info = crate::model::SorobanInfo {
        has_sdk_dependency: true,
        ..crate::model::SorobanInfo::default()
    };
    for location in &parsed.contract_definitions {
        crate::soroban::note_contract_definition(&mut info, location);
    }
    let kind = crate::soroban::classify(&info);

    let project = Project {
        root,
        kind,
        manifests: Vec::new(),
        soroban: info,
        source_files: 1,
        diagnostics,
    };
    LoadedProject {
        project,
        sources: vec![parsed],
    }
}

/// Scans an in-memory source string as a synthetic single-file project.
pub fn scan_source_str(
    src: &str,
    file_name: &str,
    config: &ScanConfig,
    registry: &RuleRegistry,
) -> ScanOutcome {
    scan(&synthetic_loaded(src, file_name), registry, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;
    use crate::confidence::Confidence;
    use crate::finding::SourceLocation;
    use crate::rule::{Rule, RuleMetadata};
    use crate::severity::Severity;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "soroban-scan-engine-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn project_fixture(name: &str) -> PathBuf {
        let root = tmpdir(name);
        write(
            &root.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n[dependencies]\nsoroban-sdk = \"21.0.0\"\n",
        );
        write(&root.join("src/lib.rs"), "fn a() {}\nfn b() {}\n");
        root
    }

    struct OneFindingRule {
        meta: RuleMetadata,
    }

    impl OneFindingRule {
        fn new(severity: Severity) -> Self {
            OneFindingRule {
                meta: RuleMetadata::new(
                    "SS-001",
                    "Test rule",
                    "test description",
                    Category::Authorization,
                    severity,
                    Confidence::Medium,
                    "fix it",
                ),
            }
        }
    }

    impl Rule for OneFindingRule {
        fn metadata(&self) -> &RuleMetadata {
            &self.meta
        }
        fn analyze(&self, ctx: &AnalysisContext<'_>, out: &mut Vec<Finding>) {
            if let Some(source) = ctx.sources().first() {
                let loc = SourceLocation::new(ctx.relative_path(&source.path), 1);
                out.push(Finding::new(&self.meta, loc, "fn a() {}"));
            }
        }
    }

    struct FabricatingRule {
        meta: RuleMetadata,
    }

    impl Rule for FabricatingRule {
        fn metadata(&self) -> &RuleMetadata {
            &self.meta
        }
        fn analyze(&self, _ctx: &AnalysisContext<'_>, out: &mut Vec<Finding>) {
            out.push(Finding::new(
                &self.meta,
                SourceLocation::new("../../etc/passwd", 1),
                "made up",
            ));
        }
    }

    fn fabricator() -> FabricatingRule {
        FabricatingRule {
            meta: RuleMetadata::new(
                "SS-002",
                "Fabricator",
                "bad",
                Category::Unsafe,
                Severity::High,
                Confidence::High,
                "n/a",
            ),
        }
    }

    #[test]
    fn runs_rules_and_emits_findings() {
        let root = project_fixture("run");
        let loaded = project::load(&root, &ScanConfig::default().discovery_options()).unwrap();
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(OneFindingRule::new(Severity::High)))
            .unwrap();
        let outcome = scan(&loaded, &registry, &ScanConfig::default());
        assert_eq!(outcome.findings.len(), 1);
        assert_eq!(outcome.stats.rules_run, 1);
        assert_eq!(outcome.findings[0].severity, Severity::High);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn filters_below_min_severity() {
        let root = project_fixture("filter");
        let loaded = project::load(&root, &ScanConfig::default().discovery_options()).unwrap();
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(OneFindingRule::new(Severity::Low)))
            .unwrap();
        let config = ScanConfig {
            min_severity: Severity::Medium,
            ..ScanConfig::default()
        };
        let outcome = scan(&loaded, &registry, &config);
        assert!(outcome.findings.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_fabricated_locations() {
        let root = project_fixture("fabricate");
        let loaded = project::load(&root, &ScanConfig::default().discovery_options()).unwrap();
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(fabricator())).unwrap();
        let outcome = scan(&loaded, &registry, &ScanConfig::default());
        assert!(outcome.findings.is_empty());
        assert_eq!(outcome.stats.findings_rejected, 1);
        assert!(outcome
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unsafe location")));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn disabled_rules_are_skipped() {
        let root = project_fixture("disabled");
        let loaded = project::load(&root, &ScanConfig::default().discovery_options()).unwrap();
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(OneFindingRule::new(Severity::High)))
            .unwrap();
        let config = ScanConfig {
            rules: crate::config::RulesConfig {
                disabled: vec!["SS-001".into()],
                ..crate::config::RulesConfig::default()
            },
            ..ScanConfig::default()
        };
        let outcome = scan(&loaded, &registry, &config);
        assert!(outcome.findings.is_empty());
        assert_eq!(outcome.stats.rules_skipped, 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scanning_is_deterministic() {
        let root = project_fixture("determinism");
        let loaded = project::load(&root, &ScanConfig::default().discovery_options()).unwrap();
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(OneFindingRule::new(Severity::High)))
            .unwrap();
        let a = scan(&loaded, &registry, &ScanConfig::default());
        let b = scan(&loaded, &registry, &ScanConfig::default());
        assert_eq!(
            serde_json::to_string(&a.findings).unwrap(),
            serde_json::to_string(&b.findings).unwrap()
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
