use super::{ChannelsConfig, GroupReplyConfig, GroupReplyMode, StreamMode};
use crate::config::traits::ChannelConfig;

struct ConfigWrapper<T: ChannelConfig>(std::marker::PhantomData<T>);

impl<T: ChannelConfig> ConfigWrapper<T> {
    fn new(_: Option<&T>) -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: ChannelConfig> crate::config::traits::ConfigHandle for ConfigWrapper<T> {
    fn name(&self) -> &'static str {
        T::name()
    }

    fn desc(&self) -> &'static str {
        T::desc()
    }
}

impl ChannelsConfig {
    /// get channels' metadata and `.is_some()`, except webhook
    #[rustfmt::skip]
    pub fn channels_except_webhook(&self) -> Vec<(Box<dyn crate::config::traits::ConfigHandle>, bool)> {
        vec![
            (
                Box::new(ConfigWrapper::new(self.bridge.as_ref())),
                self.bridge.is_some(),
            ),
            (
                Box::new(ConfigWrapper::new(self.telegram.as_ref())),
                self.telegram.is_some(),
            ),
            (
                Box::new(ConfigWrapper::new(self.discord.as_ref())),
                self.discord.is_some(),
            ),
        ]
    }

    pub fn channels(&self) -> Vec<(Box<dyn crate::config::traits::ConfigHandle>, bool)> {
        let mut ret = self.channels_except_webhook();
        ret.push((
            Box::new(ConfigWrapper::new(self.webhook.as_ref())),
            self.webhook.is_some(),
        ));
        ret
    }
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            cli: true,
            bridge: None,
            telegram: None,
            discord: None,
            webhook: None,
            message_timeout_secs: default_channel_message_timeout_secs(),
        }
    }
}

pub(crate) fn default_channel_message_timeout_secs() -> u64 {
    300
}

pub(crate) fn default_draft_update_interval_ms() -> u64 {
    500
}

pub(crate) fn default_telegram_stream_mode() -> StreamMode {
    StreamMode::Partial
}

pub(crate) fn resolve_group_reply_mode(
    group_reply: Option<&GroupReplyConfig>,
    default_mode: GroupReplyMode,
) -> GroupReplyMode {
    if let Some(mode) = group_reply.and_then(|cfg| cfg.mode) {
        return mode;
    }
    default_mode
}

pub(crate) fn clone_group_reply_allowed_sender_ids(
    group_reply: Option<&GroupReplyConfig>,
) -> Vec<String> {
    group_reply
        .map(|cfg| cfg.allowed_sender_ids.clone())
        .unwrap_or_default()
}
