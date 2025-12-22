use std::sync::Arc;

use anyhow::anyhow;

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
    })?;

    Ok(())
}

pub fn edit(
    server: &Arc<Server>,
    client: &Client,
    message_id: i64,
    new_contents: &str,
) -> crate::Result<()> {
    LOGGER.info(format!("EditMessage {message_id}: {new_contents}"));
    let Some(msg) = server.db.get_message_by_id(message_id)? else {
        return Err(anyhow!("Message does not exist"));
    };

    if msg.from != client.get_uuid()? {
        return Err(anyhow!("You are not the author of this message"));
    }

    server.db.edit_message(message_id, new_contents)?;
    Ok(())
}

pub fn delete(server: &Arc<Server>, client: &Client, message_id: i64) -> crate::Result<()> {
    LOGGER.info(format!("DeleteMessage {message_id}"));
    let Some(msg) = server.db.get_message_by_id(message_id)? else {
        return Err(anyhow!("Message does not exist"));
    };

    if msg.from != client.get_uuid()? {
        return Err(anyhow!("You are not the author of this message"));
    }

    server.db.delete_message(message_id)?;
    Ok(())
}
