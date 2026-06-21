mod ban;
mod clearwarn;
mod kick;
mod lock;
mod purge;
mod slowmode;
mod tempban;
mod timeout;
mod unban;
mod unlock;
mod warn;
mod warnings;

use crate::bot::Error;
use crate::state::SharedState;
use poise::Command;

pub fn commands() -> Vec<Command<SharedState, Error>> {
    vec![
        ban::ban(),
        clearwarn::clearwarn(),
        kick::kick(),
        lock::lock(),
        purge::purge(),
        slowmode::slowmode(),
        tempban::tempban(),
        timeout::timeout(),
        unban::unban(),
        unlock::unlock(),
        warn::warn(),
        warnings::warnings(),
    ]
}
