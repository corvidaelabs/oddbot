//! A Character is a struct that represents an Oblivion character, which will be tied with a Discord user
use crate::error::OddbotError;
use async_nats::jetstream::{self, context::CreateKeyValueError, kv};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OblivionError {
    #[error("Failed to save oblivion character")]
    SaveCharacter(#[from] kv::PutError),
    #[error("Oblivion character missing data: {0}")]
    CharacterMissingData(String),
    #[error("Could not create key-value store")]
    CreateStore(#[from] CreateKeyValueError),
    #[error("Character serialization error")]
    ParseCharacter(#[from] serde_json::Error),
    #[error("Failed to get character")]
    GetCharacter(#[from] kv::EntryError),
    #[error("Failed to delete character")]
    DeleteCharacter(#[from] kv::DeleteError),
    #[error("No character found for user {0}")]
    NoCharacterFound(String),
}

#[derive(Deserialize, Serialize)]
pub struct Character {
    pub name: String,
    description: String,
    discord_id: String,
}

#[derive(Default)]
pub struct CharacterBuilder {
    discord_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

impl CharacterBuilder {
    pub fn discord_id(mut self, discord_id: String) -> Self {
        self.discord_id = Some(discord_id);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Validates the character data and builds the character
    pub fn build(self) -> Result<Character, OblivionError> {
        let discord_id = self.discord_id.ok_or(OblivionError::CharacterMissingData(
            "discord_id".to_string(),
        ))?;

        let name = self
            .name
            .ok_or(OblivionError::CharacterMissingData("name".to_string()))?;

        let description = self.description.ok_or(OblivionError::CharacterMissingData(
            "description".to_string(),
        ))?;

        let character = Character {
            name,
            description,
            discord_id,
        };

        Ok(character)
    }
}

impl Character {
    /// Creates a new Character instance
    pub fn builder() -> CharacterBuilder {
        CharacterBuilder::default()
    }

    /// Builds and saves the character to the store
    pub async fn save(self, store: &CharacterStore) -> Result<Character, OblivionError> {
        // Save the character to the store
        store.save_character(&self).await?;
        // Return the character
        Ok(self)
    }

    /// Gets a character based on the discord id
    pub async fn get_by_discord_id(
        discord_id: &str,
        store: &CharacterStore,
    ) -> Result<Option<Character>, OblivionError> {
        store.get_character(discord_id).await
    }

    /// Deletes a character based on the discord id
    pub async fn delete_by_discord_id(
        discord_id: &str,
        store: &CharacterStore,
    ) -> Result<(), OblivionError> {
        store.delete_character(discord_id).await?;
        Ok(())
    }
}

/// A CharacterStore is a struct for storing and retrieving oblivion characters from a key-value store
pub struct CharacterStore {
    store: kv::Store,
}

impl CharacterStore {
    /// Creates a new character store instance
    pub async fn new(client: async_nats::Client) -> Result<Self, OddbotError> {
        let store = jetstream::new(client)
            .create_key_value(kv::Config {
                bucket: "oblivion_characters".to_string(),
                ..Default::default()
            })
            .await
            .map_err(OblivionError::CreateStore)?;

        Ok(CharacterStore { store })
    }

    /// Saves a character to the store
    pub async fn save_character(&self, character: &Character) -> Result<(), OblivionError> {
        let data = serde_json::to_vec(&character)?;
        self.store
            .put(&character.discord_id, data.into())
            .await
            .map_err(OblivionError::SaveCharacter)?;

        Ok(())
    }

    /// Gets a character based on the discord id
    pub async fn get_character(
        &self,
        discord_id: &str,
    ) -> Result<Option<Character>, OblivionError> {
        let data = match self.store.get(discord_id).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        let character = serde_json::from_slice(&data)?;
        Ok(Some(character))
    }

    /// Deletes a character from the store
    pub async fn delete_character(&self, discord_id: &str) -> Result<(), OblivionError> {
        self.store
            .delete(discord_id)
            .await
            .map_err(OblivionError::DeleteCharacter)
    }
}
