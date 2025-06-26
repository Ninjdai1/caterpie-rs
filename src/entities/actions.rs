use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Actions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,
    #[sea_orm(column_name = "status")]
    pub action_status: ActionStatus,
    #[sea_orm(column_name = "type")]
    pub action_type: ActionType,
    pub github_link: String,
    pub user_id: String,
}

#[derive(EnumIter, DeriveActiveEnum, PartialEq, Eq, Clone, Copy, Debug)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum ActionType {
    #[sea_orm(string_value = "R")]
    ReportBug,
    #[sea_orm(string_value = "C")]
    ConfirmBug,
    #[sea_orm(string_value = "F")]
    PRFix
}

impl ActionType {
    pub fn get_github_type(&self) -> &str {
        match self {
            Self::ConfirmBug => "Comment",
            Self::ReportBug => "Issue",
            Self::PRFix => "PR"
        }
    }

    pub fn get_points(&self) -> u8 {
        match self {
            Self::ConfirmBug => 1,
            Self::ReportBug => 3,
            Self::PRFix => 5
        }
    }
}

impl ToString for ActionType {
    fn to_string(&self) -> String {
        match self {
            Self::ReportBug => "Bug Report".to_string(),
            Self::ConfirmBug => "Bug Confirmation".to_string(),
            Self::PRFix => "Bugfix PR".to_string()
        }
    }
}

#[derive(EnumIter, DeriveActiveEnum, PartialEq, Eq, Clone, Copy, Debug)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum ActionStatus {
    #[sea_orm(string_value = "/")]
    Pending,
    #[sea_orm(string_value = "Y")]
    Confirmed,
    #[sea_orm(string_value = "N")]
    Denied
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
