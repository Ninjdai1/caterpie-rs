use std::time::Duration;
use std::vec;

use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serenity::all::{
    ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateButton, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, EditMessage, ResolvedOption, ResolvedValue,
};
use serenity::builder::CreateCommand;

use crate::EXPANSION_SENATE_ROLE;
use crate::entities::actions::{self, ActionStatus};
use crate::utils::issues::IssueIds;
use crate::{Handler, entities::prelude::*};

pub async fn run(
    h: &Handler,
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), serenity::Error> {
    command.defer_ephemeral(&ctx.http).await?;

    if let Some(member) = &command.member {
        if !member
            .roles
            .iter()
            .find(|r| **r == EXPANSION_SENATE_ROLE)
            .is_some()
        {
            command
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .content("You're not an expansion senate member !"),
                )
                .await?;
            return Ok(());
        }
    } else {
        command
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .content("You must be in a server to run this command !"),
            )
            .await?;
        return Ok(());
    }

    let user_id = match &command.data.options().first() {
        Some(ResolvedOption {
            value: ResolvedValue::User(user, _),
            ..
        }) => Some(user.id.get()),
        _ => None,
    };
    let mut msg = command.get_response(&ctx.http).await?;

    loop {
        let data = create_verification_message(h, user_id).await;
        let builder = EditInteractionResponse::new()
            .embed(data.0)
            .components(match data.1 {
                Some(r) => vec![r],
                None => vec![],
            });
        command.edit_response(&ctx.http, builder).await?;
        let btn_interaction = msg
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(3 * 60))
            .await;
        match btn_interaction {
            Some(i) => {
                i.defer(&ctx.http).await?;
                let args: Vec<_> = i.data.custom_id.split('-').collect();
                let confirmed = match args[1] {
                    "confirm" => true,
                    "refuse" => false,
                    _ => {
                        i.edit_response(
                            &ctx.http,
                            EditInteractionResponse::new()
                                .content("Stopped the verification process !")
                                .components(vec![])
                                .embeds(vec![]),
                        )
                        .await?;
                        return Ok(());
                    }
                };
                let action_id: u32 = args[2].parse().unwrap();

                let model = actions::ActiveModel {
                    id: Set(action_id),
                    action_status: Set(if confirmed {
                        ActionStatus::Confirmed
                    } else {
                        ActionStatus::Denied
                    }),
                    ..Default::default()
                };
                let _ = Actions::update(model).exec(&h.db_conn).await;
            }
            None => {
                msg.edit(
                    &ctx.http,
                    EditMessage::new().content("Interaction timed out..."),
                )
                .await?;
                msg.components.clear();
                break;
            }
        }
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("verify")
        .description("Start the submission verification process")
        .add_option(CreateCommandOption::new(
            serenity::all::CommandOptionType::User,
            "user",
            "A specific user to verify submissions for",
        ))
}

pub async fn create_verification_message(
    h: &Handler,
    user: Option<u64>,
) -> (CreateEmbed, Option<CreateActionRow>, u32) {
    let action_pending = match user {
        None => {
            Actions::find()
                .filter(actions::Column::ActionStatus.eq(actions::ActionStatus::Pending))
                .one(&h.db_conn)
                .await
        }
        Some(u) => {
            Actions::find()
                .filter(actions::Column::ActionStatus.eq(actions::ActionStatus::Pending))
                .filter(actions::Column::UserId.eq(u))
                .one(&h.db_conn)
                .await
        }
    };
    if let Err(e) = action_pending {
        log::error!("Error while fetching action submissions: {e:?}");
        return (
            CreateEmbed::new().description("I encountered an error while fetching submissions >.<"),
            None,
            0,
        );
    }
    if let Some(action) = action_pending.unwrap() {
        let issue_ids = IssueIds::from_url(&action.github_link).unwrap();
        let issue = octocrab::instance()
            .issues("rh-hideout", "pokeemerald-expansion")
            .get(issue_ids.issue_id)
            .await
            .unwrap();
        return (
            CreateEmbed::new().description(format!(
                "Submission by <@{}>: **{}** for **(#{}) {}**",
                action.user_id,
                action.action_type.to_string(),
                issue.number,
                issue.title
            )),
            Some(CreateActionRow::Buttons(vec![
                CreateButton::new_link(action.github_link)
                    .label(format!("See {}", action.action_type.get_github_type())),
                CreateButton::new("")
                    .custom_id(format!("ignore-confirm-{}", action.id))
                    .label("Confirm")
                    .style(ButtonStyle::Success),
                CreateButton::new("")
                    .custom_id(format!("ignore-refuse-{}", action.id))
                    .label("Refuse")
                    .style(ButtonStyle::Danger),
                CreateButton::new("")
                    .custom_id(format!("ignore-stop-{}", action.id))
                    .label("Stop Verifying")
                    .style(ButtonStyle::Secondary),
            ])),
            action.id,
        );
    } else {
        return (
            CreateEmbed::new().description(match user {
                None => "No pending submission left".to_string(),
                Some(u) => format!("No pending submission left for <@{u}>"),
            }),
            None,
            0,
        );
    }
}
