use serenity::all::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::{Handler, entities::actions::ActionType};

pub async fn run(
    h: &Handler,
    ctx: &Context,
    interaction: &ComponentInteraction,
    values: &Vec<String>,
) -> Result<(), serenity::Error> {
    match interaction.data.custom_id.split('-').collect::<Vec<_>>()[1] {
        "category" => {
            let data = CreateInteractionResponseMessage::new().embed(
                crate::utils::ui::generate_leaderboard_embed(
                    &h.db_conn,
                    match values.first().unwrap().split('-').collect::<Vec<_>>()[2] {
                        "bug_confirm" => Some(ActionType::ConfirmBug),
                        "bug_report" => Some(ActionType::ReportBug),
                        "pr_fix" => Some(ActionType::PRFix),
                        _ => None,
                    },
                    Some(interaction.user.id.get()),
                )
                .await,
            );
            let builder = CreateInteractionResponse::UpdateMessage(data);
            interaction.create_response(&ctx.http, builder).await
        }
        _ => Err(serenity::Error::Other(
            "Leaderboard select menu subcommand not implemented",
        )),
    }
}
