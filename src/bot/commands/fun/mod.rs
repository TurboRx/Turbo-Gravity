mod ball8;
mod balance;
mod coinflip;
mod daily;
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
        roll::roll(),
        work::work(),
    ]
}
