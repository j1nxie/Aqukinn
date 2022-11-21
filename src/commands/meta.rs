use twilight_model::channel::Message;
use crate::State;

pub async fn ping(msg: Message, state: State) -> anyhow::Result<()> {
	let shard_latency = state.shard.info()?.latency().average();
	match shard_latency {
		Some(s) => 
			state
			.http
			.create_message(msg.channel_id)
			.content(&format!("Pong! Shard latency is {:?}", s))?
			.await?,

		None => 
			state
			.http
			.create_message(msg.channel_id)
			.content("Pong! Shard latency not available yet!")?
			.await?,
	};

	Ok(())
}
