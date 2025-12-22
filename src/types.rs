pub type Author = String;

/// Shared data structures
pub mod data {
    use serde::{Deserialize, Serialize};

    use crate::types::Author;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub id: i64,
        pub channel_id: String,
        pub from: Author,
        pub contents: String,
        pub timestamp: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel {
        pub id: String,
        pub name: String,
        pub kind: ChannelKind,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[serde(untagged)]
    pub enum ChannelKind {
        Text,
        Voice,
        IFrame(String),
    }
}

pub mod handshake {
    use serde::{Deserialize, Serialize};

    use crate::types::data::Channel;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServerDetails {
        pub version: String,
        pub name: String,
        pub id: String,
        pub channels: Vec<Channel>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClientDetails {
        pub version: String,
        pub auth_token: String,
        pub last_message: Option<i64>,
    }
}

pub mod message {
    use serde::{Deserialize, Serialize};

    use crate::types::{Author, data};

    /// Messages sent *from the client* (user’s app) to the server
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "params", rename_all = "snake_case")]
    pub enum ClientMessage {
        /// Send a message to a channel
        SendMessage {
            channel_id: String,
            contents: String,
        },

        /// Edit a message (if allowed)
        EditMessage {
            message_id: i64,
            new_contents: String,
        },

        /// Delete a message (if allowed)
        DeleteMessage { message_id: i64 },
    }

    /// Messages sent *from the server* to the client
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "params", rename_all = "snake_case")]
    pub enum ServerMessage {
        /// Successful authentication
        Authenticated {
            uuid: Author,
            messages: Vec<data::Message>,
        },

        TempMessage {
            message: String,
        },

        /// A new message in a channel
        MessageCreate(data::Message),

        /// A message was edited
        MessageUpdate(data::Message),

        /// A message was deleted
        MessageDelete {
            channel_id: String,
            message_id: i64,
        },

        /// Presence updates
        PresenceUpdate {
            user_id: Author,
            status: String,
        },

        /// Typing indicator
        Typing {
            user_id: Author,
            channel_id: String,
        },

        Shutdown {
            message: String,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "error", content = "message", rename_all = "snake_case")]
    pub enum ResponseError {
        InvalidRequest(String),
        InvalidHandshake(String),
        Unauthorized(String),
        NotFound(String),
        InternalError(String),
    }

    /// WebSocket wrapper
    #[derive(Debug, Clone, Serialize)]
    #[serde(tag = "type", content = "params", rename_all = "snake_case")]
    pub enum WsMessage<T: Serialize + for<'de> Deserialize<'de>> {
        Message(T),
        Binary(Vec<u8>),
        String(String),
    }
}
