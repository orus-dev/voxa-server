use std::sync::Arc;

use anyhow::anyhow;

use crate::{server::Server, types, utils::client::Client};

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

    server.broadcast_to(
        &[&msg.channel_id, &msg.from],
        types::message::ServerMessage::MessageCreate(msg.clone()),
    )?;

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

    server.broadcast_to(
        &[&msg.channel_id, &msg.from],
        types::message::ServerMessage::MessageCreate(msg.clone()),
    )?;

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

    server.broadcast_to(
        &[&msg.channel_id, &msg.from],
        types::message::ServerMessage::MessageDelete { message_id },
    )?;

    Ok(())
}
