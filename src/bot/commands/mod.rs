pub mod fun;
pub mod misc;
pub mod moderation;
pub mod tickets;
pub mod utility;

use crate::state::SharedState;
use poise::Command;

use super::Error;

/// Collect all commands from every sub-module into a single `Vec` for
/// registration with the Poise framework.
pub fn all() -> Vec<Command<SharedState, Error>> {
    let mut cmds = Vec::new();
    cmds.extend(fun::commands());
    cmds.extend(misc::commands());
    cmds.extend(moderation::commands());
    cmds.extend(tickets::commands());
    cmds.extend(utility::commands());
    cmds
}
