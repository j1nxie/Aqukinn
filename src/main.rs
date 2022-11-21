#![allow(non_snake_case)]

use futures::stream::StreamExt;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Event};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use dotenv::dotenv;
use tracing::{error, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracer subscriber
    let tracing_subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(tracing_subscriber)
        .expect("Setting tracing default failed.");

    // initialize Discord token
    dotenv().ok();
    let token: String;
    if let Ok(s) = env::var("DISCORD_TOKEN") {
        token = s;
    } else {
        error!("Discord token not found!");
        panic!();
    }

    let (cluster, mut events) = Cluster::new(token.clone(), Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT).await?;
    let cluster = Arc::new(cluster);

    let cluster_spawn = Arc::clone(&cluster);

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = Arc::new(HttpClient::new(token));

    let cache = InMemoryCache::builder().resource_types(ResourceType::MESSAGE).build();

    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }
    
    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            http.create_message(msg.channel_id)
                .content("Pong!")?
                .await?;
        },

        Event::ShardConnected(_) => {
            info!("Connected on shard {shard_id}.");
        },

        Event::ShardDisconnected(_) => {
            warn!("Shard {shard_id} disconnected!");
        }

        _ => {},
    }

    Ok(())
}
