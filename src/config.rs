use std::collections::HashSet;

pub struct Config;

impl Config {
    /// Get the screenshot channel ID
    pub fn get_screenshot_channel_id() -> Option<u64> {
        Self::parse_optional_u64("SCREENSHOT_CHANNEL_ID")
    }

    /// Get the screenshot role ID
    pub fn get_screenshot_role_id() -> Option<u64> {
        Self::parse_optional_u64("TARGET_ROLE_ID")
    }

    /// Get the published member role ID
    pub fn get_published_member_role_id() -> Option<u64> {
        Self::parse_optional_u64("PUBLISHED_MEMBER_ROLE_ID")
    }

    /// Parse an optional u64 from an environment variable
    pub fn parse_optional_u64(env_var: &str) -> Option<u64> {
        std::env::var(env_var).ok().map(|id| {
            id.parse()
                .expect(format!("{} must be a valid u64", env_var).as_str())
        })
    }

    /// Get the allowed guild IDs
    pub fn get_allowed_guild_ids() -> HashSet<u64> {
        let guild_ids = std::env::var("ALLOWED_GUILD_IDS")
            .map(|ids| {
                ids.split(',')
                    .map(|id| {
                        id.trim()
                            .parse()
                            .expect(format!("{id} is not a valid u64").as_str())
                    })
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        guild_ids
    }
}
