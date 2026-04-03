use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONVERSATION_HISTORY_FILE: &str = "conversation-history.toml";
const MAX_PROMPT_HISTORY: usize = 200;
const MAX_RECENT_TURN_SUMMARIES: usize = 12;
const PROMPT_SUMMARY_LIMIT: usize = 120;
const REPLY_SUMMARY_LIMIT: usize = 160;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversationHistory {
    pub prompt_history: Vec<String>,
    pub recent_turn_summaries: Vec<String>,
}

impl ConversationHistory {
    fn record_prompt(&mut self, prompt: &str) {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return;
        }
        self.prompt_history.push(prompt.to_string());
        trim_to_recent(&mut self.prompt_history, MAX_PROMPT_HISTORY);
    }

    fn record_turn(&mut self, prompt: &str, reply: &str) {
        let summary = format!(
            "Q: {} A: {}",
            trim_for_history(prompt, PROMPT_SUMMARY_LIMIT),
            trim_for_history(reply, REPLY_SUMMARY_LIMIT)
        );
        self.recent_turn_summaries.push(summary);
        trim_to_recent(&mut self.recent_turn_summaries, MAX_RECENT_TURN_SUMMARIES);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationHistoryStore {
    path: PathBuf,
}

impl Default for ConversationHistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationHistoryStore {
    pub fn new() -> Self {
        Self::with_path(default_conversation_history_path())
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<ConversationHistory> {
        if !self.path.exists() {
            return Ok(ConversationHistory::default());
        }

        let contents = fs::read_to_string(&self.path)
            .with_context(|| format!("read conversation history from {}", self.path.display()))?;
        toml::from_str::<ConversationHistory>(&contents)
            .with_context(|| format!("parse conversation history from {}", self.path.display()))
    }

    pub fn save(&self, history: &ConversationHistory) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("create conversation history directory {}", parent.display())
            })?;
        }

        let contents =
            toml::to_string(history).context("serialize conversation history as toml")?;
        fs::write(&self.path, contents)
            .with_context(|| format!("write conversation history to {}", self.path.display()))?;
        Ok(())
    }

    pub fn prompt_history(&self) -> Result<Vec<String>> {
        Ok(self.load()?.prompt_history)
    }

    pub fn recent_turn_summaries(&self) -> Result<Vec<String>> {
        Ok(self.load()?.recent_turn_summaries)
    }

    pub fn record_prompt(&self, prompt: &str) -> Result<()> {
        let mut history = self.load()?;
        history.record_prompt(prompt);
        self.save(&history)
    }

    pub fn record_turn(&self, prompt: &str, reply: &str) -> Result<()> {
        let mut history = self.load()?;
        history.record_turn(prompt, reply);
        self.save(&history)
    }
}

pub fn default_conversation_history_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "paddles")
        && let Some(state_dir) = project_dirs.state_dir()
    {
        return state_dir.join(CONVERSATION_HISTORY_FILE);
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("paddles")
            .join(CONVERSATION_HISTORY_FILE);
    }

    PathBuf::from(CONVERSATION_HISTORY_FILE)
}

fn trim_for_history(input: &str, limit: usize) -> String {
    let input = input.trim();
    if input.chars().count() <= limit {
        return input.to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{}...[truncated]", kept.trim_end())
}

fn trim_to_recent(entries: &mut Vec<String>, limit: usize) {
    if entries.len() > limit {
        let drop_count = entries.len() - limit;
        entries.drain(..drop_count);
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationHistoryStore, default_conversation_history_path};

    #[test]
    fn conversation_history_store_round_trips_prompts_and_recent_turns() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store =
            ConversationHistoryStore::with_path(dir.path().join("state/conversation-history.toml"));

        store
            .record_prompt("first prompt")
            .expect("record prompt history");
        store
            .record_turn("first prompt", "first reply")
            .expect("record recent turn");

        let loaded = store.load().expect("load conversation history");
        assert_eq!(loaded.prompt_history, vec!["first prompt".to_string()]);
        assert_eq!(
            loaded.recent_turn_summaries,
            vec!["Q: first prompt A: first reply".to_string()]
        );
    }

    #[test]
    fn default_conversation_history_path_targets_machine_state() {
        let path = default_conversation_history_path();
        assert!(path.ends_with("paddles/conversation-history.toml"));
    }
}
