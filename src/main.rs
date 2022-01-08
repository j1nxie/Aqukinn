#![allow(non_snake_case)]
mod commands;
mod hooks;

#[macro_use]
extern crate tracing;

use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};

use serenity::{
    async_trait,
    client::{
        Client,
        Context,
        bridge::gateway::{GatewayIntents, ShardManager},
    },
    framework::standard::{
        macros::group,
        StandardFramework,
    },
    http::Http,
    model::{
        gateway::Ready,
        prelude::*,
        id::GuildId,
    },
    prelude::*,
};

use lavalink_rs::{
    gateway::*,
    model::*,
    LavalinkClient,
};

use songbird::SerenityInit;

use tokio::sync::Mutex;
use crate::hooks::*;
use crate::commands::{
    meta::*,
    music::*,
};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

struct Handler;
struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("track started!\nguild: {}", event.guild_id);
    }
    
    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("track finished!\nguild: {}", event.guild_id);
    }
}


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected! konaqua!", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("resumed!");
    }
    
    async fn cache_ready(&self, _: Context, _guilds: Vec<GuildId>) {
        info!("cache is ready!");
    }
}

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(join, leave, play, pause, resume, stop, now_playing, skip, clear_queue)]
struct Music;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load env located at `./.env`, relative to cwd
    dotenv::dotenv()
        .expect("[error] failed to load .env file");
    // initialize logger
    tracing_subscriber::fmt::init();
    info!("tracing initialized");

    // set discord bot token
    let token = env::var("DISCORD_TOKEN")
        .expect("[error] expected a token in the environment");

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }

            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("[error] could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("a!")
            .delimiters(vec![", ", ","])
            .owners(owners))
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .on_dispatch_error(dispatch_error)
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&MUSIC_GROUP);
//        .bucket("complicated", |b| b.limit(2).time_span(30).delay(5) // maximum 2 times every 30s, with a delay of 5s per channel
//            .limit_for(LimitedFor::Channel)
//            .await_ratelimits(1)
//            .delay_action(delay_action)).await

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .type_map_insert::<CommandCounter>(HashMap::default())
        .register_songbird()
        .await
        .expect("[error] error creating client");

    let lava_client = LavalinkClient::builder(bot_id)
        .set_host("127.0.0.1")
        .set_password(
            env::var("LAVALINK_PASSWORD").unwrap_or_else(|_| "youshallnotpass".to_string())
        )
        .build(LavalinkHandler)
        .await?;
    
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<Lavalink>(lava_client);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await
            .expect("could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start_shards(2).await {
        error!("[error] client error: {:?}", why);
    }

    Ok(())
}
