#[macro_use]
extern crate tracing;

use std::env;

use poise::serenity_prelude as serenity;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("resumed!");
    }
}

#[tokio::main]
async fn main() {
    // load env located at `./.env`, relative to cwd
    dotenv::dotenv()
        .expect("[error] failed to load .env file");
    // initialize logger
    tracing_subscriber::fmt::init();

    // set discord bot token
    let token = env::var("DISCORD_TOKEN")
        .expect("[error] expected a token in the environment");

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("[error] could not access application info: {:?}", why)
    };

    let mut client = Client::builder(&token).event_handler(Handler).await
        .expect("[error] error creating client");

    if let Err(why) = client.start().await {
        println!("[error] client error: {:?}", why);
    }
}
