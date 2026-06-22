pub use common;

//================================================================

use std::sync::mpsc;
use tokio::net::TcpStream;

//================================================================

use common::prelude::*;

//================================================================

pub struct Client {
    thread: Thread,
    pub server: Server,
    pub ready: bool,
    pub error: Option<Error>,
}

impl Client {
    /// Create a new client.
    pub fn new(address: String, account: Account) -> Self {
        Self {
            thread: Thread::new(address, account.clone()),
            server: Server::default(),
            ready: false,
            error: None,
        }
    }

    /// Update the client state, polling for any new command from the server.
    pub fn update<F: FnMut(CommandServer)>(&mut self, mut call: F) -> anyhow::Result<()> {
        while let Ok(command) = self.thread.rx.try_recv() {
            //println!("[CLIENT::Update] {command:?}");

            call(command.clone());

            match command {
                CommandServer::Error(error) => {
                    self.error = Some(error);
                }
                CommandServer::Enter(server) => {
                    self.server = server;
                    self.ready = true;
                }
                CommandServer::Leave => {
                    self.ready = false;
                }
                CommandServer::Message(message) => {
                    // TO-DO use channel index
                    self.server.push_message(0, message);
                }
                CommandServer::AccountState(index, state) => {
                    self.server.set_account_state(index, state);
                }
                CommandServer::AccountWrite(index, write) => {
                    self.server.set_account_write(index, write);
                }
            }
        }

        Ok(())
    }

    pub fn send_text(&self, text: &str) {
        self.thread
            .tx
            .send(CommandClient::Message(MessageKind::Text(text.to_string())))
            .unwrap();
    }

    pub fn send_file(&self, name: &str, file: Vec<u8>) {
        self.thread
            .tx
            .send(CommandClient::Message(MessageKind::File(
                name.to_string(),
                file,
            )))
            .unwrap();
    }

    pub fn send_sticker(&self, sticker: u64) {
        self.thread
            .tx
            .send(CommandClient::Message(MessageKind::Sticker(sticker)))
            .unwrap();
    }

    pub fn set_state(&self, state: AccountState) {
        self.thread
            .tx
            .send(CommandClient::AccountState(state))
            .unwrap();
    }

    pub fn set_write(&self, write: bool) {
        self.thread
            .tx
            .send(CommandClient::AccountWrite(write))
            .unwrap();
    }
}

//================================================================

struct Thread {
    tx: mpsc::Sender<CommandClient>,
    rx: mpsc::Receiver<CommandServer>,
}

impl Thread {
    fn new(address: String, account: Account) -> Self {
        let (tx_s, rx) = mpsc::channel::<CommandServer>();
        let (tx, rx_s) = mpsc::channel::<CommandClient>();
        tokio::spawn(async move {
            let socket = TcpStream::connect(format!("{address}:8080")).await;

            if let Ok(mut socket) = socket {
                CommandClient::Enter(account).write(&mut socket).await;

                println!("[CLIENT] Connect: {socket:#?}");

                loop {
                    tokio::select! {
                        // Server -> client command.
                        result = CommandServer::read(&mut socket) => {
                            match result {
                                Ok(command) => {
                                    tx_s.send(command).unwrap();
                                }
                                Err(err) => {
                                    tx_s.send(CommandServer::Leave);
                                    eprintln!("read error: {err}");
                                    break;
                                }
                            }
                        }

                        // Client -> server command.
                        _ = tokio::task::yield_now() => {
                            while let Ok(command) = rx_s.try_recv() {
                                command.write(&mut socket).await;
                            }
                        }
                    }
                }
            } else if let Err(error) = socket {
                println!("{error}, {address}");
                tx_s.send(CommandServer::Error(Error::Connect));
            }
        });

        Self { tx, rx }
    }
}
