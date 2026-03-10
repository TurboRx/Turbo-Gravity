use crate::bot::{Context, Error};

/// List all available commands
#[poise::command(slash_command, ephemeral)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Use /help <command> for details on a specific command.",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
