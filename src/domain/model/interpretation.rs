use crate::domain::ports::{RetrievalMode, RetrievalStrategy};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InterpretationContext {
    pub summary: String,
    pub documents: Vec<InterpretationDocument>,
    pub tool_hints: Vec<InterpretationToolHint>,
    pub decision_framework: InterpretationDecisionFramework,
    #[serde(default)]
    pub precedence_chain: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<InterpretationConflict>,
    #[serde(default)]
    pub coverage_confidence: InterpretationCoverageConfidence,
}

impl InterpretationContext {
    pub fn is_empty(&self) -> bool {
        self.summary.trim().is_empty()
            && self.documents.is_empty()
            && self.tool_hints.is_empty()
            && self.decision_framework.procedures.is_empty()
    }

    pub fn sources(&self) -> Vec<String> {
        let mut sources = self
            .documents
            .iter()
            .map(|document| document.source.clone())
            .collect::<Vec<_>>();
        for hint in &self.tool_hints {
            if !sources.contains(&hint.source) {
                sources.push(hint.source.clone());
            }
        }
        for procedure in &self.decision_framework.procedures {
            if !sources.contains(&procedure.source) {
                sources.push(procedure.source.clone());
            }
        }
        sources
    }

    pub fn render(&self) -> String {
        if self.is_empty() {
            return "No operator interpretation context was assembled.".to_string();
        }

        let mut sections = vec![self.summary.trim().to_string()];

        if !self.precedence_chain.is_empty() {
            sections.push(format!(
                "--- Precedence Chain ---\n{}",
                self.precedence_chain.join(" > ")
            ));
        }

        for document in &self.documents {
            sections.push(format!(
                "--- {} [{:?}] ---\n{}",
                document.source,
                document.category,
                document.excerpt.trim()
            ));
        }
        if !self.tool_hints.is_empty() {
            sections.push("--- Tool Hints ---".to_string());
            sections.extend(self.tool_hints.iter().map(|hint| {
                format!(
                    "- {} ({}) — {}",
                    hint.action.summary(),
                    hint.source,
                    hint.note
                )
            }));
        }
        if !self.decision_framework.procedures.is_empty() {
            sections.push("--- Decision Framework ---".to_string());
            sections.extend(self.decision_framework.procedures.iter().map(|procedure| {
                format!(
                    "- {} ({}) — {} [{} step(s)]",
                    procedure.label,
                    procedure.source,
                    procedure.purpose,
                    procedure.steps.len()
                )
            }));
        }

        if !self.conflicts.is_empty() {
            sections.push("--- Guidance Conflicts ---".to_string());
            sections.extend(self.conflicts.iter().map(|conflict| {
                format!(
                    "- {} vs {}: {}\n  Resolution: {}",
                    conflict.source_a, conflict.source_b, conflict.description, conflict.resolution
                )
            }));
        }

        sections.push(format!(
            "--- Coverage Confidence: {:?} ---",
            self.coverage_confidence
        ));

        sections.join("\n\n")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuidanceCategory {
    Rule,
    Convention,
    Constraint,
    Procedure,
    Preference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpretationConflict {
    pub source_a: String,
    pub source_b: String,
    pub description: String,
    pub resolution: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationCoverageConfidence {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpretationDocument {
    pub source: String,
    pub excerpt: String,
    pub category: GuidanceCategory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpretationToolHint {
    pub source: String,
    pub action: WorkspaceAction,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InterpretationDecisionFramework {
    pub procedures: Vec<InterpretationProcedure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpretationProcedure {
    pub source: String,
    pub label: String,
    pub purpose: String,
    pub steps: Vec<InterpretationProcedureStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpretationProcedureStep {
    pub index: usize,
    pub action: WorkspaceAction,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WorkspaceAction {
    Search {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
        #[serde(default)]
        intent: Option<String>,
    },
    ListFiles {
        #[serde(default)]
        pattern: Option<String>,
    },
    Read {
        path: String,
    },
    Inspect {
        command: String,
    },
    Shell {
        command: String,
    },
    Diff {
        #[serde(default)]
        path: Option<String>,
    },
    WriteFile {
        path: String,
        content: String,
    },
    ReplaceInFile {
        path: String,
        old: String,
        new: String,
        #[serde(default)]
        replace_all: bool,
    },
    ApplyPatch {
        patch: String,
    },
}

impl WorkspaceAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Search { .. } => "search",
            Self::ListFiles { .. } => "list_files",
            Self::Read { .. } => "read",
            Self::Inspect { .. } => "inspect",
            Self::Shell { .. } => "shell",
            Self::Diff { .. } => "diff",
            Self::WriteFile { .. } => "write_file",
            Self::ReplaceInFile { .. } => "replace_in_file",
            Self::ApplyPatch { .. } => "apply_patch",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Search {
                query,
                mode,
                strategy,
                ..
            } => format!("search `{query}` [{} / {}]", mode.label(), strategy.label()),
            Self::ListFiles { pattern } => match pattern {
                Some(pattern) if !pattern.trim().is_empty() => {
                    format!("list files matching `{pattern}`")
                }
                _ => "list files".to_string(),
            },
            Self::Read { path } => format!("read `{path}`"),
            Self::Inspect { command } => format!("inspect `{command}`"),
            Self::Shell { command } => command.clone(),
            Self::Diff { path } => match path {
                Some(path) if !path.trim().is_empty() => format!("diff `{path}`"),
                _ => "git diff --no-ext-diff".to_string(),
            },
            Self::WriteFile { path, .. } => format!("write `{path}`"),
            Self::ReplaceInFile { path, .. } => format!("replace text in `{path}`"),
            Self::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
        }
    }

    /// Human-readable description of this action for event and trace output.
    pub fn describe(&self) -> String {
        match self {
            Self::Search { query, intent, .. } => match intent {
                Some(intent) => format!("search workspace for `{query}` ({intent})"),
                None => format!("search workspace for `{query}`"),
            },
            Self::ListFiles { pattern } => match pattern {
                Some(pattern) if !pattern.trim().is_empty() => {
                    format!("list files matching `{pattern}`")
                }
                _ => "list workspace files".to_string(),
            },
            Self::Read { path } => format!("read `{path}`"),
            Self::Inspect { command } => format!("inspect `{command}`"),
            Self::Shell { command } => command.clone(),
            Self::Diff { path } => match path {
                Some(path) if !path.trim().is_empty() => {
                    format!("git diff --no-ext-diff -- {path}")
                }
                _ => "git diff --no-ext-diff".to_string(),
            },
            Self::WriteFile { path, .. } => format!("write `{path}`"),
            Self::ReplaceInFile { path, .. } => format!("replace text in `{path}`"),
            Self::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
        }
    }
}
