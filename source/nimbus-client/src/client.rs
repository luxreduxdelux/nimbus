use std::time::Duration;

use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use tokio::net::TcpStream;

//================================================================

use nimbus_common::prelude::*;

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
            println!("[CLIENT::Update] {command:?}");

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
                CommandServer::PollVote(account, channel, message, choice) => {
                    self.server.poll_vote(account, channel, message, choice);
                }
                CommandServer::AccountChannel(index, channel) => {
                    self.server.set_account_channel(index, channel);
                }
                CommandServer::AccountPresence(index, presence) => {
                    self.server.set_account_presence(index, presence);
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
    tx: tokio::sync::mpsc::UnboundedSender<CommandClient>,
    rx: tokio::sync::mpsc::UnboundedReceiver<CommandServer>,
}

impl Thread {
    fn new(address: String, account: AccountConnect) -> Self {
        let (tx_s, rx) = tokio::sync::mpsc::unbounded_channel::<CommandServer>();
        let (tx, mut rx_s) = tokio::sync::mpsc::unbounded_channel::<CommandClient>();

        tokio::spawn(async move {
            let socket = tokio::time::timeout(
                Duration::from_secs(10),
                TcpStream::connect(format!("{address}:8080")),
            )
            .await;

            if let Ok(socket) = socket {
                if let Ok(mut socket) = socket {
                    CommandClient::Enter(account).write(&mut socket).await;

                    println!("[CLIENT] Connect: {socket:#?}");

                    loop {
                        tokio::select! {
                        Some(command) = rx_s.recv() => {
                            command.write(&mut socket).await;
                        }
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
                        }
                    }
                } else if let Err(error) = socket {
                    println!("{error}, {address}");
                    // TO-DO not actually a time-out error, replace with native socket error?
                    tx_s.send(CommandServer::Error(Error::Connect(ConnectError::TimeOut)));
                }
            } else {
                println!("time-out, {address}");
                tx_s.send(CommandServer::Error(Error::Connect(ConnectError::TimeOut)));
            }
        });

        Self { tx, rx }
    }
}
