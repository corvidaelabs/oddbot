use crate::prelude::*;
use crate::skeever::squeak::Squeak;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct EventMessage<T>
where
    T: Serialize,
{
    pub subject: String,
    pub payload: T,
}

impl From<Squeak> for EventMessage<Squeak> {
    fn from(squeak: Squeak) -> Self {
        let prefix = Config::get_event_stream_prefix().unwrap_or("oddlaws.events".to_string());
        let subject = format!("{}.skeever.post", prefix);
        EventMessage {
            subject,
            payload: squeak,
        }
    }
}
