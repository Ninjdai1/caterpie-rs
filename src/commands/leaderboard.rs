use serenity::all::{CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::builder::CreateCommand;

use crate::Handler;

pub async fn run(h: &Handler, ctx: &Context, command: &CommandInteraction) -> Result<(), serenity::Error> {

    let data = CreateInteractionResponseMessage::new()
        .embed(crate::utils::ui::generate_leaderboard_embed(h, None, Some(command.user.id.get())).await)
        .ephemeral(true);
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leaderboard")
        .description("See the leaderboard")
}
