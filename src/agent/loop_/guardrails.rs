use regex::Regex;
use std::sync::LazyLock;

const TOOL_UNAVAILABLE_RETRY_PROMPT_PREFIX: &str = "Internal correction: your prior reply claimed required tools were unavailable. Use only the runtime-allowed tools listed below. If file changes are requested and `file_write`/`file_edit` are listed, call them directly.";

/// Detect completion claims that imply state-changing work already happened
/// without an accompanying tool call.
static ACTION_COMPLETION_CUE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)\b(done|completed?|finished|successfully|i(?:'ve|\s+have)|we(?:'ve|\s+have))\b",
    )
    .unwrap()
});

/// Verbs that usually imply side effects requiring tool execution.
static SIDE_EFFECT_ACTION_VERB_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)\b(create|created|write|wrote|run|ran|execute|executed|update|updated|delete|deleted|remove|removed|rename|renamed|move|moved|install|installed|save|saved|make|made)\b",
    )
    .unwrap()
});

/// Concrete artifacts often referenced in file/system action completion claims.
static SIDE_EFFECT_ACTION_OBJECT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)\b(file|files|folder|folders|directory|directories|workspace|cwd|current\s+working\s+directory|command|commands|script|scripts|path|paths)\b",
    )
    .unwrap()
});

/// Detect responses that incorrectly claim file tooling is unavailable even
/// when runtime policy allows file tools in this turn.
static TOOL_UNAVAILABLE_CLAIM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        \b(
            i\s+(?:do\s+not|don't)\s+have\s+access|
            i\s+(?:cannot|can't)\s+(?:access|use|perform|create|edit|write)|
            i\s+am\s+unable\s+to|
            no\s+(?:tool|tools|function|functions)\s+(?:available|access)
        )\b
        [^.!?\n]{0,220}
        \b(
            tool|tools|function|functions|file|file_write|file_edit|
            create|write|edit|delete
        )\b",
    )
    .unwrap()
});

pub(super) fn looks_like_unverified_action_completion_without_tool_call(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    ACTION_COMPLETION_CUE_REGEX.is_match(trimmed)
        && SIDE_EFFECT_ACTION_VERB_REGEX.is_match(trimmed)
        && SIDE_EFFECT_ACTION_OBJECT_REGEX.is_match(trimmed)
}

pub(super) fn looks_like_tool_unavailability_claim(
    text: &str,
    tool_specs: &[crate::tools::ToolSpec],
) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || !TOOL_UNAVAILABLE_CLAIM_REGEX.is_match(trimmed) {
        return false;
    }

    tool_specs
        .iter()
        .any(|spec| matches!(spec.name.as_str(), "file_write" | "file_edit"))
}

pub(super) fn build_tool_unavailable_retry_prompt(tool_specs: &[crate::tools::ToolSpec]) -> String {
    const MAX_TOOLS_IN_PROMPT: usize = 24;
    let tool_list = tool_specs
        .iter()
        .map(|spec| spec.name.as_str())
        .take(MAX_TOOLS_IN_PROMPT)
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "{TOOL_UNAVAILABLE_RETRY_PROMPT_PREFIX}\nRuntime tools: {tool_list}\nEmit the correct tool call now if tool use is required. Otherwise provide the final answer without claiming missing tools."
    )
}
