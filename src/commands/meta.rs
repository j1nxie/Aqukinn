use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{
        Args,
        macros::{command, help},
        CommandResult,
        CommandGroup,
        HelpOptions,
        help_commands
    },
    model::prelude::*,
    prelude::*
};
use std::collections::HashSet;
use crate::ShardManagerContainer;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "there was a problem getting the shard manager.").await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "no shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("pong! the shard latency is {:?}", runner.latency)).await?;

    Ok(())
}

#[help]
#[individual_command_tip = "test"]
#[command_not_found_text = "could not find: `{}`"]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>)
-> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
