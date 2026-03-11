use super::ChannelRuntimeContext;
use crate::config::Config;
use crate::providers;
use crate::tools::{Tool, ToolSpec};

pub(super) fn resolve_provider_alias(name: &str) -> Option<String> {
    let candidate = name.trim();
    if candidate.is_empty() {
        return None;
    }

    let providers_list = providers::list_providers();
    for provider in providers_list {
        if provider.name.eq_ignore_ascii_case(candidate)
            || provider
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(candidate))
        {
            return Some(provider.name.to_string());
        }
    }

    None
}

pub(super) fn resolved_default_provider(config: &Config) -> String {
    config
        .default_provider
        .clone()
        .unwrap_or_else(|| "openrouter".to_string())
}

pub(super) fn resolved_default_model(config: &Config) -> String {
    config
        .default_model
        .clone()
        .unwrap_or_else(|| "anthropic/claude-sonnet-4.6".to_string())
}

pub(super) fn snapshot_non_cli_excluded_tools(ctx: &ChannelRuntimeContext) -> Vec<String> {
    ctx.non_cli_excluded_tools
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

pub(super) fn filtered_tool_specs_for_runtime(
    tools_registry: &[Box<dyn Tool>],
    excluded_tools: &[String],
) -> Vec<ToolSpec> {
    tools_registry
        .iter()
        .map(|tool| tool.spec())
        .filter(|spec| !excluded_tools.iter().any(|excluded| excluded == &spec.name))
        .collect()
}

pub(super) fn is_non_cli_tool_excluded(ctx: &ChannelRuntimeContext, tool_name: &str) -> bool {
    ctx.non_cli_excluded_tools
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .any(|excluded| excluded == tool_name)
}

pub(super) fn build_runtime_tool_visibility_prompt(
    tools_registry: &[Box<dyn Tool>],
    excluded_tools: &[String],
    native_tools: bool,
) -> String {
    let mut prompt = String::new();
    let mut specs = filtered_tool_specs_for_runtime(tools_registry, excluded_tools);
    specs.sort_by(|a, b| a.name.cmp(&b.name));

    use std::fmt::Write;
    prompt.push_str("\n## Runtime Tool Availability (Authoritative)\n\n");
    prompt.push_str(
        "This section is generated from current runtime policy for this message. \
         Only the listed tools may be called in this turn.\n\n",
    );

    if specs.is_empty() {
        prompt.push_str("- Allowed tools: (none)\n");
    } else {
        let _ = writeln!(prompt, "- Allowed tools ({}):", specs.len());
        for spec in &specs {
            let _ = writeln!(prompt, "  - `{}`", spec.name);
        }
    }

    if excluded_tools.is_empty() {
        prompt.push_str("- Excluded by runtime policy: (none)\n\n");
    } else {
        let mut excluded_sorted = excluded_tools.to_vec();
        excluded_sorted.sort();
        let _ = writeln!(
            prompt,
            "- Excluded by runtime policy: {}\n",
            excluded_sorted.join(", ")
        );
    }

    prompt.push_str(
        "- Do not claim tools are unavailable when they are listed above; call the appropriate tool directly.\n",
    );
    prompt.push_str(
        "- If the user asks what you can do, answer from the allowed tool list above plus loaded skills and channel capabilities below.\n",
    );
    prompt.push_str(
        "- Distinguish clearly between actions available now, actions that still require approval, and workflows that remain operator-controlled.\n",
    );
    prompt.push_str(
        "- Self-improvement is not automatic by default; candidate preparation, validation, and promotion remain manual/operator-controlled unless a dedicated workflow was explicitly configured.\n",
    );
    if specs
        .iter()
        .any(|spec| matches!(spec.name.as_str(), "file_write" | "file_edit"))
    {
        prompt.push_str(
            "- File changes are supported in this turn (`file_write`/`file_edit`) when requested and policy permits.\n",
        );
    }

    if native_tools {
        prompt.push_str(
            "Tool calling for this turn uses native provider function-calling. \
             Do not emit `<tool_call>` XML tags.\n",
        );
    } else {
        prompt.push_str(
            "Tool calling for this turn uses XML tool protocol below. \
             This protocol block is generated from the same runtime policy snapshot.\n",
        );
        prompt.push_str(&crate::agent::loop_::build_tool_instructions_from_specs(
            &specs,
        ));
    }

    prompt
}
