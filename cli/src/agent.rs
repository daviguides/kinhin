//! Agent session used by `kinhin tag`: one Claude session for the whole
//! run, one turn per test file, every step logged as NDJSON.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use claude_agent_toolkit::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, PermissionMode, SettingSource, SystemPrompt, ThinkingConfig,
    ToolsOption,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::detect::Language;

const TAGGER_SYSTEM_PROMPT: &str = r#"You are a TDD lifecycle tagger. You receive one test file and the list of test ids in it. Decide, for every id, whether the test has an authority outside the code that justifies keeping it forever.

Tags:
- scaffold: the DEFAULT. The test helped build the code and has no authority of its own. Typical scaffolds: happy-path round trips (add then list, save then load), getters, "returns the expected value" for the obvious case, "does not raise", a second or third sample input for a rule another test already covers, tests of mock call counts, argument order or internal structure.
- decision: the test pins a rule somebody CHOSE, where a competent implementer could reasonably have done otherwise: precedence and tie-breaking, ordering, what is preserved versus normalised, case sensitivity, idempotence, limits and boundaries, error versus silent skip, invariants across operations. "reason" states that rule in one line.
- contract: the test pins something a consumer OUTSIDE this code reads or writes: an on-disk or exported file format, CLI output text or exit codes, an imported third-party format, a wire or public API shape. "ref" names that consumer. Calling an internal function is not a contract.
- incident: the test reproduces a production failure. Only when the file itself cites a ticket, issue or bug id; "ref" is that id.

How to decide:
- The burden of proof is on KEEPING a test. If you cannot state the chosen rule or name the outside consumer in one concrete line, it is scaffold.
- When several tests exercise the same rule, the one that states the rule most directly is the decision; the others are scaffold.
- In an ordinary suite most tests are scaffold. If you are about to keep more than half of a file, re-check each one against the definitions.
- Return every id you were given, exactly as written, once.
- Do not use tools. Answer from the file content only.

Respond with a JSON array and nothing else:
[{"test": "<id>", "tag": "scaffold|decision|contract|incident", "reason": "<one line>", "ref": "<consumer or ticket, or null>"}]"#;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagSuggestion {
    pub test: String,
    pub tag: String,
    #[serde(default)]
    pub reason: String,
    #[serde(rename = "ref", default)]
    pub ref_value: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionEvent<'a> {
    #[serde(rename = "type")]
    event_type: &'a str,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<&'a str>,
    #[serde(flatten)]
    payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct SessionMetrics {
    pub files_processed: u32,
    pub total_cost_usd: f64,
    pub total_turns: u32,
    pub log_path: Option<PathBuf>,
}

pub struct TaggerSession {
    client: ClaudeClient,
    language: Language,
    cumulative_cost: f64,
    total_turns: u32,
    files_processed: u32,
    session_id: Option<String>,
    log_path: Option<PathBuf>,
    // Kept alive for the session: the agent's working directory.
    _sandbox: tempfile::TempDir,
}

impl TaggerSession {
    /// `log_dir` is the project's `.kinhin/` directory.
    pub async fn connect(language: Language, model: &str, log_dir: &Path) -> anyhow::Result<Self> {
        let sandbox = tempfile::tempdir()?;

        // No tools and no user/project settings: the session only reads the
        // text it is sent, and the user's own hooks do not leak into it.
        let options = ClaudeAgentOptions::builder()
            .cwd(sandbox.path())
            .permission_mode(PermissionMode::BypassPermissions)
            .system_prompt(SystemPrompt::Custom(TAGGER_SYSTEM_PROMPT.to_string()))
            .tools(ToolsOption::Named(Vec::new()))
            .setting_sources(Vec::<SettingSource>::new())
            .thinking(ThinkingConfig::Disabled)
            .model(model)
            .build();

        let client = ClaudeClient::connect(options).await?;

        let session = Self {
            client,
            language,
            cumulative_cost: 0.0,
            total_turns: 0,
            files_processed: 0,
            session_id: None,
            log_path: init_log_file(log_dir),
            _sandbox: sandbox,
        };
        session.emit("session_start", None, serde_json::json!({
            "language": language.to_string(),
            "model": model,
        }));
        Ok(session)
    }

    /// Classify the tests `ids` of one file. Every id comes back exactly
    /// once: ids the model skipped or mangled default to scaffold.
    pub async fn tag_file(&mut self, file: &str, content: &str, ids: &[String]) -> anyhow::Result<Vec<TagSuggestion>> {
        self.emit("file_start", Some(file), serde_json::json!({ "tests": ids.len() }));

        let prompt = format!(
            "Language: {}\nFile: {file}\n\nTest ids to classify:\n{}\n\nFile content:\n```\n{content}\n```",
            self.language,
            ids.iter().map(|id| format!("- {id}")).collect::<Vec<_>>().join("\n"),
        );

        let cost_before = self.cumulative_cost;
        self.client.send(&prompt).await?;

        let mut text_blocks = Vec::new();
        let mut is_error = false;
        {
            let mut responses = self.client.receive_response()?;
            while let Some(message) = responses.next().await {
                match message? {
                    Message::Assistant(assistant) => {
                        for block in assistant.content {
                            if let ContentBlock::Text { text } = block {
                                text_blocks.push(text);
                            }
                        }
                    }
                    Message::Result(result) => {
                        let cost = result.total_cost_usd.unwrap_or(self.cumulative_cost);
                        self.cumulative_cost = cost;
                        self.total_turns += result.num_turns;
                        self.files_processed += 1;
                        is_error = result.is_error;
                        if self.session_id.is_none() {
                            self.session_id = Some(result.session_id.clone());
                        }
                        self.emit("file_complete", Some(file), serde_json::json!({
                            "turns": result.num_turns,
                            "cost_usd": cost - cost_before,
                            "duration_ms": result.duration_ms,
                            "is_error": result.is_error,
                            "session_id": result.session_id,
                        }));
                        break;
                    }
                    _ => {}
                }
            }
        }

        let full_text = text_blocks.join("\n");
        if is_error {
            self.emit("file_error", Some(file), serde_json::json!({ "response": full_text }));
            anyhow::bail!("the session reported an error: {}", full_text.lines().next().unwrap_or("(no text)"));
        }
        let parsed = match parse_suggestions(&full_text) {
            Ok(parsed) => parsed,
            Err(e) => {
                self.emit("file_error", Some(file), serde_json::json!({ "response": full_text }));
                return Err(e);
            }
        };
        let (suggestions, defaulted) = reconcile(ids, parsed);

        self.emit("file_tagged", Some(file), serde_json::json!({
            "tests_classified": suggestions.len(),
            "scaffold": suggestions.iter().filter(|s| s.tag == "scaffold").count(),
            "permanent": suggestions.iter().filter(|s| s.tag != "scaffold").count(),
            "defaulted_to_scaffold": defaulted,
        }));
        Ok(suggestions)
    }

    pub async fn disconnect(mut self) -> anyhow::Result<SessionMetrics> {
        let metrics = SessionMetrics {
            files_processed: self.files_processed,
            total_cost_usd: self.cumulative_cost,
            total_turns: self.total_turns,
            log_path: self.log_path.clone(),
        };
        self.emit("session_end", None, serde_json::json!({
            "files_processed": metrics.files_processed,
            "total_cost_usd": metrics.total_cost_usd,
            "total_turns": metrics.total_turns,
            "session_id": self.session_id,
        }));
        self.client.disconnect().await?;
        Ok(metrics)
    }

    fn emit(&self, event_type: &str, file: Option<&str>, payload: serde_json::Value) {
        let Some(path) = &self.log_path else {
            return;
        };
        let event = SessionEvent {
            event_type,
            timestamp: chrono::Utc::now().to_rfc3339(),
            file,
            payload,
        };
        if let (Ok(json), Ok(mut log)) = (
            serde_json::to_string(&event),
            OpenOptions::new().create(true).append(true).open(path),
        ) {
            let _ = writeln!(log, "{json}");
        }
    }
}

fn init_log_file(log_dir: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(log_dir).ok()?;
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f");
    Some(log_dir.join(format!("session-{timestamp}.jsonl")))
}

const VALID_TAGS: [&str; 4] = ["scaffold", "decision", "contract", "incident"];

/// Keep exactly one suggestion per requested id, in request order.
/// Returns the suggestions and how many ids had to default to scaffold.
pub fn reconcile(ids: &[String], parsed: Vec<TagSuggestion>) -> (Vec<TagSuggestion>, usize) {
    let mut defaulted = 0;
    let suggestions = ids
        .iter()
        .map(|id| {
            let found = parsed
                .iter()
                .find(|s| s.test == *id)
                .filter(|s| VALID_TAGS.contains(&s.tag.as_str()));
            match found {
                Some(suggestion) => suggestion.clone(),
                None => {
                    defaulted += 1;
                    TagSuggestion {
                        test: id.clone(),
                        tag: "scaffold".to_string(),
                        reason: "not classified by the model: default".to_string(),
                        ref_value: None,
                    }
                }
            }
        })
        .collect();
    (suggestions, defaulted)
}

pub fn parse_suggestions(text: &str) -> anyhow::Result<Vec<TagSuggestion>> {
    let trimmed = text.trim();
    if let Ok(suggestions) = serde_json::from_str::<Vec<TagSuggestion>>(trimmed) {
        return Ok(suggestions);
    }
    if let (Some(start), Some(end)) = (trimmed.find('['), trimmed.rfind(']')) {
        if start < end {
            if let Ok(suggestions) = serde_json::from_str::<Vec<TagSuggestion>>(&trimmed[start..=end]) {
                return Ok(suggestions);
            }
        }
    }
    let preview: String = trimmed.chars().take(300).collect();
    anyhow::bail!("the model did not answer with a JSON array of suggestions: {preview}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestion(test: &str, tag: &str) -> TagSuggestion {
        TagSuggestion {
            test: test.into(),
            tag: tag.into(),
            reason: "r".into(),
            ref_value: None,
        }
    }

    #[test]
    fn parses_an_array_wrapped_in_a_markdown_fence() {
        let text = "```json\n[{\"test\": \"TestA::test_x\", \"tag\": \"decision\", \"reason\": \"why\", \"ref\": null}]\n```";
        assert_eq!(parse_suggestions(text).unwrap()[0].test, "TestA::test_x");
    }

    #[test]
    fn prose_without_json_is_an_error() {
        assert!(parse_suggestions("I could not classify these tests.").is_err());
    }

    #[test]
    fn missing_and_invalid_answers_default_to_scaffold() {
        let ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let parsed = vec![suggestion("a", "decision"), suggestion("b", "keep"), suggestion("zzz", "contract")];
        let (out, defaulted) = reconcile(&ids, parsed);
        assert_eq!(out.iter().map(|s| s.tag.as_str()).collect::<Vec<_>>(), ["decision", "scaffold", "scaffold"]);
        assert_eq!(out.iter().map(|s| s.test.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
        assert_eq!(defaulted, 2);
    }
}
