use serenity::prelude::*;
use serenity::{
    framework::standard::{
        CommandResult,
        DispatchError,
        macros::hook
    },
    model::prelude::*
};
use crate::CommandCounter;

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    info!("got command '{}' by user '{}'", command_name, msg.author.name);

    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>()
        .expect("expected CommandCounter in TypeMap");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
}

#[hook]
pub async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => info!("processed command: {}", command_name),
        Err(why) => error!("command '{}' returned error: {:?}", command_name, why)
    }
}

#[hook]
pub async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    error!("could not find command: {}", unknown_command_name);
}

#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("try this again in {} seconds", info.as_secs()))
                .await;
        }
    }
}
