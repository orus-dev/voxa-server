use crate::{server::Server, types::message::ServerMessage, utils::client::Client};
use std::sync::Arc;

crate::logger!(LOGGER "Chunk Loader");

pub fn load_chunk(
    server: &Arc<Server>,
    client: &Client,
    channel_id: &str,
    chunk_id: usize,
) -> crate::Result<()> {
    let mut chunk = server
        .db
        .get_chunk_node(&client.get_uuid()?, channel_id, chunk_id)?;
    chunk.reverse();
    client.send(ServerMessage::Chunk(chunk))?;
    Ok(())
}
