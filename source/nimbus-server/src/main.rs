mod command;

//================================================================

use igd::Gateway;
use igd::{PortMappingProtocol, search_gateway};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

//================================================================

use crate::command::*;
use nimbus_protocol::prelude::*;

//================================================================

struct App {
    client: HashMap<AccountID, (Account, tokio::sync::mpsc::Sender<CommandServer>)>,
    storage: Storage,
    argument: Argument,
}

impl App {
    fn new(argument: Argument) -> Self {
        Self {
            client: Default::default(),
            storage: Storage::new(&argument.file).unwrap(),
            argument,
        }
    }
}

impl App {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    async fn send_all(&self, command: CommandServer) {
        for (_, (_, client)) in &self.client {
            client.send(command.clone()).await;
        }
    }

    async fn listen(
        app: Arc<Mutex<Self>>,
        socket: TcpListener,
        a_tx: Sender<(AccountID, CommandClient)>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("Exiting...");
                    break;
                }
                Ok((socket, _)) = socket.accept() => {
                    App::accept(app.clone(), socket, a_tx.clone());
                }
            }
        }

        Ok(())
    }

    fn accept(
        app: Arc<Mutex<Self>>,
        mut socket: TcpStream,
        a_tx: Sender<(AccountID, CommandClient)>,
    ) {
        let app = app.clone();
        let a_tx = a_tx.clone();
        let (tx, rx) = channel::<CommandServer>(128);

        println!("[SERVER::Accept] Connect: {socket:#?}");

        tokio::spawn(async move {
            loop {
                let command = CommandClient::read(&mut socket).await;

                println!("[SERVER::Accept] {command:#?}");

                if let Ok(command) = command {
                    match command {
                        CommandClient::Enter(account) => {
                            let app = app.lock().await;

                            if let Ok(Some(index)) = app.storage.get_account_key(account.key) {
                                CommandServer::Enter(
                                    index,
                                    Server::from_storage(&app.storage).unwrap(),
                                )
                                .write(&mut socket)
                                .await;

                                App::client(socket, a_tx, rx, index).await;
                                break;
                            } else {
                                let index = app.storage.count_account().unwrap();
                                app.storage.insert_account_key(account.key, index).unwrap();
                                app.storage
                                    .insert_account(account.into_account(index))
                                    .unwrap();

                                CommandServer::Enter(
                                    index,
                                    Server::from_storage(&app.storage).unwrap(),
                                )
                                .write(&mut socket)
                                .await;

                                App::client(socket, a_tx, rx, index).await;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    fn handle(app: Arc<Mutex<Self>>, mut rx: Receiver<(AccountID, CommandClient)>) {
        tokio::spawn(async move {
            while let Some((index, command)) = rx.recv().await {
                println!("[SERVER::Loop] {command:#?}");

                match &command {
                    CommandClient::Message(channel, message) => {
                        let app = app.lock().await;
                        let m_index = app.storage.count_account().unwrap();
                        let message =
                            Message::new(*channel, m_index, Some(index), message.clone(), None);
                        app.storage.insert_message(message.clone());
                        app.send_all(CommandServer::Message(message)).await;
                    }
                    CommandClient::MessageDelete(message) => {
                        let app = app.lock().await;
                        app.storage.remove_message(*message);
                        app.send_all(CommandServer::MessageDelete(*message)).await;
                    }
                    CommandClient::PollVote(message, choice) => {
                        let app = app.lock().await;
                        //app.server.poll_vote(index, *message, *choice);
                        //app.send_all(CommandServer::PollVote(index, *message, *choice))
                        //    .await;
                    }
                    CommandClient::AccountChannel(channel) => {
                        let app = app.lock().await;
                        app.storage.edit_account(index, |account| {
                            account.channel = *channel;
                        });
                        app.send_all(CommandServer::AccountChannel(index, *channel))
                            .await;
                    }
                    CommandClient::AccountPresence(presence) => {
                        let app = app.lock().await;
                        app.storage.edit_account(index, |account| {
                            account.presence = presence.clone();
                        });
                        app.send_all(CommandServer::AccountPresence(index, presence.clone()))
                            .await;
                    }
                    CommandClient::AccountState(state) => {
                        let app = app.lock().await;
                        app.storage.edit_account(index, |account| {
                            account.state = state.clone();
                        });
                        app.send_all(CommandServer::AccountState(index, state.clone()))
                            .await;
                    }
                    CommandClient::AccountWrite(write) => {
                        let app = app.lock().await;
                        app.storage.edit_account(index, |account| {
                            account.write = *write;
                        });
                        app.send_all(CommandServer::AccountWrite(index, *write))
                            .await;
                    }
                    _ => {}
                }
            }
        });
    }

    async fn client(
        mut handle: TcpStream,
        tx: tokio::sync::mpsc::Sender<(AccountID, CommandClient)>,
        mut rx: tokio::sync::mpsc::Receiver<CommandServer>,
        account: AccountID,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = CommandClient::read(&mut handle) => {
                        match result {
                            Ok(command) => {
                                tx.send((account, command)).await.unwrap();
                            }
                            Err(err) => {
                                eprintln!("read error: {err}");
                                tx.send((account, CommandClient::Leave)).await.unwrap();
                                break;
                            }
                        }
                    }
                    Some(command) = rx.recv() => {
                        //println!("[SERVER] wrote command: {command:?}");
                        command.write(&mut handle).await;
                    }
                }
            }
        });
    }

    async fn socket(app: Arc<Mutex<Self>>) -> anyhow::Result<TcpListener> {
        Ok(TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            app.lock().await.argument.port,
        ))
        .await?)
    }

    fn gateway() -> anyhow::Result<Gateway> {
        let gateway = search_gateway(Default::default())?;
        let address = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 6), 8080);
        gateway.add_port(PortMappingProtocol::TCP, 8080, address, 0, "Nimbus Server")?;
        Ok(gateway)
    }

    fn welcome(&self) {
        println!(
            "Welcome to Nimbus! Have a nimble day.\n  • Version: {}\n  • File: {}\n  • Port: {}\n  • uPnP status: {}",
            Self::VERSION,
            self.argument.file,
            self.argument.port,
            if self.argument.uPnP {
                "active"
            } else {
                "inactive"
            }
        );
    }
}

impl Drop for App {
    fn drop(&mut self) {
        /*
        if let Some(gateway) = &self.uPnP
            && gateway.remove_port(PortMappingProtocol::TCP, 8080).is_err()
        {
            println!("error: could not remove uPnP port.");
        }
        */
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    match Command::new() {
        Command::Run(argument) => {
            let app = Arc::new(Mutex::new(App::new(argument)));
            let socket = App::socket(app.clone()).await?;
            let (tx, rx) = channel::<(AccountID, CommandClient)>(256);

            app.lock().await.welcome();

            App::handle(app.clone(), rx);
            App::listen(app, socket, tx).await?;

            Ok(())
        }
        Command::Help => {
            println!("{}", Command::HELP_TEXT.replace("{version}", App::VERSION));

            Ok(())
        }
    }
}
