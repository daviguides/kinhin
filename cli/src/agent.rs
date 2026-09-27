use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use claude_agent_toolkit::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, PermissionMode, SystemPrompt,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::detect::Language;

const TAGGER_SYSTEM_PROMPT: &str = r#"You are a TDD lifecycle tagger. Given a test file, classify each test function by its authority:

- @scaffold: no authority, construction-time only. DEFAULT for any test you're unsure about.
- @decision(ref=): encodes a non-obvious decision. Ref must point to documentation.
- @contract(ref=): pins a boundary another system depends on. Ref names the consumer.
- @incident(ref=): reproduces a production failure. Ref is the ticket/incident.

Rules:
- Tests asserting mock call counts, argument order, or implementation shape → scaffold
- Property tests / invariant tests → decision (invariants are decisions)
- Tests crossing system boundaries → contract
- Burden of proof is on KEEPING, not deleting
- When unsure → scaffold

Respond ONLY with a JSON array. No markdown, no explanation, just the array:
[{"test": "test_name", "tag": "scaffold|decision|contract|incident", "reason": "one line", "ref": "path or ticket or null"}]"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSuggestion {
    pub test: String,
    pub tag: String,
    pub reason: String,
    #[serde(rename = "ref")]
    pub ref_value: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionEvent {
    #[serde(rename = "type")]
    event_type: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(flatten)]
    payload: serde_json::Value,
}

pub struct SessionMetrics {
    pub files_processed: u32,
    pub total_cost_usd: f64,
    pub total_turns: u32,
    pub session_id: Option<String>,
}

pub struct TaggerSession {
    client: ClaudeClient,
    language: Language,
    cumulative_cost: f64,
    total_turns: u32,
    files_processed: u32,
    session_id: Option<String>,
    log_path: Option<PathBuf>,
}

impl TaggerSession {
    pub async fn connect(language: Language, model: Option<&str>) -> anyhow::Result<Self> {
        let sandbox = tempfile::tempdir()?;

        let mut builder = ClaudeAgentOptions::builder()
            .cwd(sandbox.path())
            .permission_mode(PermissionMode::BypassPermissions)
            .system_prompt(SystemPrompt::Preset {
                preset: "claude_code".to_string(),
                append: Some(TAGGER_SYSTEM_PROMPT.to_string()),
                exclude_dynamic_sections: None,
            });

        if let Some(m) = model {
            builder = builder.model(m);
        }

        let options = builder.build();

        let client = ClaudeClient::connect(options).await?;

        let log_path = Self::init_log_file();

        let session = Self {
            client,
            language,
            cumulative_cost: 0.0,
            total_turns: 0,
            files_processed: 0,
            session_id: None,
            log_path,
        };

        session.emit_event("session_start", None, serde_json::json!({
            "language": language.to_string(),
        }));

        Ok(session)
    }

    pub async fn tag_file(&mut self, content: &str) -> anyhow::Result<Vec<TagSuggestion>> {
        let file_label = content.lines().next().unwrap_or("(unknown)").to_string();

        self.emit_event("file_start", Some(&file_label), serde_json::json!({}));

        let prompt = format!(
            "Language: {}\n\nClassify each test in this file:\n\n```\n{content}\n```",
            self.language
        );

        let cost_before = self.cumulative_cost;
        self.client.send(&prompt).await?;

        let mut text_blocks = Vec::new();
        let mut tool_calls = Vec::new();

        {
            let mut responses = self.client.receive_response()?;
            while let Some(message) = responses.next().await {
                match message? {
                    Message::Assistant(assistant) => {
                        for block in assistant.content {
                            match block {
                                ContentBlock::Text { text } => text_blocks.push(text),
                                ContentBlock::ToolUse { name, .. } => {
                                    tool_calls.push(name);
                                }
                                _ => {}
                            }
                        }
                    }
                    Message::Result(result) => {
                        let cost = result.total_cost_usd.unwrap_or(0.0);
                        let query_cost = cost - cost_before;
                        self.cumulative_cost = cost;
                        self.total_turns += result.num_turns;
                        self.files_processed += 1;

                        if self.session_id.is_none() {
                            self.session_id = Some(result.session_id.clone());
                        }

                        self.emit_event("file_complete", Some(&file_label), serde_json::json!({
                            "turns": result.num_turns,
                            "cost_usd": format!("{query_cost:.4}"),
                            "tool_calls": tool_calls,
                            "is_error": result.is_error,
                        }));

                        break;
                    }
                    _ => {}
                }
            }
        }

        let full_text = text_blocks.join("\n");
        let suggestions = parse_suggestions(&full_text)?;

        self.emit_event("file_tagged", Some(&file_label), serde_json::json!({
            "tests_classified": suggestions.len(),
            "scaffold": suggestions.iter().filter(|s| s.tag == "scaffold").count(),
            "permanent": suggestions.iter().filter(|s| s.tag != "scaffold").count(),
        }));

        Ok(suggestions)
    }

    pub fn metrics(&self) -> SessionMetrics {
        SessionMetrics {
            files_processed: self.files_processed,
            total_cost_usd: self.cumulative_cost,
            total_turns: self.total_turns,
            session_id: self.session_id.clone(),
        }
    }

    pub async fn disconnect(mut self) -> anyhow::Result<SessionMetrics> {
        let metrics = self.metrics();

        self.emit_event("session_end", None, serde_json::json!({
            "files_processed": metrics.files_processed,
            "total_cost_usd": format!("{:.4}", metrics.total_cost_usd),
            "total_turns": metrics.total_turns,
            "session_id": metrics.session_id,
        }));

        self.client.disconnect().await?;

        if let Some(path) = &self.log_path {
            eprintln!("  session log: {}", path.display());
        }

        Ok(metrics)
    }

    fn init_log_file() -> Option<PathBuf> {
        let dir = Path::new(".kinhin");
        std::fs::create_dir_all(dir).ok()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let path = dir.join(format!("session-{timestamp}.jsonl"));
        Some(path)
    }

    fn emit_event(&self, event_type: &str, file: Option<&str>, payload: serde_json::Value) {
        let event = SessionEvent {
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            file: file.map(|f| f.to_string()),
            payload,
        };

        let json = match serde_json::to_string(&event) {
            Ok(j) => j,
            Err(_) => return,
        };

        if let Some(path) = &self.log_path {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(f, "{json}");
            }
        }
    }
}

fn parse_suggestions(text: &str) -> anyhow::Result<Vec<TagSuggestion>> {
    let trimmed = text.trim();

    if let Ok(suggestions) = serde_json::from_str::<Vec<TagSuggestion>>(trimmed) {
        return Ok(suggestions);
    }

    let json_start = trimmed.find('[');
    let json_end = trimmed.rfind(']');

    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_slice = &trimmed[start..=end];
        if let Ok(suggestions) = serde_json::from_str::<Vec<TagSuggestion>>(json_slice) {
            return Ok(suggestions);
        }
    }

    anyhow::bail!("failed to parse tag suggestions from model response:\n{trimmed}")
}
