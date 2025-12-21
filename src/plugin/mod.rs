pub mod loader;
pub mod types;

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::Child,
    sync::{Arc, Mutex},
};

use crate::{
    plugin::types::{LoaderMessage, PluginMessage},
    server::Server,
    types::message::ServerMessage,
};

pub struct Plugin {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    id: String,
    child: Arc<Mutex<Child>>,
}

impl Plugin {
    pub fn send(&mut self, m: &LoaderMessage) -> crate::Result<()> {
        self.stream
            .write(serde_json::to_string(&m).unwrap().as_bytes())?;
        Ok(())
    }

    pub fn read(&mut self) -> crate::Result<PluginMessage> {
        let mut buf = String::new();
        self.reader.read_line(&mut buf)?;
        Ok(serde_json::from_str(&buf)?)
    }

    pub fn run(&mut self, server: &Arc<Server>) -> crate::Result<()> {
        loop {
            match self.read()? {
                PluginMessage::SendMessage {
                    channel_id,
                    contents,
                } => {
                    let msg = server.db.insert_message(
                        &channel_id,
                        &self.id,
                        &contents,
                        chrono::Utc::now().timestamp(),
                    )?;

                    for c in server.clients.lock().unwrap().iter() {
                        let c = c.clone();
                        let server = server.clone();
                        let msg = msg.clone();
                        std::thread::spawn(move || {
                            server
                                .wrap_err(&c, c.send(ServerMessage::MessageCreate(msg)))
                                .expect("Failed to broadcast");
                        });
                    }
                }
            }
        }
    }

    pub fn stop(&mut self) -> crate::Result<()> {
        self.send(&LoaderMessage::Shutdown)?;
        self.stream.shutdown(std::net::Shutdown::Both)?;
        self.child.lock().unwrap().kill()?;
        Ok(())
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
