mod commands;
mod interactions;
mod entities;
mod utils;

use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
use serenity::all::{ChannelId, ComponentInteractionDataKind, GuildId, MessageId, Ready};
use serenity::all::{Command, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction};
use serenity::async_trait;
use serenity::prelude::*;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use log::{info, debug, error};

use crate::utils::config::Config;
use crate::utils::ui::update_permanent_leaderboard;

pub struct Handler {
    db_conn: DatabaseConnection,
    is_loop_running: AtomicBool
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    contest_start_timestamp: 1749506400,
    feed_channel: ChannelId::new(1387691060477689856),
    permanent_leaderboard: (ChannelId::new(1386765701590814842), MessageId::from(1387686765644615700))
});

static CONTEST_START_DATE: LazyLock<DateTime<Utc>> = LazyLock::new(|| DateTime::from_timestamp(CONFIG.contest_start_timestamp, 0).unwrap());


#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                debug!("Received command interaction: {command:#?}");

                let res = match command.data.name.as_str() {
                    "ping" => commands::ping::run(&self, &ctx, &command).await,
                    "submit" => commands::submit::run(&self, &ctx, &command).await,
                    "leaderboard" => commands::leaderboard::run(&self, &ctx, &command).await,
                    "verify" => commands::verify::run(&self, &ctx, &command).await,
                    "dev" => commands::dev::run(&self, &ctx, &command).await,
                    _ => Err(SerenityError::Other("command not implemented")),
                };

                if let Err(e) = res {
                    error!("Error while running command {}: {e:?}", command.data.name);
                    let data = CreateInteractionResponseMessage::new().content("Encountered an error while running command, please report to the developers").ephemeral(true);
                    let builder = CreateInteractionResponse::Message(data);
                    let _ = command.create_response(&ctx.http, builder).await;
                }
            },
            Interaction::Component(interaction) => {
                let args: Vec<&str> = interaction.data.custom_id.split('-').collect();
                if args[0] == "ignore" {
                    return;
                }
                let res = match interaction.data.kind {
                    ComponentInteractionDataKind::StringSelect {ref values} => match args[0] {
                        "leaderboard" => interactions::selectmenus::leaderboard::run(&self, &ctx, &interaction, &values).await,
                        _ => Err(SerenityError::Other("interaction not implemented"))
                    },
                    _ => Err(SerenityError::Other("component interaction type not implemented"))
                };

                if let Err(e) = res {
                    error!("Error while running component interaction {}: {e:?}", interaction.data.custom_id);
                    let data = CreateInteractionResponseMessage::new().content("Encountered an error while running component interaction, please report to the developers").ephemeral(true);
                    let builder = CreateInteractionResponse::Message(data);
                    let _ = interaction.create_response(&ctx.http, builder).await;
                }
            }
            _ => ()
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        if let Err(why) = self.db_conn.ping().await {
            error!("Error while pinging database: {why:?}")
        } else {
            info!("Database is alive !")
        }

        debug!("Registering commands...");
        let commands = Command::set_global_commands(&ctx.http, vec![
            commands::ping::register(),
            commands::submit::register(),
            commands::leaderboard::register(),
            commands::verify::register(),
            commands::dev::register()
        ]).await;
        debug!("Registered slash commands: {commands:#?}");
        info!("Registered {} commands", commands.unwrap().iter().len());

        let ctx = Arc::new(ctx);
        let db_conn = Arc::new(self.db_conn.clone());
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            let db_conn1 = Arc::clone(&db_conn);
            tokio::spawn(async move {
                loop {
                    update_permanent_leaderboard(&db_conn1, &ctx1).await;
                    tokio::time::sleep(Duration::from_secs(120)).await;
                }
            });
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
    }
}


#[tokio::main]
async fn main() {
    let working_dir = env::var("WORKDIR").unwrap_or(".".to_string());

    println!("Loading log config file at {}/config/log4rs.yaml", working_dir);
    if let Err(err) = log4rs::init_file(format!("{}/config/log4rs.yaml", working_dir), Default::default()) {
        println!("Error while loading logger config: {err:?}");
        return;
    }

    let github_pat = env::var("GITHUB_PAT").expect("Expected a github personal access token in the environment");
    octocrab::initialise(octocrab::Octocrab::builder().personal_token(github_pat).build().unwrap());

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let db_url: String = format!("sqlite:{}/caterpie.db?mode=rwc", working_dir);
    let db_t = Database::connect(db_url).await;
    if let Err(err) = &db_t {
        error!("Error while connecting to database: {err:?}");
        return;
    }
    let db = db_t.unwrap();

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    if let Err(err) = db.execute(builder.build(schema.create_table_from_entity(crate::entities::prelude::Actions).if_not_exists())).await {
        error!("Error while creating database tables: {err:?}");
        return;
    }

    let mut client =
        Client::builder(&token, intents)
        .event_handler(Handler {
            db_conn: db,
            is_loop_running: AtomicBool::new(false)
        })
        .await.expect("Err creating client");

    if let Err(err) = client.start().await {
        error!("Client error: {err:?}");
    }
}
