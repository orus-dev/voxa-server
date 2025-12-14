pub mod loader;
pub mod types;

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use crate::plugin::types::{LoaderMessage, PluginMessage};

pub struct Plugin(TcpStream, BufReader<TcpStream>);

impl Plugin {
    pub fn send(&mut self, m: &LoaderMessage) {
        self.0
            .write(serde_json::to_string(&m).unwrap().as_bytes())
            .unwrap();
    }

    pub fn read(&mut self) -> PluginMessage {
        let mut buf = String::new();
        self.1.read_line(&mut buf).unwrap();
        serde_json::from_str(&buf).unwrap()
    }
}
