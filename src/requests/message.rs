use std::sync::Arc;

use serde_json::ser;

use crate::{plugin::types::LoaderMessage, server::Server, types, utils::client::Client};

crate::logger!(LOGGER "Message Manager");

pub fn send(
    server: &Arc<Server>,
    client: &Client,
    channel_id: &str,
    contents: &str,
) -> crate::Result<()> {
    LOGGER.info(format!("SendMessage to {channel_id}: {contents}"));

    if contents.is_empty() {
        client.send(types::message::ResponseError::InvalidRequest(format!(
            "Invalid message: empty message"
        )))?;

        return Ok(());
    }

    let msg = server.db.insert_message(
        &channel_id,
        &client.get_uuid()?,
        &contents,
        chrono::Utc::now().timestamp(),
    )?;

    let server = server.clone();

    for c in server.clients.lock().unwrap().iter() {
        let c = c.clone();
        let server = server.clone();
        let msg = msg.clone();
        std::thread::spawn(move || {
            server
                .wrap_err(
                    &c,
                    c.send(types::message::ServerMessage::MessageCreate(msg)),
                )
                .expect("Failed to broadcast");
        });
    }

    server.send_plugin_message(&LoaderMessage::MessageSent {
        user_id: client.get_uuid().unwrap_or_default(),
        msg: msg,
    });

    Ok(())
}

pub fn edit(
    _server: &Arc<Server>,
    _client: &Client,
    message_id: usize,
    new_contents: &str,
) -> crate::Result<()> {
    LOGGER.info(format!("EditMessage {message_id}: {new_contents}"));
    Ok(())
}

pub fn delete(_server: &Arc<Server>, _client: &Client, message_id: usize) -> crate::Result<()> {
    LOGGER.info(format!("DeleteMessage {message_id}"));
    Ok(())
}
