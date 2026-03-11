use crate::util::truncate_with_ellipsis;

const AUTO_CRON_DELIVERY_CHANNELS: &[&str] = &["telegram", "discord", "slack", "mattermost"];

/// Extract a short hint from tool call arguments for progress display.
pub(super) fn truncate_tool_args_for_progress(
    name: &str,
    args: &serde_json::Value,
    max_len: usize,
) -> String {
    let hint = match name {
        "shell" => args.get("command").and_then(|v| v.as_str()),
        "file_read" | "file_write" => args.get("path").and_then(|v| v.as_str()),
        _ => args
            .get("action")
            .and_then(|v| v.as_str())
            .or_else(|| args.get("query").and_then(|v| v.as_str())),
    };
    match hint {
        Some(s) => truncate_with_ellipsis(s, max_len),
        None => String::new(),
    }
}

pub(super) fn qualifies_for_non_cli_investigation_batch(
    tool_name: &str,
    args: &serde_json::Value,
) -> bool {
    match tool_name {
        // Planning is stateful but low-risk and commonly used to organize one
        // investigation turn; avoid prompting for every create/update step.
        "task_plan" | "glob_search" | "content_search" | "lossless_search" | "file_read"
        | "memory_search" | "memory_recall" => true,
        "process" => matches!(
            args.get("action").and_then(serde_json::Value::as_str),
            Some("list" | "output")
        ),
        _ => false,
    }
}

pub(super) fn maybe_inject_cron_add_delivery(
    tool_name: &str,
    tool_args: &mut serde_json::Value,
    channel_name: &str,
    reply_target: Option<&str>,
) {
    if tool_name != "cron_add"
        || !AUTO_CRON_DELIVERY_CHANNELS
            .iter()
            .any(|supported| supported == &channel_name)
    {
        return;
    }

    let Some(reply_target) = reply_target.map(str::trim).filter(|v| !v.is_empty()) else {
        return;
    };

    let Some(args_obj) = tool_args.as_object_mut() else {
        return;
    };

    let is_agent_job = match args_obj.get("job_type").and_then(serde_json::Value::as_str) {
        Some("agent") => true,
        Some(_) => false,
        None => args_obj.contains_key("prompt"),
    };
    if !is_agent_job {
        return;
    }

    let delivery = args_obj
        .entry("delivery".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let Some(delivery_obj) = delivery.as_object_mut() else {
        return;
    };

    let mode = delivery_obj
        .get("mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none");
    if mode.eq_ignore_ascii_case("none") || mode.trim().is_empty() {
        delivery_obj.insert(
            "mode".to_string(),
            serde_json::Value::String("announce".to_string()),
        );
    } else if !mode.eq_ignore_ascii_case("announce") {
        // Respect explicitly chosen non-announce modes.
        return;
    }

    let needs_channel = delivery_obj
        .get("channel")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|value| value.trim().is_empty());
    if needs_channel {
        delivery_obj.insert(
            "channel".to_string(),
            serde_json::Value::String(channel_name.to_string()),
        );
    }

    let needs_target = delivery_obj
        .get("to")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|value| value.trim().is_empty());
    if needs_target {
        delivery_obj.insert(
            "to".to_string(),
            serde_json::Value::String(reply_target.to_string()),
        );
    }
}
