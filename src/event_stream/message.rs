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
        let subject = Squeak::get_subject();
        EventMessage {
            subject,
            payload: squeak,
        }
    }
}
