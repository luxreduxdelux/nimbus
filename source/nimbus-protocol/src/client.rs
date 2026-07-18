use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use std::time::Duration;
use tokio::net::TcpStream;

//================================================================

use crate::account::*;
use crate::cache::*;
use crate::command::*;
use crate::server::*;

//================================================================

pub struct Client {
    thread: Thread,
    pub server: Server,
    pub key: AccountKey,
    pub index: u64,
    pub ready: bool,
    pub error: Option<Error>,
    pub cache: Cache,
}

impl Client {
    /// Create a new client.
    pub fn new(
        address: String,
        key: AccountKey,
        account: AccountConnect,
        call: Option<CommandCall>,
    ) -> Self {
        let thread = Thread::new(address, account, call);

        Self {
            cache: Cache::new(thread.tx.clone()),
            thread,
            server: Server::default(),
            key,
            index: Default::default(),
            ready: Default::default(),
            error: Default::default(),
        }
    }

    /// Update the client state, polling for any new command from the server.
    pub fn update<F: FnMut(&mut Self, &CommandServer)>(
        &mut self,
        mut call: F,
    ) -> anyhow::Result<()> {
        while let Ok(command) = self.thread.rx.try_recv() {
            println!("[CLIENT::Update] {command:#?}");

            call(self, &command);

            self.cache.update(&command);

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
                _ => {}
            }
        }

        Ok(())
    }

    pub fn get_local_account(&mut self) -> Option<&Account> {
        self.cache.get_account(self.index)
    }

    pub fn send(&self, command: CommandClient) {
        if let Err(error) = self.thread.tx.send(command) {}
    }
}

//================================================================

pub type CommandCall = Box<dyn FnMut(CommandServer) + Send>;
pub type ThreadTX = tokio::sync::mpsc::UnboundedSender<CommandClient>;
pub type ThreadRX = tokio::sync::mpsc::UnboundedReceiver<CommandServer>;

struct Thread {
    tx: ThreadTX,
    rx: ThreadRX,
}

impl Thread {
    fn new(address: String, account: AccountConnect, mut call: Option<CommandCall>) -> Self {
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
                                        if let Some(call) = &mut call {
                                            call(command.clone());
                                        }

                                        tx_s.send(command).unwrap();
                                    }
                                    Err(err) => {
                                        tx_s.send(CommandServer::Leave);
                                        eprintln!("[CLIENT] read error: {err}");
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
