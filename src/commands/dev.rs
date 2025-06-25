use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, ResolvedOption, ResolvedValue};
use serenity::builder::CreateCommand;

use crate::Handler;

pub async fn run(_: &Handler, ctx: &Context, command: &CommandInteraction) -> Result<(), serenity::Error> {
    if command.user.id.get() != 697438073646088194 {
        return command.defer_ephemeral(&ctx.http).await;
    }
    command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Dev").ephemeral(true))).await?;
    if let Some(ResolvedOption {value: ResolvedValue::String(message), ..}) = &command.data.options().first() {
        let msg = CreateMessage::new().content(*message);
        command.channel_id.send_message(&ctx.http, msg).await?;
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("dev").description("A dev command")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "content", "content"))
}
