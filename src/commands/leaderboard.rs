use serenity::all::{CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuOption};
use serenity::builder::CreateCommand;

use crate::Handler;

pub async fn run(h: &Handler, ctx: &Context, command: &CommandInteraction) -> Result<(), serenity::Error> {
    let data = CreateInteractionResponseMessage::new()
        .embed(crate::utils::ui::generate_leaderboard_embed(&h.db_conn, None, Some(command.user.id.get())).await)
        .select_menu(
            CreateSelectMenu::new("leaderboard-category", serenity::all::CreateSelectMenuKind::String { options: vec![
                CreateSelectMenuOption::new("Bugs Confirmed", "leaderboard-category-bug_confirm")
                    .description("Display an extended leaderboard for the confirmed bugs category")
                    .emoji('✅'),
                CreateSelectMenuOption::new("Bugs Discovered", "leaderboard-category-bug_report")
                    .description("Display an extended leaderboard for the discovered bugs category")
                    .emoji('🐛'),
                CreateSelectMenuOption::new("Bugs Solved", "leaderboard-category-pr_fix")
                    .description("Display an extended leaderboard for the solved bugs category")
                    .emoji('📍'),
                CreateSelectMenuOption::new("General", "leaderboard-category-general")
                    .description("Display the default leaderboard with all categories")
                    .emoji('⭐')
            ] }).placeholder("Leaderboard categories")
        )
        .ephemeral(true);
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leaderboard")
        .description("See the leaderboard")
}
