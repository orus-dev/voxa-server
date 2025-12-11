use std::{
    collections::HashSet,
    fs,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{
    logger,
    plugin::loader::PluginLoader,
    types,
    utils::{self, client::Client},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub server_name: String,
    pub server_id: String,
    pub server_key: String,
    pub port: u16,
    pub channels: Vec<types::data::Channel>,
}

#[allow(dead_code)]
pub struct Server {
    pub root: PathBuf,
    pub config: ServerConfig,
    pub clients: Mutex<HashSet<Client>>,
    pub db: utils::database::Database,
    pub plugin_loader: PluginLoader,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7080,
            server_name: format!("Server Name"),
            server_id: format!("important"),
            server_key: format!("important"),
            channels: Vec::new(),
        }
    }
}

impl ServerConfig {
    pub fn build(self, root: &Path) -> Arc<Server> {
        Server::new_config(root, self)
    }

    pub fn from_str(s: &str) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl Server {
    logger!(LOGGER "Server");

    pub fn new_config(root: &Path, config: ServerConfig) -> Arc<Self> {
        Arc::new(Self {
            db: utils::database::Database::new(&config).unwrap(),
            root: root.to_path_buf(),
            config,
            clients: Mutex::new(HashSet::new()),
            plugin_loader: PluginLoader::new(),
        })
    }

    pub fn run(self: &Arc<Self>) -> crate::Result<()> {
        // Start plugin loader
        Self::LOGGER.info("Starting loader");
        self.plugin_loader.start_server();

        // Load plugins
        Self::LOGGER.info("Loading plugins");
        for entry in fs::read_dir(self.root.join("plugins"))? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                self.plugin_loader.load(&path);
            }
        }
        Self::LOGGER.info("Plugins loaded");

        // Initialize plugins
        Self::LOGGER.info("Initializing plugins");

        // Start server
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.port))?;
        Self::LOGGER.info(format!("Server listening at 0.0.0.0:{}", self.config.port));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    std::thread::spawn({
                        let srv = self.clone();

                        move || {
                            let Some(client) = Self::LOGGER
                                .extract(srv.init_client(stream), "Failed to initialize client")
                            else {
                                return;
                            };

                            Self::LOGGER.extract(
                                srv.wrap_err(&client, srv.handle_client(&client)),
                                "Client handler failed",
                            );
                        }
                    });
                }
                Err(e) => {
                    Self::LOGGER.error(format!("Connection failed: {e}"));
                }
            }
        }

        Ok(())
    }

    fn init_client(self: &Arc<Self>, stream: TcpStream) -> anyhow::Result<Client> {
        Self::LOGGER.info(format!("New connection: {}", stream.peer_addr()?));
        // Initialize client
        let mut client = Client::new(stream)?;

        // Initialize handshake
        self.wrap_err(
            &client,
            client.send(types::handshake::ServerDetails {
                name: self.config.server_name.clone(),
                id: self.config.server_id.clone(),
                version: format!("0.0.1"),
                channels: self.config.channels.clone(),
            }),
        )?;

        match self.wrap_err(&client, client.read_t::<types::handshake::ClientDetails>())? {
            Some(types::message::WsMessage::Message(types::handshake::ClientDetails {
                auth_token,
                last_message,
                ..
            })) => {
                let auth_res = utils::auth::auth(self, &mut client, &auth_token);
                let uuid = self.wrap_err(&client, auth_res)?;
                self.wrap_err(
                    &client,
                    client.send(types::message::ServerMessage::Authenticated {
                        uuid,
                        messages: if let Some(i) = last_message {
                            self.wrap_err(&client, self.db.get_messages_after_id(i))?
                        } else {
                            self.wrap_err(&client, self.db.get_messages_after_id(0))?
                        },
                    }),
                )?;
            }
            Some(v) => {
                self.wrap_err(
                    &client,
                    client.send(types::message::ResponseError::InvalidHandshake(format!(
                        "Invalid handshake: {v:?}"
                    ))),
                )?;
            }
            None => {}
        }

        // Insert to the set of all connected clients
        self.clients.lock().unwrap().insert(client.clone());

        Ok(client)
    }

    fn handle_client(self: &Arc<Self>, client: &Client) -> anyhow::Result<()> {
        // The main req/res loop
        'outer: loop {
            let req = client.read()?;
            if let Some(r) = &req {
                // for p in self.plugins.lock().unwrap().iter_mut() {
                //     if p.on_request(r, client, self) {
                //         continue 'outer;
                //     }
                // }

                self.wrap_err(&client, self.call_request(r, &client))?;
            }
        }
    }

    /// When there is a error it removes the client
    pub fn wrap_err<T, E: std::fmt::Display>(
        self: &Arc<Self>,
        client: &Client,
        res: std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        if let Err(e) = &res {
            self.clients.lock().unwrap().remove(&client);
            if client
                .send(types::message::ResponseError::InternalError(e.to_string()))
                .is_err()
            {}
        }

        res
    }
}
