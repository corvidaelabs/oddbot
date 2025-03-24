pub struct OddbotConfig;

impl OddbotConfig {
    /// Get the screenshot channel ID
    pub fn get_screenshot_channel_id() -> Option<u64> {
        Self::parse_optional_u64("SCREENSHOT_CHANNEL_ID")
    }

    /// Get the screenshot role ID
    pub fn get_screenshot_role_id() -> Option<u64> {
        Self::parse_optional_u64("SCREENSHOT_ROLE_ID")
    }

    /// Get the published member role ID
    pub fn get_published_member_role_id() -> Option<u64> {
        Self::parse_optional_u64("PUBLISHED_MEMBER_ROLE_ID")
    }

    /// Get the guild ID
    pub fn get_guild_id() -> Option<u64> {
        Self::parse_optional_u64("GUILD_ID")
    }

    /// Get the oblivion social channel ID
    pub fn get_oblivion_social_channel_id() -> Option<u64> {
        Self::parse_optional_u64("OBLIVION_SOCIAL_CHANNEL_ID")
    }

    /// Get the oblivion social role ID
    pub fn get_oblivion_social_role_id() -> Option<u64> {
        Self::parse_optional_u64("OBLIVION_SOCIAL_ROLE_ID")
    }

    /// Get the event stream name
    pub fn get_event_stream_name() -> Option<String> {
        std::env::var("EVENT_STREAM_NAME").ok()
    }

    /// Get the event stream prefix
    pub fn get_event_stream_prefix() -> Option<String> {
        std::env::var("EVENT_STREAM_PREFIX").ok()
    }

    /// Get the NATS URL
    pub fn get_nats_url() -> String {
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
    }

    /// Parse an optional u64 from an environment variable
    pub fn parse_optional_u64(env_var: &str) -> Option<u64> {
        std::env::var(env_var).ok().map(|id| {
            id.parse()
                .expect(format!("{} must be a valid u64", env_var).as_str())
        })
    }
}
