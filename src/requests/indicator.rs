use crate::{server::Server, types::message::ServerMessage, utils::client::Client};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum Indicator {
    Typing { user_id: String, channel_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorContext {
    pub indicator: Indicator,
    pub expires: u16,
}

crate::logger!(LOGGER "Typing Indicator");

pub fn start_typing(server: &Arc<Server>, client: &Client, channel_id: &str) -> crate::Result<()> {
    server.broadcast(ServerMessage::Indicator(IndicatorContext {
        indicator: Indicator::Typing {
            user_id: client.get_uuid().context("Failed to get uuid")?,
            channel_id: channel_id.to_string(),
        },
        expires: 5, // 5 secs
    }));

    Ok(())
}
