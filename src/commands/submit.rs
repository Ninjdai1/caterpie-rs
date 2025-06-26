use std::time::{Duration};

use sea_orm::{ActiveValue, EntityTrait};
use crate::entities::actions::ActionType;
use crate::entities::{actions, prelude::*};

use crate::utils::issues::IssueIds;
use crate::{Handler, CONFIG, CONTEST_START_DATE};

use serenity::all::*;

pub async fn run(h: &Handler, ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    if let (
            Some(ResolvedOption {value: ResolvedValue::String(submit_type), ..}),
            Some(ResolvedOption { value: ResolvedValue::String(submit_link), .. })
        ) = (&command.data.options().first(), &command.data.options().get(1))
    {
        command.defer_ephemeral(&ctx.http).await?;
        let action_type = match *submit_type {
            "bug_report" => ActionType::ReportBug,
            "bug_confirm" => ActionType::ConfirmBug,
            _ => ActionType::PRFix
        };

        let issue_ids_t = IssueIds::from_url(*submit_link);
        log::debug!("{issue_ids_t:#?}");
        if issue_ids_t.is_none() {
            command.edit_response(&ctx.http, EditInteractionResponse::new().content("Invalid URL Provided !")).await?;
            return Ok(());
        }
        let issue_ids = issue_ids_t.unwrap();
        
        let fetched_issue = octocrab::instance().issues("rh-hideout", "pokeemerald-expansion").get(issue_ids.issue_id).await;
        if let Err(e) = fetched_issue {
            log::debug!("Error while fetching issue at {submit_link:?}: {e:?}");
            return Err(Error::Other("Error while fetching specified url"))
        }
        let issue = fetched_issue.unwrap();
        log::debug!("Fetched issue #{}: {}", issue.number, issue.title);

        let valid_link = match action_type {
            ActionType::ReportBug => issue.pull_request.is_none() && issue_ids.comment_id.is_none() && issue.labels.iter().find(|label| label.name == "bug").is_some(),
            ActionType::ConfirmBug => issue.pull_request.is_none() && issue_ids.comment_id.is_some() && issue.labels.iter().find(|label| label.name == "bug").is_some(),
            ActionType::PRFix => issue.pull_request.is_some(),
        };

        if !valid_link {
            command.edit_response(&ctx.http, EditInteractionResponse::new().content(format!("The URL provided ({}) doesn't correspond to the specified submission type ({})", submit_link, action_type.to_string()))).await?;
            return Ok(())
        }

        let action_creation_date = match action_type {
            ActionType::PRFix | ActionType::ReportBug => issue.created_at,
            ActionType::ConfirmBug => {
                let fetched_comment = octocrab::instance().issues("rh-hideout", "pokeemerald-expansion").get_comment(issue_ids.comment_id.unwrap().into()).await;
                if let Err(_) = fetched_comment {
                    command.edit_response(&ctx.http, EditInteractionResponse::new().content(format!("Couldn't locate comment at provided URL ({}).", submit_link))).await?;
                    return Ok(());
                } else {
                    fetched_comment.unwrap().created_at
                }
            }
        };

        if action_creation_date.signed_duration_since(*CONTEST_START_DATE).num_seconds() < 0 {
            command.edit_response(&ctx.http, EditInteractionResponse::new().content(
                    format!("The submitted {} dates from before the start of the contest >.<\nThe contest started <t:{}:R> while the {} was created <t:{}:R>",
                        action_type.get_github_type(),
                        CONTEST_START_DATE.timestamp(),
                        action_type.get_github_type(),
                        action_creation_date.timestamp()))).await?;
            return Ok(())
        }

        let submitted_action = actions::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(command.user.id.get().to_string()),
            action_type: ActiveValue::Set(action_type),
            github_link: ActiveValue::Set(submit_link.to_string()),
            action_status: ActiveValue::Set(actions::ActionStatus::Pending)
        };

        let reply_builder = EditInteractionResponse::new()
            .content(format!("Submit a {} for {} ?", action_type.to_string(), issue.title))
            .button(CreateButton::new("ignore-submit-confirm").style(ButtonStyle::Success).label("Confirm"))
            .button(CreateButton::new("ignore-submit-cancel").style(ButtonStyle::Danger).label("Cancel"));

        command.edit_response(&ctx.http, reply_builder).await?;
        let mut msg = command.get_response(&ctx.http).await?;
        let btn_interaction = msg.await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60)).await;
        match btn_interaction {
            Some(i) => {
                i.defer(&ctx.http).await?;
                let args: Vec<_> = i.data.custom_id.split('-').collect();
                let confirmed = args[2] == "confirm";
                if confirmed {
                    let _ = Actions::insert(submitted_action).exec(&h.db_conn).await;
                    command.edit_response(&ctx.http, EditInteractionResponse::new().content(format!("Successfully submitted your {} !", action_type.to_string())).components(vec![]).embeds(vec![])).await?;
                    CONFIG.feed_channel.send_message(&ctx.http, CreateMessage::new().embed(
                        CreateEmbed::new().description(
                            format!("### <@{}> {} a bug ! +{} points\nLinked {}: [#{} - {}]({})",
                            command.user.id.get(),
                            match action_type {ActionType::ConfirmBug => "confirmed", ActionType::ReportBug => "discovered", ActionType::PRFix => "solved"},
                            action_type.get_points(),
                            action_type.get_github_type(), issue.number, issue.title, submit_link))
                    )).await?;
                } else {
                    command.edit_response(&ctx.http, EditInteractionResponse::new().content("Cancelled the submission").components(vec![]).embeds(vec![])).await?;
                }
            },
            None =>  {
                msg.edit(&ctx.http, EditMessage::new().content("Interaction timed out...")).await?;
                msg.components.clear();
                return Ok(());
            },
        };
        Ok(())
    } else {
        Err(Error::Other("Invalid input"))
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("submit").description("Submit a bug report, a fix PR or confirm a bug")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "type", "The type of submission")
            .add_string_choice("Discover Bug", "bug_report")
            .add_string_choice("Confirm Bug", "bug_confirm")
            .add_string_choice("Solve Bug", "fix_pr")
            .required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::String, "url", "The url of the issue, comment or PR you submit for")
            .min_int_value(0)
            .max_int_value(100_000)
            .required(true))
}
