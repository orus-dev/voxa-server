use crate::{
    logger,
    plugin::{
        Plugin,
        types::{PluginHandshake, PluginJson},
    },
    server::Server,
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

    pub fn load(&self, path: &Path) -> Plugin {
        let json_string = fs::read_to_string(path.join("plugin.json")).unwrap();
        let plugin_json: PluginJson = serde_json::from_str(&json_string).unwrap();
        LOGGER.info(format!("Loading {}", plugin_json.id));

        let child = Arc::new(Mutex::new(
            Command::new(plugin_json.file)
                .args(plugin_json.args)
                .current_dir(path)
                .spawn()
                .unwrap(),
        ));

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
        Plugin {
            stream: a.try_clone().unwrap(),
            reader: BufReader::new(a),
            id: plugin_json.id,
            child,
        }
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

    pub fn remove(&self, plugin_id: &str) {
        self.plugin_clients.lock().unwrap().remove(plugin_id);
    }

    pub fn clear(&self) {
        self.plugin_clients.lock().unwrap().clear();
    }

    pub fn load_all(&self, server: &Arc<Server>) {
        let path = server.root.join("plugins");
        if !path.exists() {
            fs::create_dir(&path).unwrap();
        }

        // Load plugins
        LOGGER.info("Loading plugins");
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let mut p = self.load(&path);
                let s = server.clone();
                server.plugins.lock().unwrap().push(p.clone());
                std::thread::spawn(move || p.run(&s));
            }
        }
        LOGGER.info("Plugins loaded");
    }
}

impl Clone for Plugin {
    fn clone(&self) -> Self {
        Plugin {
            stream: self.stream.try_clone().unwrap(),
            reader: BufReader::new(self.stream.try_clone().unwrap()),
            id: self.id.clone(),
            child: self.child.clone(),
        }
    }
}
