use interprocess::local_socket::{GenericNamespaced, ListenerOptions, prelude::*};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipeClientCommand {
    Presence(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipeServerCommand {
    Presence(String, String),
}

//================================================================

pub type ServerTX = Sender<PipeServerCommand>;
pub type ClientTX = Sender<PipeClientCommand>;
pub type ServerRX = Receiver<PipeServerCommand>;
pub type ClientRX = Receiver<PipeClientCommand>;

pub struct PipeServer {}

impl PipeServer {
    const SERVER_NAME: &str = "nimbus_pipe";

    pub fn update(&mut self) {}

    pub fn new() -> anyhow::Result<Self> {
        let name = Self::SERVER_NAME.to_ns_name::<GenericNamespaced>()?;
        let listener = ListenerOptions::new().name(name.clone()).create_sync()?;

        std::thread::spawn(move || while let Ok(_connection) = listener.accept() {});

        Ok(Self {})
    }
}

//================================================================
