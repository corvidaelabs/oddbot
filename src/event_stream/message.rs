use serde::{Deserialize, Serialize};

use crate::skeever::squeak::Squeak;

#[derive(Deserialize, Serialize)]
pub struct EventMessage<T>
where
    T: Serialize,
{
    pub subject: String,
    pub payload: T,
}

const EVENT_PREFIX: &str = "oddlaws.events";

impl From<Squeak> for EventMessage<Squeak> {
    fn from(squeak: Squeak) -> Self {
        let subject = format!("{}.skeever.post", EVENT_PREFIX);
        EventMessage {
            subject,
            payload: squeak,
        }
    }
}
