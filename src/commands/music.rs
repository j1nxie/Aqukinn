use std::str::FromStr;
use twilight_model::channel::Message;
use crate::State;

pub async fn join(msg: Message, state: State) -> anyhow::Result<()> {
	let author_id = msg.author.id;
	let guild_id = msg.content.parse()?;
	
	Ok(())
}

pub async fn play(msg: Message, state: State) -> anyhow::Result<()> {
	Ok(())
}
