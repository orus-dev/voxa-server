use crate::{logger, plugin::Plugin};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    net::{TcpListener, TcpStream},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};

logger!(LOGGER "Plugin Loader");

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginJson {
    pub id: String,
    pub version: String,
    pub supported_versions: Vec<String>,
    pub file: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginHandshake {
    pub id: String,
}

pub struct PluginLoader {
    plugins: Vec<Plugin>,
    plugin_clients: Arc<Mutex<HashMap<String, TcpStream>>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn load(&self, json_path: &Path) -> Plugin {
        let json_string = fs::read_to_string(json_path).unwrap();
        let plugin_json: PluginJson = serde_json::from_str(&json_string).unwrap();
        LOGGER.info(format!("Loading {}", plugin_json.id));

        Command::new(plugin_json.file)
            .args(plugin_json.args)
            .current_dir("plugins")
            .spawn()
            .unwrap();

        while self
            .plugin_clients
            .lock()
            .unwrap()
            .get(&plugin_json.id)
            .is_none()
        {}
        LOGGER.info(format!("Loaded {}", plugin_json.id));
        Plugin(
            self.plugin_clients
                .lock()
                .unwrap()
                .get(&plugin_json.id)
                .unwrap()
                .try_clone()
                .unwrap(),
        )
    }

    pub fn start_server(&self) {
        let plugin_clients = self.plugin_clients.clone();
        let listener = TcpListener::bind("0.0.0.0:7243").unwrap();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut s = String::new();
                stream.read_to_string(&mut s).unwrap();
                let plugin_handshake: PluginHandshake = serde_json::from_str(&s).unwrap();
                plugin_clients
                    .lock()
                    .unwrap()
                    .insert(plugin_handshake.id, stream);
            }
        });
    }
}
