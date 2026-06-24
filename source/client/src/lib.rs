pub use common;

//================================================================

use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use std::sync::mpsc;
use tokio::net::TcpStream;

//================================================================

use common::prelude::*;

//================================================================

pub struct Client {
    thread: Thread,
    pub server: Server,
    pub key: AccountKey,
    pub index: u64,
    pub ready: bool,
    pub error: Option<Error>,
}

impl Client {
    /// Create a new client.
    pub fn new(address: String, key: AccountKey, account: AccountConnect) -> Self {
        Self {
            thread: Thread::new(address, account),
            server: Server::default(),
            key,
            index: Default::default(),
            ready: Default::default(),
            error: Default::default(),
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
                CommandServer::Enter(index, server) => {
                    self.server = server;
                    self.index = index;
                    self.ready = true;
                }
                CommandServer::Leave => {
                    self.ready = false;
                }
                CommandServer::Nonce(challenge) => {
                    let key = SigningKey::from_bytes(&self.key);
                    let challenge = key.sign(&challenge);

                    self.thread
                        .tx
                        .send(CommandClient::Nonce(challenge.to_vec()))
                        .unwrap();
                }
                CommandServer::Message(message) => {
                    self.server.push_message(message.channel, message);
                }
                CommandServer::MessageDelete(channel, message) => {
                    self.server.delete_message(channel, message);
                }
                CommandServer::AccountChannel(index, channel) => {
                    self.server.set_account_channel(index, channel);
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

    pub fn send(&self, command: CommandClient) -> anyhow::Result<()> {
        Ok(self.thread.tx.send(command)?)
    }
}

//================================================================

struct Thread {
    tx: mpsc::Sender<CommandClient>,
    rx: mpsc::Receiver<CommandServer>,
}

impl Thread {
    fn new(address: String, account: AccountConnect) -> Self {
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
