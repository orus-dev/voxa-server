pub mod chunk;
pub mod indicator;
pub mod message;
pub mod voice;

use std::sync::Arc;

use crate::{
    server::Server,
    types::message::{ClientMessage, ServerMessage, WsMessage},
    utils::client::Client,
};

impl Server {
    pub fn call_node_request(
        self: &Arc<Self>,
        req: &WsMessage<ClientMessage>,
        client: &Client,
    ) -> crate::Result<()> {
        match req {
            WsMessage::Message(req) => match req {
                ClientMessage::SendMessage {
                    channel_id,
                    contents,
                } => {
                    message::send(self, client, channel_id, contents)?;
                }

                ClientMessage::EditMessage {
                    message_id,
                    new_contents,
                } => message::edit(self, client, *message_id, new_contents)?,

                ClientMessage::DeleteMessage { message_id } => {
                    message::delete(self, client, *message_id)?
                }

                ClientMessage::LoadChunk {
                    chunk_id,
                    channel_id,
                } => chunk::load_chunk(self, client, channel_id, *chunk_id)?,

                ClientMessage::Typing { channel_id } => {
                    indicator::start_typing(self, client, channel_id)?
                }

                ClientMessage::JoinVoice { channel_id } => voice::join(self, client, channel_id)?,
                ClientMessage::LeaveVoice { channel_id } => voice::leave(self, client, channel_id)?,
            },

            WsMessage::Binary(data) => {
                // Self::LOGGER.info(format!("Binary message: {data:?}"));
                voice::voice(self, client, data)?;
            }

            WsMessage::String(s) => {
                Self::LOGGER.info(format!("String message: {s}"));
            }
        }

        Ok(())
    }

    pub fn broadcast_to(
        self: &Arc<Self>,
        targets: &[&String],
        message: ServerMessage,
    ) -> crate::Result<()> {
        for c in self.clients.lock().unwrap().iter() {
            if !targets.contains(&&c.get_uuid()?) {
                continue;
            }

            let c = c.clone();
            let server = self.clone();
            let message = message.clone();
            std::thread::spawn(move || {
                server
                    .wrap_err(&c, c.send(&message))
                    .expect("Failed to broadcast");
            });
        }

        Ok(())
    }
}
