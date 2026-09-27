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

pub async fn tag_session(
    test_file_content: &str,
    language: Language,
) -> anyhow::Result<Vec<TagSuggestion>> {
    let sandbox = tempfile::tempdir()?;

    let options = ClaudeAgentOptions::builder()
        .cwd(sandbox.path())
        .permission_mode(PermissionMode::BypassPermissions)
        .system_prompt(SystemPrompt::Preset {
            preset: "claude_code".to_string(),
            append: Some(TAGGER_SYSTEM_PROMPT.to_string()),
            exclude_dynamic_sections: None,
        })
        .max_turns(1)
        .build();

    let mut client = ClaudeClient::connect(options).await?;

    let prompt = format!(
        "Language: {language}\n\nClassify each test in this file:\n\n```\n{test_file_content}\n```"
    );

    client.send(&prompt).await?;

    let mut text_blocks = Vec::new();

    {
        let mut responses = client.receive_response()?;
        while let Some(message) = responses.next().await {
            match message? {
                Message::Assistant(assistant) => {
                    for block in assistant.content {
                        if let ContentBlock::Text { text } = block {
                            text_blocks.push(text);
                        }
                    }
                }
                Message::Result(_) => break,
                _ => {}
            }
        }
    }

    client.disconnect().await?;

    let full_text = text_blocks.join("\n");
    parse_suggestions(&full_text)
}

fn parse_suggestions(text: &str) -> anyhow::Result<Vec<TagSuggestion>> {
    let trimmed = text.trim();

    // Try direct parse first
    if let Ok(suggestions) = serde_json::from_str::<Vec<TagSuggestion>>(trimmed) {
        return Ok(suggestions);
    }

    // Extract JSON array from markdown code block
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
