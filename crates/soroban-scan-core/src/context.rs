//! Analysis context: the read-only view rules use during a scan.

use std::path::{Path, PathBuf};

use crate::model::Project;
use crate::source::{FunctionSummary, ParsedSource};

/// Immutable context passed to every rule.
///
/// The context is fixed for the duration of a scan, which keeps rule evaluation
/// deterministic and prevents rules from mutating shared state.
pub struct AnalysisContext<'a> {
    project: &'a Project,
    sources: &'a [ParsedSource],
    root: &'a Path,
}

impl<'a> AnalysisContext<'a> {
    /// Creates a context for a project and its parsed sources.
    pub fn new(project: &'a Project, sources: &'a [ParsedSource]) -> Self {
        AnalysisContext {
            project,
            sources,
            root: project.root.as_path(),
        }
    }

    /// The project metadata.
    pub fn project(&self) -> &'a Project {
        self.project
    }

    /// The scan root.
    pub fn root(&self) -> &'a Path {
        self.root
    }

    /// All parsed sources.
    pub fn sources(&self) -> &'a [ParsedSource] {
        self.sources
    }

    /// Whether the project is confirmed or likely Soroban.
    pub fn is_soroban(&self) -> bool {
        use crate::model::ProjectKind::*;
        matches!(self.project.kind, LikelySoroban | ConfirmedSoroban)
    }

    /// Whether the project is confirmed Soroban.
    pub fn is_confirmed_soroban(&self) -> bool {
        self.project.kind == crate::model::ProjectKind::ConfirmedSoroban
    }

    /// Returns the parsed source for a path, if present.
    pub fn source_for(&self, path: &Path) -> Option<&'a ParsedSource> {
        self.sources.iter().find(|s| s.path == path)
    }

    /// Converts a path to a scan-root-relative path when possible.
    pub fn relative_path(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(self.root) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => path.to_path_buf(),
        }
    }

    /// Iterates over all exported contract entry points.
    pub fn contract_entry_points(
        &self,
    ) -> impl Iterator<Item = (&'a ParsedSource, &'a FunctionSummary)> + '_ {
        self.sources.iter().flat_map(|source| {
            source
                .contract_entry_points()
                .map(move |func| (source, func))
        })
    }

    /// Iterates over every parsed source and every function it defines.
    pub fn all_functions(
        &self,
    ) -> impl Iterator<Item = (&'a ParsedSource, &'a FunctionSummary)> + '_ {
        self.sources
            .iter()
            .flat_map(|source| source.functions.iter().map(move |f| (source, f)))
    }

    /// The Soroban SDK version requirements discovered in manifests.
    pub fn soroban_sdk_versions(&self) -> &[String] {
        &self.project.soroban.sdk_versions
    }
}
