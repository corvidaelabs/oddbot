use super::handler::Handler;
use crate::prelude::*;
use serenity::all::{
    ChannelId, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId,
    GuildMemberUpdateEvent, Interaction, MessageId, Reaction,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::{Member, Role};
use serenity::model::id::RoleId;
use serenity::prelude::*;

#[async_trait]
impl EventHandler for Handler {
    /// This event will be dispatched on any message
    async fn message(&self, _ctx: Context, msg: Message) {
        tracing::debug!("Received message: {:?}", msg);
        // Having a screenshot channel ID presumes that we want to handle screenshots
        if let Some(screenshot_channel_id) = OddbotConfig::get_screenshot_channel_id() {
            // The default behavior is to only allow screenshots from users with a certain role
            // Later, we want probably add more complex logic to handle screenshots, like reaction-based permissions
            // Currently, we don't allow screenshots from users without a certain role
            if let Some(role_id) = OddbotConfig::get_screenshot_role_id() {
                self.handle_ss(&msg, role_id, screenshot_channel_id).await;
            }
        }

        // Having an oblivion social channel ID presumes that we have oblivion social enabled
        if let Some(oblivion_channel) = OddbotConfig::get_oblivion_social_channel_id() {
            // We only allow interaction from users with a certain role
            if let Some(role_id) = OddbotConfig::get_oblivion_social_role_id() {
                self.handle_oblivion_message(&msg, role_id, oblivion_channel)
                    .await;
            }
        }
    }

    /// This event will be dispatched when the bot is ready
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Ready and connected as {}", ready.user.name);
        if let Err(err) = self.init(ctx).await {
            panic!("Failed bot initialization: {}", err);
        }
    }

    /// This event will be dispatched when an interaction is created
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            tracing::debug!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "register" => {
                    super::commands::oblivion::character::save_character(
                        &ctx,
                        &command,
                        &self.character_store,
                    )
                    .await
                    .unwrap();
                    None
                }
                "whoami" => {
                    super::commands::oblivion::character::get_character(
                        &ctx,
                        &command,
                        &self.character_store,
                    )
                    .await
                    .unwrap();
                    None
                }
                "die" => {
                    super::commands::oblivion::character::delete_character(
                        &ctx,
                        &command,
                        &self.character_store,
                    )
                    .await
                    .unwrap();
                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    tracing::error!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    /// This event will be dispatched when a member's roles are updated
    async fn guild_member_update(
        &self,
        _ctx: Context,
        old_if_available: Option<Member>,
        new: Option<Member>,
        _update: GuildMemberUpdateEvent,
    ) {
        if let Some(old) = old_if_available {
            if let Some(new) = new {
                tracing::debug!(
                    "Member updated: {} ({:?} -> {:?})",
                    new.user.name,
                    old.roles,
                    new.roles
                );
            }
        }
    }

    /// This event will be dispatched when a role is created
    async fn guild_role_create(&self, _ctx: Context, new: Role) {
        tracing::debug!("New role created: {} in guild", new.name);
    }

    /// This event will be dispatched when a role is deleted
    async fn guild_role_delete(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _removed_role_id: RoleId,
        removed_role_data: Option<Role>,
    ) {
        tracing::debug!("Role deleted: {:?} in guild", removed_role_data);
    }

    /// This event will be dispatched when a role is updated
    async fn guild_role_update(&self, _ctx: Context, _old_if_available: Option<Role>, new: Role) {
        tracing::debug!("Role updated: {} in guild", new.name);
    }

    /// This event will be dispatched when a reaction is added to a message
    async fn reaction_add(&self, _ctx: Context, reaction: Reaction) {
        tracing::debug!("Reaction added: {:?}", reaction);
    }

    /// This event will be dispatched when a reaction is removed from a message
    async fn reaction_remove(&self, _ctx: Context, reaction: Reaction) {
        tracing::debug!("Reaction removed: {:?}", reaction);
    }

    /// This event will be dispatched when all reactions are removed from a message
    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        message_id: MessageId,
    ) {
        tracing::debug!(
            "All reactions removed from message {} in channel {}",
            message_id,
            channel_id
        );
    }

    /// This event will be dispatched when all reactions of a specific emoji are removed from a message
    async fn reaction_remove_emoji(&self, _ctx: Context, reaction: Reaction) {
        tracing::debug!("All {:?} reactions removed from message", reaction.emoji);
    }
}
