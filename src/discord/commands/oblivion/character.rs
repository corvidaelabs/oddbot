use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::CreateQuickModal;

use crate::{
    discord::character::{Character, CharacterStore},
    error::OddbotError,
};

pub async fn save_character(
    ctx: &Context,
    interaction: &CommandInteraction,
    store: &CharacterStore,
) -> Result<(), OddbotError> {
    let modal = CreateQuickModal::new("Register your character")
        .timeout(std::time::Duration::from_secs(600))
        .short_field("Character Name")
        .paragraph_field("Character Description");
    let response = interaction.quick_modal(ctx, modal).await?.unwrap();
    let user_id = interaction.user.id;
    let inputs = response.inputs;
    let (name, description) = (&inputs[0], &inputs[1]);

    // Save the character to the store
    Character::builder()
        .discord_id(user_id.to_string())
        .name(name.to_string())
        .description(description.to_string())
        .build()?
        .save(&store)
        .await?;

    response
        .interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(
                format!("**Name**: {name}\n\nCharacter Description: {description}"),
            )),
        )
        .await?;
    Ok(())
}

pub async fn get_character(
    ctx: &Context,
    interaction: &CommandInteraction,
    store: &CharacterStore,
) -> Result<(), OddbotError> {
    let user_id = interaction.user.id;

    // Get the character from the store
    let character = Character::get_by_discord_id(&user_id.to_string(), &store).await?;

    let Some(character) = character else {
        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(format!("I don't know you.")),
                ),
            )
            .await?;
        return Ok(());
    };

    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(
                format!("Who are you? You are {} of course!", character.name),
            )),
        )
        .await?;
    Ok(())
}

pub async fn delete_character(
    ctx: &Context,
    interaction: &CommandInteraction,
    store: &CharacterStore,
) -> Result<(), OddbotError> {
    let user_id = interaction.user.id;
    // Get the character first to save their name for the response
    let character = Character::get_by_discord_id(&user_id.to_string(), &store).await?;
    let Some(character) = character else {
        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(format!("I don't know you.")),
                ),
            )
            .await?;
        return Ok(());
    };

    Character::delete_by_discord_id(&user_id.to_string(), store).await?;

    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(format!("o7 {}", character.name)),
            ),
        )
        .await?;
    Ok(())
}
