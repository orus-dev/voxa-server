use std::{
    collections::HashSet,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

const WELCOME: &str = "\x1b[38;2;169;86;252m
    _          _                 
   / \\   __  _(_) ___  _ __ ___  
  / _ \\  \\ \\/ / |/ _ \\| '_ ` _ \\ 
 / ___ \\  >  <| | (_) | | | | | |
/_/   \\_\\/_/\\_\\_|\\___/|_| |_| |_|
\x1b[0m";

use crate::{
    cli, logger,
    plugin::{Plugin, loader::PluginLoader, types::LoaderMessage},
    requests::voice,
    types::{
        self,
        message::{ClientMessage, WsMessage},
    },
    utils::{self, auth, client::Client, voice::Voice},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub server_name: String,
    pub server_id: String,
    pub server_key: String,
    pub port: u16,
    pub channels: Vec<types::data::Channel>,
}

pub struct Server {
    pub root: PathBuf,
    pub config: ServerConfig,
    pub clients: Mutex<HashSet<Client>>,
    pub plugins: Mutex<Vec<Plugin>>,
    pub db: utils::database::Database,
    pub shutting_down: AtomicBool,
    pub indicators: Mutex<Vec<crate::requests::indicator::IndicatorContext>>,
    pub voice: Mutex<crate::utils::voice::Voice>,
    pub call_request: fn(&Arc<Self>, &WsMessage<ClientMessage>, &Client) -> crate::Result<()>,
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

    pub fn build_req(
        self,
        root: &Path,
        req: fn(&Arc<Server>, &WsMessage<ClientMessage>, &Client) -> crate::Result<()>,
    ) -> Arc<Server> {
        Server::new_req_config(root, req, self)
    }

    pub fn from_str(s: &str) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl Server {
    logger!(LOGGER "Server");

    pub fn new_req_config(
        root: &Path,
        call_request: fn(&Arc<Self>, &WsMessage<ClientMessage>, &Client) -> crate::Result<()>,
        config: ServerConfig,
    ) -> Arc<Self> {
        Arc::new(Self {
            db: utils::database::Database::new(&config).unwrap(),
            root: root.to_path_buf(),
            config,
            clients: Mutex::new(HashSet::new()),
            plugins: Mutex::new(Vec::new()),
            shutting_down: AtomicBool::new(false),
            indicators: Mutex::new(Vec::new()),
            voice: Mutex::new(Voice::new()),
            call_request: call_request,
        })
    }

    pub fn new_config(root: &Path, config: ServerConfig) -> Arc<Self> {
        Arc::new(Self {
            db: utils::database::Database::new(&config).unwrap(),
            root: root.to_path_buf(),
            config,
            clients: Mutex::new(HashSet::new()),
            plugins: Mutex::new(Vec::new()),
            shutting_down: AtomicBool::new(false),
            indicators: Mutex::new(Vec::new()),
            voice: Mutex::new(Voice::new()),
            call_request: Self::call_server_request,
        })
    }

    pub fn run(self: &Arc<Self>) -> crate::Result<()> {
        // Start plugin loader
        let plugin_loader = PluginLoader::new();
        Self::LOGGER.info("Starting loader");
        plugin_loader.start_server();

        // Load plugins
        plugin_loader.load_all(self);

        // Initialize plugins
        Self::LOGGER.info("Initializing plugins");

        Self::LOGGER.info("Authenticating");
        auth::test(self)?;

        // Initialize indicators
        Self::LOGGER.info("Initializing indicators");
        self.spawn_indicator_thread();

        // Initialize CLI
        Self::LOGGER.info("Initializing CLI");
        cli::start_cli(self.clone(), plugin_loader);

        // Start server
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.port))?;
        Self::LOGGER.info(format!("Server listening at 0.0.0.0:{}", self.config.port));

        println!(
            "{WELCOME}\nversion {}\nType 'help' to see available commands.",
            env!("CARGO_PKG_VERSION")
        );

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

    fn init_client(self: &Arc<Self>, stream: TcpStream) -> crate::Result<Client> {
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
                ..
            })) => {
                let auth_res = utils::auth::auth(self, &mut client, &auth_token);
                let uuid = self.wrap_err(&client, auth_res)?;
                self.wrap_err(
                    &client,
                    client.send(types::message::ServerMessage::Authenticated {
                        uuid,
                        indicators: self.indicators.lock().unwrap().clone(),
                        voice_chat: self.voice.lock().unwrap().get_connections(),
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

    fn handle_client(self: &Arc<Self>, client: &Client) -> crate::Result<()> {
        // The main req/res loop
        while !self.shutting_down.load(Ordering::SeqCst) {
            let req = client.read()?;
            if let Some(r) = &req {
                match r {
                    WsMessage::Binary(_) => {
                        // ignore binary
                    }
                    _ => {
                        self.send_plugin_message(&LoaderMessage::Request {
                            user_id: client.get_uuid().unwrap_or_default(),
                            msg: r.clone(),
                        })?;
                    }
                }

                self.wrap_err(&client, (self.call_request)(self, r, &client))?;
            }
        }
        Ok(())
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
            {
                let Ok(user_id) = client.get_uuid() else {
                    return res;
                };

                let channel_id = {
                    let v = self.voice.lock().unwrap();
                    let Some((channel_id, _)) = v.find_user(&user_id) else {
                        return res;
                    };
                    channel_id.clone()
                };

                voice::leave(self, client, &channel_id).unwrap();
            }
        }

        res
    }

    pub fn send_plugin_message(self: &Arc<Self>, msg: &LoaderMessage) -> crate::Result<()> {
        for p in self.plugins.lock().unwrap().iter_mut() {
            p.send(msg)?;
        }
        Ok(())
    }

    pub fn shutdown(self: &Arc<Self>) {
        Self::LOGGER.info("Server shutting down...");

        // Signal shutdown
        self.shutting_down.store(true, Ordering::SeqCst);

        // Disconnect clients
        let clients = self.clients.lock().unwrap();
        for client in clients.iter() {
            let _ = client.send(types::message::ServerMessage::Shutdown {
                message: format!("Server shutting down... we'll be back shortly"),
            });
            let _ = client.close();
        }

        // Stop plugins
        for plugin in self.plugins.lock().unwrap().iter_mut() {
            if let Err(e) = plugin.stop() {
                Self::LOGGER.warn(e.context("Couldn't stop plugin"));
            }
        }

        Self::LOGGER.info("Shutdown complete");
        Self::LOGGER.info("Exiting process..");
        std::process::exit(0);
    }
}
