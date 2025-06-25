use std::collections::HashMap;

use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::builder::CreateCommand;

use crate::entities::actions::ActionType;
use crate::{Handler, entities::prelude::*};

pub async fn run(h: &Handler, ctx: &Context, command: &CommandInteraction) -> Result<(), serenity::Error> {
    let actions = Actions::find().all(&h.db_conn).await;

    // user_id: Score
    let mut score_map: HashMap<String, Score> = HashMap::new();
    for action in actions.unwrap() {
        let score = score_map.entry(action.user_id).or_insert(Score::default());
        match action.action_type {
            ActionType::ReportBug => score.bug_report += 1,
            ActionType::ConfirmBug => score.bug_confirm += 1,
            ActionType::PRFix => score.pr_fix += 1
        };
    }

    let mut score_vec: Vec<_> = score_map.iter().collect();
    score_vec.sort_by_key(|k| k.1.bug_confirm);
    score_vec.reverse();
    let bug_confirm_podium: [(&String, &Score); 3] = [
        (score_vec[0].0, score_vec[0].1),
        (score_vec[1].0, score_vec[1].1),
        (score_vec[2].0, score_vec[2].1)
    ];

    score_vec.sort_by_key(|k| k.1.bug_report);
    score_vec.reverse();
    let bug_report_podium: [(&String, &Score); 3] = [
        (score_vec[0].0, score_vec[0].1),
        (score_vec[1].0, score_vec[1].1),
        (score_vec[2].0, score_vec[2].1)
    ];

    score_vec.sort_by_key(|k| k.1.pr_fix);
    score_vec.reverse();
    let pr_fix_podium: [(&String, &Score); 3] = [
        (score_vec[0].0, score_vec[0].1),
        (score_vec[1].0, score_vec[1].1),
        (score_vec[2].0, score_vec[2].1)
    ];

    score_vec.sort_by_key(|k| k.1.get_total_points());
    score_vec.reverse();
    let overall_podium: [(&String, &Score); 3] = [
        (score_vec[0].0, score_vec[0].1),
        (score_vec[1].0, score_vec[1].1),
        (score_vec[2].0, score_vec[2].1)
    ];

    let data = CreateInteractionResponseMessage::new()
        .embed(CreateEmbed::new()
            .title("Bug Catching Contest 2025 Leaderboard")
            .description(format!("\
                    **Bug Confirmations**\n🥇 <@{}>: {} confirmed bugs\n🥈 <@{}>: {} confirmed bugs\n🥉 <@{}>: {} confirmed bugs\n\n\
                    **Bug Discovered**\n🥇 <@{}>: {} discovered bugs\n🥈 <@{}>: {} discovered bugs\n🥉 <@{}>: {} discovered bugs\n\n\
                    **Bug Solved**\n🥇 <@{}>: {} solved bugs\n🥈 <@{}>: {} solved bugs\n🥉 <@{}>: {} solved bugs\n\n\
                    **Points**\n🏆🥇 <@{}>: {} points\n🥈 <@{}>: {} points\n🥉 <@{}>: {} points\
                    ", bug_confirm_podium[0].0, bug_confirm_podium[0].1.bug_confirm, bug_confirm_podium[1].0, bug_confirm_podium[1].1.bug_confirm, bug_confirm_podium[2].0, bug_confirm_podium[2].1.bug_confirm,
                    bug_report_podium[0].0, bug_report_podium[0].1.bug_report, bug_report_podium[1].0, bug_report_podium[1].1.bug_report, bug_report_podium[2].0, bug_report_podium[2].1.bug_report,
                    pr_fix_podium[0].0, pr_fix_podium[0].1.pr_fix, pr_fix_podium[1].0, pr_fix_podium[1].1.pr_fix, pr_fix_podium[2].0, pr_fix_podium[2].1.pr_fix,
                    overall_podium[0].0, overall_podium[0].1.get_total_points(), overall_podium[1].0, overall_podium[1].1.get_total_points(), overall_podium[2].0, overall_podium[2].1.get_total_points())));
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leaderboard").description("See the leaderboard")
}

#[derive(Default, Clone, Copy)]
struct Score {
    bug_confirm: u64,
    bug_report: u64,
    pr_fix: u64
}

impl Score {
    pub fn get_total_points(&self) -> u64 {
        self.bug_confirm
            + 3 * self.bug_report 
            + 5 * self.pr_fix
    }
}
