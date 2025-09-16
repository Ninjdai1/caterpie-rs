use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;

use crate::Handler;

pub async fn run(
    _: &Handler,
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), serenity::Error> {
    let data = CreateInteractionResponseMessage::new().content("pong");
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}
