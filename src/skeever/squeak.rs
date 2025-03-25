use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::OddbotConfig;

pub struct SqueakBuilder {
    content: Option<String>,
    user_name: Option<String>,
}

#[derive(Error, Debug)]
pub enum SqueakError {
    #[error("User name is required")]
    UserNameRequired,
    #[error("Content is required")]
    ContentRequired,
}

impl SqueakBuilder {
    /// Sets the content of the squeak
    pub fn content(mut self, value: String) -> Self {
        self.content = Some(value);
        self
    }

    /// Sets the user name of the squeak
    pub fn user(mut self, name: String) -> Self {
        self.user_name = Some(name);
        self
    }

    /// Builds the squeak
    pub fn build(self) -> Result<Squeak, SqueakError> {
        let Some(user_name) = self.user_name else {
            return Err(SqueakError::UserNameRequired);
        };

        let Some(content) = self.content else {
            return Err(SqueakError::ContentRequired);
        };

        Ok(Squeak {
            id: ulid::Ulid::new(),
            content,
            author: User { name: user_name },
        })
    }
}

impl Default for SqueakBuilder {
    fn default() -> Self {
        SqueakBuilder {
            content: None,
            user_name: None,
        }
    }
}

impl IntoFuture for SqueakBuilder {
    type Output = Result<Squeak, SqueakError>;
    type IntoFuture = futures::future::Ready<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        futures::future::ready(self.build())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Squeak {
    pub id: ulid::Ulid,
    pub content: String,
    pub author: User,
}

impl Squeak {
    pub fn builder() -> SqueakBuilder {
        SqueakBuilder::default()
    }

    pub fn get_subject() -> String {
        let prefix =
            OddbotConfig::get_event_stream_prefix().unwrap_or("oddlaws.events".to_string());
        format!("{}.skeever.post", prefix)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub name: String,
}
