//! Command-line interface for the Soroban Security Scanner.
//!
//! Exit codes (stable):
//! * `0` — success, no findings at or above the configured failure threshold.
//! * `1` — findings at or above the configured failure threshold.
//! * `2` — usage error (bad arguments, unknown rule, invalid configuration).
//! * `3` — runtime error (I/O failure, unreadable scan root).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};

use soroban_scan_core::baseline::{self, Baseline};
use soroban_scan_core::category::Category;
use soroban_scan_core::confidence::Confidence;
use soroban_scan_core::config::ScanConfig;
use soroban_scan_core::engine;
use soroban_scan_core::project;
use soroban_scan_core::registry::RuleRegistry;
use soroban_scan_core::report::{self, OutputFormat};
use soroban_scan_core::severity::Severity;

const EXIT_SUCCESS: u8 = 0;
const EXIT_FINDINGS: u8 = 1;
const EXIT_USAGE: u8 = 2;
const EXIT_ERROR: u8 = 3;

const DEFAULT_CONFIG_FILE: &str = "soroban-scan.toml";

const CONFIG_TEMPLATE: &str = r#"# Soroban Security Scanner configuration
# See https://github.com/AyinkxLab/soroban-security-scanner

# Minimum severity to report: info, low, medium, high, critical
min_severity = "info"

# Minimum confidence to report: low, medium, high
min_confidence = "low"

# Glob patterns (relative to the scan root) to exclude.
exclude = []

# Follow symbolic links during discovery (disabled by default for safety).
follow_symlinks = false

[rules]
# Rule ids to disable.
disabled = []
# If non-empty, only these rule ids run.
enabled = []
# Per-rule severity overrides, for example: { "SS-001" = "critical" }
# severity_overrides = {}
# Per-rule confidence overrides, for example: { "SS-007" = "low" }
# confidence_overrides = {}
"#;

/// Argument value for `--format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FormatArg {
    Terminal,
    Json,
    Sarif,
    Markdown,
}

impl From<FormatArg> for OutputFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Terminal => OutputFormat::Terminal,
            FormatArg::Json => OutputFormat::Json,
            FormatArg::Sarif => OutputFormat::Sarif,
            FormatArg::Markdown => OutputFormat::Markdown,
        }
    }
}

/// Argument value for `--fail-on`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FailOnArg {
    None,
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl FailOnArg {
    fn threshold(self) -> Option<Severity> {
        match self {
            FailOnArg::None => None,
            FailOnArg::Info => Some(Severity::Info),
            FailOnArg::Low => Some(Severity::Low),
            FailOnArg::Medium => Some(Severity::Medium),
            FailOnArg::High => Some(Severity::High),
            FailOnArg::Critical => Some(Severity::Critical),
        }
    }
}

/// Deterministic security analysis for Stellar Soroban smart contracts.
#[derive(Debug, Parser)]
#[command(
    name = "soroban-scan",
    version,
    about = "Deterministic security analysis for Stellar Soroban smart contracts",
    long_about = "Statically analyzes Soroban/Rust source code and reports \
security findings with stable rule ids, severity, confidence, source locations, \
and remediation guidance. The scanner never executes, deploys, or mutates the \
code it analyzes.",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
// CLI argument structs differ in size by design; boxing them would hurt readability.
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Scan a directory or file for security findings.
    Scan(ScanArgs),
    /// List the available security rules.
    Rules(RulesArgs),
    /// Explain a rule in detail.
    Explain(ExplainArgs),
    /// Print the scanner name and version.
    Version,
    /// Write a starter configuration file.
    Init(InitArgs),
    /// Inspect the scanner configuration.
    Config(ConfigArgs),
}

#[derive(Debug, Args)]
struct ScanArgs {
    /// Directory or file to scan.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = FormatArg::Terminal)]
    format: FormatArg,

    /// Write the report to a file instead of stdout.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Only report findings at or above this severity.
    #[arg(long, value_name = "LEVEL")]
    min_severity: Option<Severity>,

    /// Only report findings at or above this confidence.
    #[arg(long, value_name = "LEVEL")]
    min_confidence: Option<Confidence>,

    /// Exclude a glob (relative to the scan root). Repeatable.
    #[arg(long = "exclude", value_name = "GLOB")]
    exclude: Vec<String>,

    /// Configuration file to use.
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Read a newline-separated list of files to scan, relative to PATH.
    #[arg(long, value_name = "FILE")]
    files_from: Option<PathBuf>,

    /// Disable a rule by id. Repeatable.
    #[arg(long = "disable", value_name = "RULE")]
    disable: Vec<String>,

    /// Run only the given rule(s), by id. Repeatable.
    #[arg(long = "rule", value_name = "RULE")]
    rule: Vec<String>,

    /// Exit with code 1 if a finding at or above this severity is present.
    #[arg(long, value_enum, default_value_t = FailOnArg::High)]
    fail_on: FailOnArg,

    /// Compare against a baseline file, marking findings as new or existing.
    #[arg(long, value_name = "FILE")]
    baseline: Option<PathBuf>,

    /// Write current findings to a baseline file and exit successfully.
    #[arg(long, value_name = "FILE")]
    write_baseline: Option<PathBuf>,

    /// With --baseline, report only findings that are new.
    #[arg(long, requires = "baseline")]
    new_only: bool,

    /// Incremental scan: do not report resolved findings.
    #[arg(long, requires = "baseline")]
    incremental: bool,

    /// Suppress the report header and diagnostics.
    #[arg(long, short = 'q', conflicts_with = "verbose")]
    quiet: bool,

    /// Include diagnostics in the report.
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Debug, Args)]
struct RulesArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = FormatArg::Terminal)]
    format: FormatArg,

    /// Filter by category.
    #[arg(long, value_name = "CATEGORY")]
    category: Option<String>,

    /// Filter by minimum default severity.
    #[arg(long, value_name = "LEVEL")]
    min_severity: Option<Severity>,

    /// Case-insensitive search over rule id, title, description, and category.
    #[arg(long, value_name = "TEXT")]
    search: Option<String>,
}

#[derive(Debug, Args)]
struct ExplainArgs {
    /// Rule id (for example SS-001).
    rule: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = FormatArg::Terminal)]
    format: FormatArg,
}

#[derive(Debug, Args)]
struct InitArgs {
    /// Directory in which to create the configuration file.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Overwrite an existing configuration file.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Print the effective configuration as TOML.
    Show {
        /// Configuration file to load.
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    /// Validate a configuration file.
    Validate {
        /// Configuration file to validate.
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

/// A CLI error, distinguishing usage errors from runtime errors.
#[derive(Debug)]
enum CliError {
    Usage(String),
    Runtime(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Usage(message) | CliError::Runtime(message) => f.write_str(message),
        }
    }
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            CliError::Usage(_) => EXIT_USAGE,
            CliError::Runtime(_) => EXIT_ERROR,
        }
    }
}

type CliResult<T> = Result<T, CliError>;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: Cli) -> CliResult<u8> {
    let registry = RuleRegistry::with_default_rules();
    match cli.command {
        Command::Version => {
            println!("{}", soroban_scan_core::version_string());
            Ok(EXIT_SUCCESS)
        }
        Command::Scan(args) => run_scan(args, &registry),
        Command::Rules(args) => run_rules(args, &registry),
        Command::Explain(args) => run_explain(args, &registry),
        Command::Init(args) => run_init(args),
        Command::Config(args) => run_config(args),
    }
}

fn run_scan(args: ScanArgs, registry: &RuleRegistry) -> CliResult<u8> {
    if !args.path.exists() {
        return Err(CliError::Runtime(format!(
            "path does not exist: {}",
            args.path.display()
        )));
    }

    let mut config = load_config(args.config.as_deref(), &args.path)?;

    if let Some(severity) = args.min_severity {
        config.min_severity = severity;
    }
    if let Some(confidence) = args.min_confidence {
        config.min_confidence = confidence;
    }
    config.exclude.extend(args.exclude.iter().cloned());
    config.rules.disabled.extend(args.disable.iter().cloned());
    config.rules.enabled.extend(args.rule.iter().cloned());

    validate_rule_ids(&config, registry)?;

    let mut outcome = if let Some(list) = &args.files_from {
        if args.path.is_file() {
            return Err(CliError::Usage(
                "--files-from requires PATH to be a directory".to_string(),
            ));
        }
        let files = read_file_list(list, &args.path)?;
        let loaded = project::load_files(&args.path, &files).map_err(runtime)?;
        engine::scan(&loaded, registry, &config)
    } else if args.path.is_file() {
        let file = args.path.clone();
        let root = file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let loaded = project::load_files(&root, &[file]).map_err(runtime)?;
        engine::scan(&loaded, registry, &config)
    } else {
        engine::run_scan(&args.path, &config, registry).map_err(runtime)?
    };

    if let Some(path) = &args.write_baseline {
        let baseline = Baseline::from_findings(&outcome.findings);
        let text = baseline.to_json().map_err(runtime)?;
        std::fs::write(path, text)
            .map_err(|e| CliError::Runtime(format!("cannot write {}: {e}", path.display())))?;
        if !args.quiet {
            eprintln!(
                "wrote baseline with {} finding(s) to {}",
                baseline.findings.len(),
                path.display()
            );
        }
        return Ok(EXIT_SUCCESS);
    }

    let mut new_indices: Option<Vec<usize>> = None;
    if let Some(path) = &args.baseline {
        let text = std::fs::read_to_string(path).map_err(|e| {
            CliError::Runtime(format!("cannot read baseline {}: {e}", path.display()))
        })?;
        let baseline = Baseline::from_json(&text).map_err(runtime)?;
        let comparison = baseline::compare(&outcome.findings, &baseline);
        let resolved = if args.incremental {
            0
        } else {
            comparison.resolved_count()
        };
        if !args.quiet {
            eprintln!(
                "baseline: {} new, {} existing, {} resolved{}",
                comparison.new_count(),
                comparison.existing_count(),
                resolved,
                if args.incremental {
                    " (incremental)"
                } else {
                    ""
                }
            );
        }
        new_indices = Some(comparison.new);
    }

    // The failure gate only considers new findings when a baseline is supplied.
    let threshold = args.fail_on.threshold();
    let failed = match threshold {
        None => false,
        Some(threshold) => match &new_indices {
            Some(indices) => indices
                .iter()
                .any(|&index| outcome.findings[index].severity.meets(threshold)),
            None => outcome
                .findings
                .iter()
                .any(|finding| finding.severity.meets(threshold)),
        },
    };

    if args.new_only {
        if let Some(indices) = &new_indices {
            let keep: std::collections::BTreeSet<usize> = indices.iter().copied().collect();
            outcome.findings = outcome
                .findings
                .into_iter()
                .enumerate()
                .filter(|(index, _)| keep.contains(index))
                .map(|(_, finding)| finding)
                .collect();
        }
    }

    let rendered = if args.quiet && matches!(args.format, FormatArg::Terminal) {
        report::render_terminal_compact(&outcome)
    } else {
        report::render(&outcome, registry, args.format.into())
    };

    match &args.output {
        Some(path) => {
            std::fs::write(path, rendered)
                .map_err(|e| CliError::Runtime(format!("cannot write {}: {e}", path.display())))?;
            if !args.quiet {
                eprintln!("report written to {}", path.display());
            }
        }
        None => print!("{rendered}"),
    }

    if args.verbose && !args.quiet {
        for diagnostic in &outcome.diagnostics {
            eprintln!("diagnostic: {}", diagnostic.message);
        }
    }

    Ok(if failed { EXIT_FINDINGS } else { EXIT_SUCCESS })
}

fn run_rules(args: RulesArgs, registry: &RuleRegistry) -> CliResult<u8> {
    let category = match &args.category {
        Some(value) => Some(value.parse::<Category>().map_err(CliError::Usage)?),
        None => None,
    };

    let search = args.search.as_ref().map(|s| s.to_ascii_lowercase());

    let metas: Vec<_> = registry
        .metadata_sorted()
        .into_iter()
        .filter(|meta| match category {
            Some(c) => meta.category == c,
            None => true,
        })
        .filter(|meta| match args.min_severity {
            Some(min) => meta.default_severity.meets(min),
            None => true,
        })
        .filter(|meta| match &search {
            Some(query) => {
                meta.id.to_ascii_lowercase().contains(query)
                    || meta.title.to_ascii_lowercase().contains(query)
                    || meta.description.to_ascii_lowercase().contains(query)
                    || meta.category.as_str().contains(query.as_str())
            }
            None => true,
        })
        .collect();

    print!("{}", report::render_rule_list(&metas, args.format.into()));
    Ok(EXIT_SUCCESS)
}

fn run_explain(args: ExplainArgs, registry: &RuleRegistry) -> CliResult<u8> {
    let rule = registry
        .by_id(&args.rule)
        .ok_or_else(|| CliError::Usage(format!("unknown rule id: {}", args.rule)))?;
    print!(
        "{}",
        report::render_rule_detail(rule.metadata(), args.format.into())
    );
    Ok(EXIT_SUCCESS)
}

fn run_init(args: InitArgs) -> CliResult<u8> {
    if !args.path.exists() {
        return Err(CliError::Runtime(format!(
            "directory does not exist: {}",
            args.path.display()
        )));
    }
    let target = if args.path.is_dir() {
        args.path.join(DEFAULT_CONFIG_FILE)
    } else {
        args.path.clone()
    };
    if target.exists() && !args.force {
        return Err(CliError::Usage(format!(
            "{} already exists (use --force to overwrite)",
            target.display()
        )));
    }
    std::fs::write(&target, CONFIG_TEMPLATE)
        .map_err(|e| CliError::Runtime(format!("cannot write {}: {e}", target.display())))?;
    println!("wrote {}", target.display());
    Ok(EXIT_SUCCESS)
}

fn run_config(args: ConfigArgs) -> CliResult<u8> {
    match args.command {
        ConfigCommand::Show { config } => {
            let effective = load_config(config.as_deref(), Path::new("."))?;
            let text = toml::to_string_pretty(&effective)
                .map_err(|e| CliError::Runtime(format!("cannot serialize config: {e}")))?;
            print!("{text}");
            Ok(EXIT_SUCCESS)
        }
        ConfigCommand::Validate { config } => {
            let effective = load_config(config.as_deref(), Path::new("."))?;
            let registry = RuleRegistry::with_default_rules();
            validate_rule_ids(&effective, &registry)?;
            println!(
                "configuration is valid (min severity: {}, min confidence: {})",
                effective.min_severity, effective.min_confidence
            );
            Ok(EXIT_SUCCESS)
        }
    }
}

fn load_config(explicit: Option<&Path>, root: &Path) -> CliResult<ScanConfig> {
    if let Some(path) = explicit {
        let text = std::fs::read_to_string(path)
            .map_err(|e| CliError::Runtime(format!("cannot read {}: {e}", path.display())))?;
        return ScanConfig::from_toml(&text).map_err(|e| CliError::Usage(e.to_string()));
    }

    let candidates = [
        root.join(DEFAULT_CONFIG_FILE),
        PathBuf::from(DEFAULT_CONFIG_FILE),
    ];
    for candidate in candidates {
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate).map_err(|e| {
                CliError::Runtime(format!("cannot read {}: {e}", candidate.display()))
            })?;
            return ScanConfig::from_toml(&text).map_err(|e| CliError::Usage(e.to_string()));
        }
    }

    Ok(ScanConfig::default())
}

fn read_file_list(list: &Path, root: &Path) -> CliResult<Vec<PathBuf>> {
    let text = std::fs::read_to_string(list)
        .map_err(|e| CliError::Runtime(format!("cannot read file list {}: {e}", list.display())))?;
    let mut files = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let candidate = Path::new(line);
        if candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(CliError::Usage(format!(
                "file list entry escapes the scan root: {line}"
            )));
        }
        let resolved = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            root.join(candidate)
        };
        files.push(resolved);
    }
    if files.is_empty() {
        return Err(CliError::Usage("file list is empty".to_string()));
    }
    Ok(files)
}

fn validate_rule_ids(config: &ScanConfig, registry: &RuleRegistry) -> CliResult<()> {
    let mut referenced: Vec<&String> = Vec::new();
    referenced.extend(&config.rules.disabled);
    referenced.extend(&config.rules.enabled);
    for id in referenced {
        if registry.by_id(id).is_none() {
            return Err(CliError::Usage(format!("unknown rule id: {id}")));
        }
    }
    for id in config
        .rules
        .severity_overrides
        .keys()
        .chain(config.rules.confidence_overrides.keys())
    {
        if registry.by_id(id).is_none() {
            return Err(CliError::Usage(format!("unknown rule id: {id}")));
        }
    }
    Ok(())
}

fn runtime(error: soroban_scan_core::error::ScanError) -> CliError {
    CliError::Runtime(error.to_string())
}
