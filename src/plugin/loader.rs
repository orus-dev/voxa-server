use crate::{
    logger,
    plugin::{
        Plugin,
        types::{PluginHandshake, PluginJson},
    },
};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};

logger!(LOGGER "Plugin Loader");

pub struct PluginLoader {
    plugin_clients: Arc<Mutex<HashMap<String, TcpStream>>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
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
        let a = self
            .plugin_clients
            .lock()
            .unwrap()
            .get(&plugin_json.id)
            .unwrap()
            .try_clone()
            .unwrap();
        Plugin(a.try_clone().unwrap(), BufReader::new(a))
    }

    pub fn start_server(&self) {
        let plugin_clients = self.plugin_clients.clone();
        let listener = TcpListener::bind("0.0.0.0:7243").unwrap();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                let mut reader = BufReader::new(&stream);
                let mut s = String::new();
                reader.read_line(&mut s).unwrap();
                let plugin_handshake: PluginHandshake = serde_json::from_str(&s).unwrap();
                plugin_clients
                    .lock()
                    .unwrap()
                    .insert(plugin_handshake.id, stream);
            }
        });
    }
}
