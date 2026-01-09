use crate::{requests::indicator::IndicatorContext, server::Server, types, utils::client::Client};
use std::sync::Arc;

pub fn start_typing(server: &Arc<Server>, client: &Client, channel_id: &str) -> crate::Result<()> {
    let user_id = client.get_uuid()?;

    server.broadcast_to(
        &[&user_id.clone(), &channel_id.to_string()],
        types::message::ServerMessage::Indicator(IndicatorContext {
            indicator: crate::requests::indicator::Indicator::Typing {
                user_id,
                channel_id: channel_id.to_string(),
            },
            expires: 2, // 2 secs
        }),
    )?;

    Ok(())
}
