mod commands;
mod entities;
mod utils;

use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
use serenity::all::Ready;
use serenity::all::{Command, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction};
use serenity::async_trait;
use serenity::prelude::*;

use std::env;

use log::{info, debug, error};

pub struct Handler {
    db_conn: DatabaseConnection
}
    

static CONTEST_START_DATE: DateTime<Utc> = DateTime::from_timestamp(1749506400, 0).unwrap();


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
                    _ => Err(SerenityError::Other("command not implemented")),
                };

                if let Err(e) = res {
                    error!("Error while running command {}: {e:?}", command.data.name);
                    let data = CreateInteractionResponseMessage::new().content("Encountered an error while running command, please report to the developers").ephemeral(true);
                    let builder = CreateInteractionResponse::Message(data);
                    let _ = command.create_response(&ctx.http, builder).await;
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
            commands::verify::register()
        ]).await;
        debug!("Registered slash commands: {commands:#?}");
        info!("Registered {} commands", commands.unwrap().iter().len());
    }

}


#[tokio::main]
async fn main() {
    let working_dir = env::var("WORKDIR").expect("Expected a working directory in the environment");

    println!("Loading log config file at {}/config/log4rs.yaml", working_dir);
    if let Err(err) = log4rs::init_file(format!("{}/config/log4rs.yaml", working_dir), Default::default()) {
        println!("Error while loading logger config: {err:?}");
        return;
    }

    octocrab::initialise(octocrab::Octocrab::default());

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
        Client::builder(&token, intents).event_handler(Handler { db_conn: db }).await.expect("Err creating client");

    if let Err(err) = client.start().await {
        error!("Client error: {err:?}");
    }
}
