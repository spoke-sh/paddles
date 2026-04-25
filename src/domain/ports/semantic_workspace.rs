use super::context_gathering::{RetrievalMode, RetrievalStrategy, RetrieverOption};
use crate::domain::model::{WorkspaceAction, WorkspaceTextPosition};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticWorkspaceOperation {
    Definitions,
    References,
    Symbols,
    Hover,
    Diagnostics,
}

impl SemanticWorkspaceOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Definitions => "definitions",
            Self::References => "references",
            Self::Symbols => "symbols",
            Self::Hover => "hover",
            Self::Diagnostics => "diagnostics",
        }
    }

    pub fn action_label(self) -> &'static str {
        match self {
            Self::Definitions => "semantic_definitions",
            Self::References => "semantic_references",
            Self::Symbols => "semantic_symbols",
            Self::Hover => "semantic_hover",
            Self::Diagnostics => "semantic_diagnostics",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticWorkspaceQuery {
    pub operation: SemanticWorkspaceOperation,
    pub path: Option<String>,
    pub position: Option<WorkspaceTextPosition>,
}

impl SemanticWorkspaceQuery {
    pub fn from_action(action: &WorkspaceAction) -> Option<Self> {
        match action {
            WorkspaceAction::SemanticDefinitions { path, position } => Some(Self {
                operation: SemanticWorkspaceOperation::Definitions,
                path: Some(path.clone()),
                position: Some(*position),
            }),
            WorkspaceAction::SemanticReferences { path, position } => Some(Self {
                operation: SemanticWorkspaceOperation::References,
                path: Some(path.clone()),
                position: Some(*position),
            }),
            WorkspaceAction::SemanticSymbols { path } => Some(Self {
                operation: SemanticWorkspaceOperation::Symbols,
                path: Some(path.clone()),
                position: None,
            }),
            WorkspaceAction::SemanticHover { path, position } => Some(Self {
                operation: SemanticWorkspaceOperation::Hover,
                path: Some(path.clone()),
                position: Some(*position),
            }),
            WorkspaceAction::SemanticDiagnostics { path } => Some(Self {
                operation: SemanticWorkspaceOperation::Diagnostics,
                path: path.clone(),
                position: None,
            }),
            _ => None,
        }
    }

    pub fn fallback_actions(&self) -> Vec<WorkspaceAction> {
        let mut actions = Vec::new();
        if let Some(path) = self.path.as_ref().filter(|path| !path.trim().is_empty()) {
            actions.push(WorkspaceAction::Read { path: path.clone() });
        }
        actions.push(WorkspaceAction::Search {
            query: self
                .path
                .as_ref()
                .filter(|path| !path.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| self.operation.label().to_string()),
            mode: RetrievalMode::Linear,
            strategy: RetrievalStrategy::Lexical,
            retrievers: vec![RetrieverOption::PathFuzzy, RetrieverOption::SegmentFuzzy],
            intent: Some(format!("semantic {} fallback", self.operation.label())),
        });
        actions
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticWorkspaceStatus {
    Available,
    Unavailable,
}

impl SemanticWorkspaceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticWorkspaceResult {
    pub status: SemanticWorkspaceStatus,
    pub summary: String,
    pub fallback_actions: Vec<WorkspaceAction>,
}

impl SemanticWorkspaceResult {
    pub fn unavailable(query: &SemanticWorkspaceQuery) -> Self {
        let fallback_actions = query.fallback_actions();
        let fallback_summary = fallback_actions
            .iter()
            .map(WorkspaceAction::summary)
            .collect::<Vec<_>>()
            .join("; ");
        Self {
            status: SemanticWorkspaceStatus::Unavailable,
            summary: format!(
                "semantic workspace unavailable for {}; fallback actions: {fallback_summary}",
                query.operation.label()
            ),
            fallback_actions,
        }
    }
}

pub trait SemanticWorkspacePort: Send + Sync + fmt::Debug {
    fn is_available(&self) -> bool;

    fn execute(&self, query: SemanticWorkspaceQuery) -> SemanticWorkspaceResult;
}
