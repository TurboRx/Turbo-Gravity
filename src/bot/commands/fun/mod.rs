mod balance;
mod ball8;
mod coinflip;
mod daily;
mod leaderboard;
mod roll;
mod work;

use crate::bot::Error;
use crate::state::SharedState;
use poise::Command;

pub fn commands() -> Vec<Command<SharedState, Error>> {
    vec![
        ball8::ball8(),
        balance::balance(),
        coinflip::coinflip(),
        daily::daily(),
        leaderboard::leaderboard(),
        roll::roll(),
        work::work(),
    ]
}
