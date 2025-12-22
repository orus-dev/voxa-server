use std::sync::Arc;

use anyhow::Context;
use serde::Deserialize;

use crate::logger;
use crate::server::Server;
use crate::utils::client::Client;

logger!(LOGGER "Auth");

#[derive(Debug, Deserialize)]
struct AuthApiRes {
    user_id: String,
}

pub fn auth(server: &Arc<Server>, client: &mut Client, token: &str) -> crate::Result<String> {
    let mut res = ureq::get(format!(
        "https://vxchat.netlify.app/api/auth?token={token}&key={}",
        server.config.server_key
    ))
    .call()
    .context("Failed to authenticate")?;
    let api_res: AuthApiRes = serde_json::from_str(&res.body_mut().read_to_string()?)?;
    client.set_uuid(&api_res.user_id);
    LOGGER.info(format!("{} successfully authenticated", api_res.user_id));
    Ok(api_res.user_id)
}

pub fn test(server: &Arc<Server>) -> crate::Result<()> {
    ureq::get(format!(
        "https://vxchat.netlify.app/api/auth-test?key={}&id={}",
        server.config.server_key, server.config.server_id
    ))
    .call()
    .context("Failed to authenticate")?;

    Ok(())
}
