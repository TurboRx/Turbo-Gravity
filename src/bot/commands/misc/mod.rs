mod choose;
mod giveaway;
mod poll;
mod remind;

use crate::bot::Error;
use crate::state::SharedState;
use poise::Command;

pub fn commands() -> Vec<Command<SharedState, Error>> {
    vec![
        choose::choose(),
        giveaway::giveaway(),
        poll::poll(),
        remind::remind(),
    ]
}
