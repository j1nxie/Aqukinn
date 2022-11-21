use std::{error::Error, sync::Arc};
use twilight_http::Client as HttpClient;
use twilight_model::id::{Id, marker::ChannelMarker};

pub async fn ping(
    http: Arc<HttpClient>,
    channel_id: Id<ChannelMarker>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    http.create_message(channel_id)
        .content("Pong!")?
        .await?;
    Ok(())
}
