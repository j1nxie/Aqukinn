#![allow(non_snake_case)]

use futures::stream::StreamExt;
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    Body, Request,
};
use std::{env, error::Error, sync::Arc, net::SocketAddr, str::FromStr, future::Future};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Event, Shard};
use twilight_model::{
    gateway::{
        payload::{incoming::MessageCreate, outgoing::UpdateVoiceState},
        Intents,
    },
    channel::Message,
};
use twilight_http::Client as HttpClient;
use twilight_lavalink::{
    http::LoadedTracks,
    model::{Destroy, Pause, Play, Seek, Stop, Volume},
    Lavalink,
};
use twilight_standby::Standby;
use dotenv::dotenv;
use tracing::{error, info, warn, debug};
use tracing_subscriber::FmtSubscriber;

use crate::commands::meta::*;

mod commands;

pub type State = Arc<StateRef>;

#[derive(Debug)]
pub struct StateRef {
    http: HttpClient,
    lavalink: Lavalink,
    hyper: HyperClient<HttpConnector>,
    shard: Shard,
    standby: Standby,
}

fn spawn(fut: impl Future<Output = anyhow::Result<()>> + Send + 'static) {
    tokio::spawn(async move {
        if let Err(why) = fut.await {
            debug!("Handler error: {why:?}");
        }
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracer subscriber
    let tracing_subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(tracing_subscriber)
        .expect("Setting tracing default failed.");

    let (mut events, state) = {
        // initialize Discord token
        dotenv().ok();
        let token: String;
        if let Ok(s) = env::var("DISCORD_TOKEN") {
            token = s;
        } else {
            error!("Discord token not found!");
            panic!();
        }

        let lavalink_host: SocketAddr;
        if let Ok(s) = env::var("LAVALINK_HOST") {
            if let Ok(a) = SocketAddr::from_str(&s) {
                lavalink_host = a;
            } else {
                error!("Invalid Lavalink host!");
                panic!();
            }
        } else {
            error!("Lavalink host not found!");
            panic!();
        }

        let lavalink_auth: String;
        if let Ok(s) = env::var("LAVALINK_AUTH") {
            lavalink_auth = s;
        } else {
            error!("Lavalink authorization not found!");
            panic!();
        }

        let shard_count = 1u64;

        let http = HttpClient::new(token.clone());
        let user_id = http.current_user().await?.model().await?.id;

        let lavalink = Lavalink::new(user_id, shard_count);
        lavalink.add(lavalink_host, lavalink_auth).await?;

        let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
        let (shard, events) = Shard::new(token, intents);

        shard.start().await?;

        (
            events,
            Arc::new(StateRef {
                http,
                lavalink,
                hyper: HyperClient::new(),
                shard,
                standby: Standby::new(),
            }),
        )
    };

    while let Some(event) = events.next().await {
        state.standby.process(&event);
        state.lavalink.process(&event).await?;

        match event {
            Event::MessageCreate(msg) => {
                if msg.guild_id.is_none() || !msg.content.starts_with("a!") {
                    continue;
                }

                match msg.content.strip_prefix("a!").unwrap().split_whitespace().next() {
                    Some("ping") => spawn(ping(msg.0, Arc::clone(&state))),
                    _ => continue,
                }
            }

            Event::ShardConnected(_) => info!("Connected!"),

            Event::ShardDisconnected(_) => warn!("Disconnected!"),

            _ => continue,
        }
    }

    Ok(())
}
