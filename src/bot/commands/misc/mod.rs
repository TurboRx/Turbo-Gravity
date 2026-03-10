mod choose;
mod poll;
mod remind;

use crate::bot::Error;
use crate::state::SharedState;
use poise::Command;

pub fn commands() -> Vec<Command<SharedState, Error>> {
    vec![choose::choose(), poll::poll(), remind::remind()]
}
