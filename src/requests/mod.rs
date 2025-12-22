pub mod message;

use std::sync::Arc;

use crate::{
    server::Server,
    types::message::{ClientMessage, ServerMessage, WsMessage},
    utils::client::Client,
};

impl Server {
    pub fn call_request(
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
            },

            WsMessage::Binary(b) => {
                Self::LOGGER.info(format!("Binary message: {b:?}"));
            }

            WsMessage::String(s) => {
                Self::LOGGER.info(format!("String message: {s}"));
            }
        }

        Ok(())
    }

    pub fn broadcast(self: &Arc<Self>, msg: ServerMessage) {
        for c in self.clients.lock().unwrap().iter() {
            let c = c.clone();
            let server = self.clone();
            let msg = msg.clone();
            std::thread::spawn(move || {
                server
                    .wrap_err(&c, c.send(msg))
                    .expect("Failed to broadcast");
            });
        }
    }
}
