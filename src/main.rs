mod cli;
mod macros;
mod node_requests;
mod plugin;
mod requests;
mod server;
mod types;
mod utils;

use std::path::PathBuf;

use server::ServerConfig;
use utils::vfs;

pub use anyhow::Context as ErrorContext;
pub use anyhow::Result;
pub use once_cell;

fn main() -> Result<()> {
    let root = PathBuf::from("");
    let config: ServerConfig = if let Ok(env_config) = std::env::var("VX_CONFIG") {
        ServerConfig::from_str(&env_config)?
    } else {
        vfs::read_config(&root.join("config.json"))?
    };

    if let Ok(a) = std::env::var("AXIOM_NODE") {
        if a == "true" {
            config
                .build_req(&root, crate::server::Server::call_node_request)
                .run()?;

            return Ok(());
        }
    }

    config.build(&root).run()?;
    Ok(())
}
