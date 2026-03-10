mod avatar;
mod channelinfo;
mod contime;
mod embed;
mod help;
mod ping;
mod roleinfo;
mod serverinfo;
mod stats;
mod uptime;
mod userinfo;

use crate::bot::Error;
use crate::state::SharedState;
use poise::Command;

pub fn commands() -> Vec<Command<SharedState, Error>> {
    vec![
        avatar::avatar(),
        channelinfo::channelinfo(),
        contime::contime(),
        embed::embed(),
        help::help(),
        ping::ping(),
        roleinfo::roleinfo(),
        serverinfo::serverinfo(),
        stats::stats(),
        uptime::uptime(),
        userinfo::userinfo(),
    ]
}
