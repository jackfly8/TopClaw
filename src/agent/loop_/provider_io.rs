use crate::agent::loop_::{
    ToolLoopCancelled, DRAFT_CLEAR_SENTINEL, STREAM_TOOL_MARKER_WINDOW_CHARS,
};
use crate::providers::traits::{StreamEvent, StreamOptions};
use crate::providers::{ChatMessage, ChatRequest, Provider, ToolCall};
use anyhow::Result;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
pub(super) struct StreamedChatOutcome {
    pub response_text: String,
    pub tool_calls: Vec<ToolCall>,
    pub forwarded_live_deltas: bool,
}

fn looks_like_streamed_tool_payload(window: &str) -> bool {
    let lowered = window.to_ascii_lowercase();
    lowered.contains("<tool_call")
        || lowered.contains("<toolcall")
        || lowered.contains("\"tool_calls\"")
}

pub(super) async fn call_provider_chat(
    provider: &dyn Provider,
    messages: &[ChatMessage],
    request_tools: Option<&[crate::tools::ToolSpec]>,
    model: &str,
    temperature: f64,
    cancellation_token: Option<&CancellationToken>,
) -> Result<crate::providers::ChatResponse> {
    let chat_future = provider.chat(
        ChatRequest {
            messages,
            tools: request_tools,
        },
        model,
        temperature,
    );

    if let Some(token) = cancellation_token {
        tokio::select! {
            () = token.cancelled() => Err(ToolLoopCancelled.into()),
            result = chat_future => result,
        }
    } else {
        chat_future.await
    }
}

pub(super) async fn consume_provider_streaming_response(
    provider: &dyn Provider,
    messages: &[ChatMessage],
    request_tools: Option<&[crate::tools::ToolSpec]>,
    model: &str,
    temperature: f64,
    cancellation_token: Option<&CancellationToken>,
    on_delta: Option<&tokio::sync::mpsc::Sender<String>>,
) -> Result<StreamedChatOutcome> {
    let mut provider_stream = provider.stream_chat(
        ChatRequest {
            messages,
            tools: request_tools,
        },
        model,
        temperature,
        StreamOptions::new(true),
    );
    let mut outcome = StreamedChatOutcome::default();
    let mut delta_sender = on_delta;
    let mut suppress_forwarding = false;
    let mut marker_window = String::new();

    loop {
        let next_chunk = if let Some(token) = cancellation_token {
            tokio::select! {
                () = token.cancelled() => return Err(ToolLoopCancelled.into()),
                chunk = provider_stream.next() => chunk,
            }
        } else {
            provider_stream.next().await
        };

        let Some(event_result) = next_chunk else {
            break;
        };

        let event = event_result.map_err(|err| anyhow::anyhow!("provider stream error: {err}"))?;
        match event {
            StreamEvent::Final => break,
            StreamEvent::ToolCall(tool_call) => {
                outcome.tool_calls.push(tool_call);
                suppress_forwarding = true;
                if outcome.forwarded_live_deltas {
                    if let Some(tx) = delta_sender {
                        let _ = tx.send(DRAFT_CLEAR_SENTINEL.to_string()).await;
                    }
                    outcome.forwarded_live_deltas = false;
                }
            }
            StreamEvent::TextDelta(chunk) => {
                if chunk.delta.is_empty() {
                    continue;
                }

                outcome.response_text.push_str(&chunk.delta);
                marker_window.push_str(&chunk.delta);

                if marker_window.len() > STREAM_TOOL_MARKER_WINDOW_CHARS {
                    let keep_from = marker_window.len() - STREAM_TOOL_MARKER_WINDOW_CHARS;
                    let boundary = marker_window
                        .char_indices()
                        .find(|(idx, _)| *idx >= keep_from)
                        .map_or(0, |(idx, _)| idx);
                    marker_window.drain(..boundary);
                }

                if !suppress_forwarding && looks_like_streamed_tool_payload(&marker_window) {
                    suppress_forwarding = true;
                    if outcome.forwarded_live_deltas {
                        if let Some(tx) = delta_sender {
                            let _ = tx.send(DRAFT_CLEAR_SENTINEL.to_string()).await;
                        }
                        outcome.forwarded_live_deltas = false;
                    }
                }

                if suppress_forwarding {
                    continue;
                }

                if let Some(tx) = delta_sender {
                    if !outcome.forwarded_live_deltas {
                        let _ = tx.send(DRAFT_CLEAR_SENTINEL.to_string()).await;
                        outcome.forwarded_live_deltas = true;
                    }
                    if tx.send(chunk.delta).await.is_err() {
                        delta_sender = None;
                    }
                }
            }
        }
    }

    Ok(outcome)
}
