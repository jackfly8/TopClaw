//! Channel factory module.
//!
//! This module provides channel instantiation from configuration.
//! It creates all supported channel types based on the runtime config.

use super::traits::Channel;
use crate::config::Config;
use std::sync::Arc;

#[cfg(feature = "channel-discord")]
pub use super::DiscordChannel;
pub use super::TelegramChannel;

/// A configured channel with its display name.
pub struct ConfiguredChannel {
    pub display_name: &'static str,
    pub channel: Arc<dyn Channel>,
}

/// Collect all configured channels from the config.
pub fn collect_configured_channels(config: &Config) -> Vec<ConfiguredChannel> {
    let mut channels = Vec::new();

    if let Some(ref tg) = config.channels_config.telegram {
        let mut telegram = TelegramChannel::new(
            tg.bot_token.clone(),
            tg.allowed_users.clone(),
            tg.effective_group_reply_mode().requires_mention(),
        )
        .with_group_reply_allowed_senders(tg.group_reply_allowed_sender_ids())
        .with_streaming(tg.stream_mode, tg.draft_update_interval_ms)
        .with_transcription(config.transcription.clone())
        .with_workspace_dir(config.workspace_dir.clone());

        if let Some(ref base_url) = tg.base_url {
            telegram = telegram.with_api_base(base_url.clone());
        }

        channels.push(ConfiguredChannel {
            display_name: "Telegram",
            channel: Arc::new(telegram),
        });
    }

    #[cfg(feature = "channel-discord")]
    if let Some(ref dc) = config.channels_config.discord {
        channels.push(ConfiguredChannel {
            display_name: "Discord",
            channel: Arc::new(
                DiscordChannel::new(
                    dc.bot_token.clone(),
                    dc.guild_id.clone(),
                    dc.allowed_users.clone(),
                    dc.listen_to_bots,
                    dc.effective_group_reply_mode().requires_mention(),
                )
                .with_group_reply_allowed_senders(dc.group_reply_allowed_sender_ids())
                .with_transcription(config.transcription.clone())
                .with_workspace_dir(config.workspace_dir.clone()),
            ),
        });
    }

    channels
}
