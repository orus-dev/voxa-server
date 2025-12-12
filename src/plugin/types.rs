use serde::{Deserialize, Serialize};

use crate::types::message::{ClientMessage, WsMessage};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginJson {
    pub id: String,
    pub version: String,
    pub supported_versions: Vec<String>,
    pub file: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginHandshake {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum LoaderMessage {
    Request(WsMessage<ClientMessage>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum PluginMessage {}
