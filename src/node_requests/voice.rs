use crate::{server::Server, utils::client::Client};
use std::sync::Arc;

crate::logger!(LOGGER "Voice chat");

pub fn join(server: &Arc<Server>, client: &Client, channel_id: &str) -> crate::Result<()> {
    let user_id = client.get_uuid()?;

    let voice_id = server
        .voice
        .lock()
        .unwrap()
        .set(user_id.clone(), channel_id.to_string());

    server.broadcast(crate::types::message::ServerMessage::VoiceJoin {
        user_id,
        channel_id: channel_id.to_string(),
        voice_id: voice_id,
    });

    Ok(())
}

pub fn leave(server: &Arc<Server>, client: &Client, channel_id: &str) -> crate::Result<()> {
    let user_id = client.get_uuid()?;

    let Some(voice_id) = server.voice.lock().unwrap().remove(channel_id, &user_id) else {
        return Ok(());
    };

    server.broadcast(crate::types::message::ServerMessage::VoiceLeave {
        user_id,
        channel_id: channel_id.to_string(),
        voice_id,
    });

    Ok(())
}

pub fn voice(server: &Arc<Server>, client: &Client, data: &[u8]) -> crate::Result<()> {
    let v = server.voice.lock().unwrap();
    let user_id = client.get_uuid()?;
    let Some((channel_id, voice_id)) = v.find_user(&user_id) else {
        return Ok(());
    };

    let mut targets = v.get(&channel_id);

    if let Some(pos) = targets.iter().position(|x| *x == &user_id) {
        targets.remove(pos);
    }

    let prefix = voice_id.to_le_bytes();
    let mut payload = Vec::with_capacity(prefix.len() + data.len());
    payload.extend_from_slice(&prefix);
    payload.extend_from_slice(data);

    server.broadcast_bin_to(&targets, payload)?;

    Ok(())
}
