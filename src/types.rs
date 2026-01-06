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
    }
}

pub mod message {
    use serde::{Deserialize, Serialize};

    use crate::{
        requests::indicator::IndicatorContext,
        types::{
            Author,
            data::{self, Message},
        },
    };

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
        DeleteMessage {
            message_id: i64,
        },

        LoadChunk {
            channel_id: String,
            chunk_id: usize,
        },

        Typing {
            channel_id: String,
        },

        JoinVoice {
            channel_id: String,
        },

        LeaveVoice {
            channel_id: String,
        },
    }

    /// Messages sent *from the server* to the client
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "params", rename_all = "snake_case")]
    pub enum ServerMessage {
        /// Successful authentication
        Authenticated {
            uuid: Author,
            indicators: Vec<IndicatorContext>,
        },

        TempMessage {
            message: String,
        },

        /// A new message in a channel
        MessageCreate(data::Message),

        /// A message was edited
        MessageUpdate {
            message_id: i64,
            contents: String,
        },

        /// A message was deleted
        MessageDelete {
            message_id: i64,
        },

        /// Presence updates
        PresenceUpdate {
            user_id: Author,
            status: String,
        },

        /// Indicator
        Indicator(IndicatorContext),

        Shutdown {
            message: String,
        },

        Chunk(Vec<Message>),

        VoiceJoin {
            user_id: String,
            channel_id: String,
            voice_id: u16,
        },

        VoiceLeave {
            user_id: String,
            channel_id: String,
            voice_id: u16,
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
