pub struct Config;

impl Config {
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

    /// Parse an optional u64 from an environment variable
    pub fn parse_optional_u64(env_var: &str) -> Option<u64> {
        std::env::var(env_var).ok().map(|id| {
            id.parse()
                .expect(format!("{} must be a valid u64", env_var).as_str())
        })
    }
}
