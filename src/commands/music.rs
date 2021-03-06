use serenity::{
    client::Context,
    framework::standard::{
        Args,
        macros::command,
        CommandResult,
    },
    model::channel::Message,
    prelude::*,
};

use crate::Lavalink;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "you're not in a voice channel. why are you calling me into one then?").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird client placed in at initialization.").clone();
    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

    match handler {
        Ok(connection_info) => {
            let data = ctx.data.read().await;
            let lava_client = data.get::<Lavalink>().unwrap().clone();
            lava_client.create_session_with_songbird(&connection_info).await?;

            msg.channel_id
                .say(&ctx.http, &format!("joined voice channel: {}", connect_to.mention()))
                .await?;
            return Ok(());
        },
        Err(why) => {
            msg.channel_id.say(&ctx.http, format!("error joining channel: {}", why)).await?;
            return Ok(());
        }
    }
}

#[command]
#[only_in(guilds)]
#[aliases(dc)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let manager = songbird::get(ctx).await
        .expect("Songbird client placed in at initialization.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id.say(&ctx.http, format!("failed: {:?}", e)).await?;
        }

        {
            let data = ctx.data.read().await;
            let lava_client = data.get::<Lavalink>().unwrap().clone();
            lava_client.destroy(guild_id).await?;
        }

        msg.channel_id.say(&ctx.http, format!("left voice channel: {}", channel_id.unwrap().mention())).await?;
    } else {
        msg.reply(ctx, "i'm not in a voice channel.").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[min_args(1)]
#[aliases(p)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "error finding channel info.").await?;
            return Ok(());
        }
    };
    
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    let manager = songbird::get(ctx).await
        .expect("Songbird client placed in at initialization.").clone();

    if let Some(_handler) = manager.get(guild_id) {
        let query_info = lava_client.auto_search_tracks(&query).await?;

        if query_info.tracks.is_empty() {
            msg.channel_id.say(&ctx, "could not find any videos of the search query.").await?;
            return Ok(());
        }

        if query.contains("/playlist") {
            for track in query_info.tracks.clone() {
                if let Err(why) = &lava_client
                    .play(guild_id, track.clone())
                        .queue()
                        .await
                {
                    error!("{}", why);
                    return Ok(());
                };
                msg.channel_id.say(&ctx.http, format!("added to queue: **{}**", track.info.as_ref().unwrap().title)).await?;
            }
        } else {
                if let Err(why) = &lava_client
                    .play(guild_id, query_info.tracks[0].clone())
                        .queue()
                        .await
                {
                    error!("{}", why);
                    return Ok(());
                };
                msg.channel_id.say(&ctx.http, format!("added to queue: **{}**", query_info.tracks[0].info.as_ref().unwrap().title)).await?;
        }
    } else {
        msg.channel_id.say(&ctx.http, format!("use `a!join` first, i'm not in a voice channel, baka.")).await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    match lava_client.stop(msg.guild_id.unwrap()).await {
        Ok(_) => {
            msg.channel_id.say(&ctx.http, "there, i stopped the player for you.").await?;
            return Ok(());
        },
        _ => {
            msg.channel_id.say(&ctx.http, "there's nothing to stop. the player isn't even playing, what do you mean?").await?;
            return Ok(());
        }
    }
}

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        match &node.is_paused {
            true => {
                msg.channel_id.say(&ctx.http, "it's already paused, what do you mean?").await?;
                return Ok(());
            },
            false => {
                lava_client.pause(msg.guild_id.unwrap()).await?;  
                msg.channel_id.say(&ctx.http, "okay, i paused the music for you.").await?;
                return Ok(());
            },
        }
    } else {
        lava_client.pause(msg.guild_id.unwrap()).await?;  
        msg.channel_id.say(&ctx.http, "okay, i paused the music for you.").await?;
        return Ok(());
    }
}

#[command]
#[only_in(guilds)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();
    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        match &node.is_paused {
            true => {
                lava_client.pause(msg.guild_id.unwrap()).await?;  
                msg.channel_id.say(&ctx.http, "okay fine, i'll put the music back on for you.").await?;
                return Ok(());
            },
            false => {
                msg.channel_id.say(&ctx.http, "it's still playing, baka.").await?;
                return Ok(());
            },
        }
    } else {
        msg.channel_id.say(&ctx.http, "it's still playing, baka.").await?;
        return Ok(());
    }
}

#[command]
#[only_in(guilds)]
#[aliases(np)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            msg.channel_id.say(&ctx.http, format!("now playing: **{}**", track.track.info.as_ref().unwrap().title)).await?;
        } else {
            msg.channel_id.say(&ctx.http, "nothing is playing at the moment.").await?;
        }
    } else {
        msg.channel_id.say(&ctx.http, "nothing is playing at the moment.").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if query.is_empty() {
        if let Some(track) = lava_client.skip(msg.guild_id.unwrap()).await {
            msg.channel_id.say(&ctx.http, format!("skipped: **{}**", track.track.info.as_ref().unwrap().title)).await?;
            return Ok(());
        } else {
            msg.channel_id.say(&ctx.http, "nothing to skip.").await?;
            return Ok(());
        }
    }

    match query.parse::<u32>() {
        Ok(i) => {
            for _ in 0..i {
                if let Some(track) = lava_client.skip(msg.guild_id.unwrap()).await {
                    msg.channel_id.say(&ctx.http, format!("skipped: **{}**", track.track.info.as_ref().unwrap().title)).await?;
                } else {
                    msg.channel_id.say(&ctx.http, "nothing to skip.").await?;
                }
            }
            return Ok(());
        },
        Err(_) => {
            msg.channel_id.say(&ctx.http, "**error:** you have to type in a valid number!! >//<").await?;
            return Ok(());
        }
    }
}

#[command]
#[only_in(guilds)]
#[aliases(cq, clearqueue)]
async fn clear_queue(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>().unwrap().clone();

    if let Some(mut node) = lava_client.nodes().await.get_mut(&msg.guild_id.unwrap().0) {
        match &node.queue.is_empty() {
            true => {
                msg.channel_id.say(&ctx.http, "there's nothing in the queue to clear...").await?;
                return Ok(());
            },
            false => {
                msg.channel_id.say(&ctx.http, format!("cleared **{}** tracks from the queue for you.", &node.queue.len())).await?;
                node.queue.clear();
                return Ok(());
            },
        }
    } else {
        msg.channel_id.say(&ctx.http, "there's nothing in the queue to clear...").await?;
        return Ok(());
    }
}
