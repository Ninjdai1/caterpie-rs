use std::{cmp::min, collections::HashMap, usize};
use sea_orm::EntityTrait;

use serenity::all::CreateEmbed;

use crate::{entities::{actions::ActionType, prelude::*}, Handler};

#[derive(Default, Clone, Copy)]
pub struct Score {
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

pub async fn generate_leaderboard_embed(h: &Handler, action_type: Option<ActionType>, id: Option<u64>) -> CreateEmbed {
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

    CreateEmbed::new()
            .title("Bug Catching Contest 2025 Leaderboard")
            .description(format!("\
                **Bug Confirmations**\n{}\n\n\
                **Bug Discovered**\n{}\n\n\
                **Bug Solved**\n{}\n\n\
                **Points**\n{}",
                    generate_leaderboard_string(&mut score_vec, Some(ActionType::ConfirmBug), id, 3),
                    generate_leaderboard_string(&mut score_vec, Some(ActionType::ReportBug), id, 3),
                    generate_leaderboard_string(&mut score_vec, Some(ActionType::PRFix), id, 3),
                    generate_leaderboard_string(&mut score_vec, None, id, 3),
            ))
}

fn generate_leaderboard_string(score_vec: &mut Vec<(&String, &Score)>, action_type: Option<ActionType>, id: Option<u64>, max: usize) -> String {
    let sort_fn = match action_type {
        None => |k: (&String, &Score)| k.1.get_total_points(),
        Some(ActionType::ConfirmBug) => |k: (&String, &Score)| k.1.bug_confirm,
        Some(ActionType::ReportBug) => |k: (&String, &Score)| k.1.bug_report,
        Some(ActionType::PRFix) => |k: (&String, &Score)| k.1.pr_fix
    };
    let type_str = match action_type {
        None => "points",
        Some(ActionType::ConfirmBug) => "confirmed bugs",
        Some(ActionType::ReportBug) => "discovered bugs",
        Some(ActionType::PRFix) => "solved bugs"
    };
    score_vec.sort_by_key(|k: &(&String, &Score)| sort_fn(*k));
    score_vec.reverse();
    
    let mut res_str = String::new();
    println!("Format for {type_str} - {max}/{}", score_vec.len());
    for i in 0..min(max, score_vec.len()) {
        let u = score_vec[i];
        let prefix = match i {0=>"🥇".to_string(), 1=>"🥈".to_string(), 2=>"🥉".to_string(), _=>format!("#{}", i+1)};
        res_str.push_str(&format!("\n{} <@{}>: {} {}",
            prefix,
            u.0,
            sort_fn(u),
            type_str
        ).to_owned());
    }
    res_str
}
