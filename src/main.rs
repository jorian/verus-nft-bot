extern crate verusnftlib;

use std::sync::Arc;

use color_eyre::Report;
use secrecy::ExposeSecret;
use tracing::{debug, error, instrument};
use tracing_subscriber::filter::EnvFilter;

use serenity::{
    client::{Client, Context},
    framework::standard::{
        macros::{group, hook},
        DispatchError, StandardFramework,
    },
    model::{channel::Message, gateway::GatewayIntents},
};

use verusnftlib::bot::{
    events,
    utils::database::{DatabasePool, SequenceStart},
    utils::{self, database::GuildId},
};

#[group]
struct General;

#[tokio::main(worker_threads = 8)]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config =
        verusnftlib::configuration::get_configuration().expect("failed to read configuration");

    setup_logging().await?;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .on_dispatch_error(on_dispatch_error)
        .group(&GENERAL_GROUP);

    let handler = Arc::new(events::Handler {});

    let mut intents = GatewayIntents::all();
    intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
    intents.remove(GatewayIntents::GUILD_MESSAGE_TYPING);

    let mut client = Client::builder(config.application.discord.expose_secret(), intents)
        .event_handler_arc(handler.clone())
        .framework(framework)
        .await
        .expect("Error creating serenity client");

    {
        // in a block to close the write borrow
        let mut data = client.data.write().await;

        let pg_pool = utils::database::obtain_postgres_pool(&config.database).await?;
        sqlx::migrate!("./migrations").run(&pg_pool).await?;
        data.insert::<DatabasePool>(pg_pool);

        let guild_id = config.application.discord_guild_id.clone();
        data.insert::<GuildId>(guild_id.parse()?);

        let sequence_start = config.application.sequence_start;
        data.insert::<SequenceStart>(sequence_start as i64);
    }

    debug!("starting client");

    if let Err(why) = client.start().await {
        error!(
            "An error occurred while running the discord bot client: {:?}",
            why
        );
    }

    Ok(())
}

async fn setup_logging() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "serenity=info,verusnft=debug")
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}

#[hook]
pub async fn on_dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    _command_name: &str,
) {
    match error {
        DispatchError::OnlyForDM => {
            if let Err(e) = msg
                .reply(ctx, "This can only be done in DM with this bot")
                .await
            {
                error!("something went wrong while sending a reply in DM: {:?}", e);
            }
        }
        _ => {
            error!("Unhandled dispatch error: {:?}", error);
        }
    }
}
