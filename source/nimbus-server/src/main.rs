use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use igd::Gateway;
use igd::{PortMappingProtocol, search_gateway};
use rand::RngCore;
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

//================================================================

use nimbus_common::prelude::*;

//================================================================

struct App {
    client: HashMap<AccountID, (Account, tokio::sync::mpsc::Sender<CommandServer>)>,
    server: Server,
    file: String,
    port: u16,
    uPnP: bool,
    verbose: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut file = "server.data".to_string();
        let mut port = 8080;
        let mut uPnP = false;
        let mut verbose = false;
        let mut list = std::env::args();
        list.next();

        while let Some(argument) = list.next() {
            match argument.as_str() {
                "--file" => {
                    if let Some(argument) = list.next() {
                        file = argument;
                    } else {
                        println!("missing argument \"{{file}}\" for command \"--file\".");
                    }
                }
                "--port" => {
                    if let Some(argument) = list.next() {
                        if let Ok(argument) = argument.parse() {
                            port = argument;
                        } else {
                            println!(
                                "invalid numerical argument \"{argument}\" for command \"--port\"."
                            );
                        }
                    } else {
                        println!("missing argument \"{{port}}\" for command \"--port\".");
                    }
                }
                "--uPnP" => {
                    uPnP = true;
                }
                "--verbose" => {
                    verbose = true;
                }
                x => {
                    println!("unknown argument \"{x}\".");
                }
            }
        }

        Self {
            client: Default::default(),
            server: Server::load(&file),
            file,
            port,
            uPnP,
            verbose,
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

        println!("[SERVER] Connect: {socket:#?}");

        tokio::spawn(async move {
            let mut key_challenge = None;

            loop {
                let command = CommandClient::read(&mut socket).await;

                println!("[SERVER] {command:#?}");

                if let Ok(command) = command {
                    match command {
                        CommandClient::Enter(account_connect) => {
                            if let Err(error) = account_connect.is_valid() {
                                CommandServer::Error(Error::Account(error))
                                    .write(&mut socket)
                                    .await;
                                break;
                            }

                            let mut app = app.lock().await;

                            if app.server.account_key.contains_key(&account_connect.key) {
                                let mut challenge = [0; 32];
                                OsRng.fill_bytes(&mut challenge);

                                key_challenge = Some((account_connect, challenge));

                                CommandServer::Nonce(challenge.to_vec())
                                    .write(&mut socket)
                                    .await;
                            } else {
                                // TO-DO always do nonce challenge, irrespective of whether this account key has already been in the server before
                                let account_i = app.server.count_account;
                                let account_c = account_connect.clone().into_account(account_i);
                                app.server.account_key.insert(account_c.key, account_i);
                                app.server.push_account(account_c.clone());

                                // TO-DO do I need to store account_c?
                                app.client.insert(account_i, (account_c, tx));

                                // TO-DO send this new account to every other client, THEN send this.
                                let message = Message::new(
                                    0,
                                    None,
                                    MessageKind::System(MessageSystem::Enter(0)),
                                    None,
                                );
                                app.server.push_message(0, message.clone());
                                app.send_all(CommandServer::Message(message)).await;

                                // TO-DO send to EVERY other client that this client is now online
                                CommandServer::Enter(account_i, app.server.clone())
                                    .write(&mut socket)
                                    .await;

                                App::client(socket, a_tx, rx, account_i).await;
                                break;
                            }
                        }
                        CommandClient::Nonce(signature) => {
                            let mut app = app.lock().await;

                            if let Some((ref account_connect, challenge)) = key_challenge {
                                let v_key = VerifyingKey::from_bytes(&account_connect.key).unwrap();
                                let v_sig =
                                    Signature::from_bytes(signature.as_slice().try_into().unwrap());

                                if v_key.verify_strict(&challenge, &v_sig).is_ok() {
                                    if let Some(account_i) =
                                        app.server.account_key.get(&account_connect.key).cloned()
                                    {
                                        let account_c =
                                            account_connect.clone().into_account(account_i);
                                        app.server.account.insert(account_i, account_c.clone());

                                        // TO-DO do I need to store account_c?
                                        app.client.insert(account_i, (account_c, tx));

                                        // TO-DO send this new account to every other client, THEN send this.
                                        let message = Message::new(
                                            0,
                                            None,
                                            MessageKind::System(MessageSystem::Enter(0)),
                                            None,
                                        );
                                        app.server.push_message(0, message.clone());
                                        app.send_all(CommandServer::Message(message)).await;

                                        // TO-DO send to EVERY other client that this client is now online
                                        CommandServer::Enter(account_i, app.server.clone())
                                            .write(&mut socket)
                                            .await;

                                        App::client(socket, a_tx, rx, account_i).await;
                                        break;
                                    } else {
                                        CommandServer::Error(Error::Connect(ConnectError::Nonce))
                                            .write(&mut socket)
                                            .await;
                                    }
                                }
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
                    CommandClient::Leave => {
                        let mut app = app.lock().await;
                        app.client.remove(&index);

                        // TO-DO maybe should be CommandServer::AccountLeave?
                        app.server
                            .set_account_presence(index, AccountPresence::Offline);
                        app.server.set_account_write(index, false);
                        app.send_all(CommandServer::AccountPresence(
                            index,
                            AccountPresence::Offline,
                        ))
                        .await;
                        app.send_all(CommandServer::AccountWrite(index, false))
                            .await;
                    }
                    CommandClient::Message(channel, message) => {
                        let message = Message::new(*channel, Some(index), message.clone(), None);
                        let mut app = app.lock().await;
                        app.server.push_message(*channel, message.clone());
                        app.send_all(CommandServer::Message(message)).await;
                    }
                    CommandClient::MessageReply(channel, message, content) => {
                        let message =
                            Message::new(*channel, Some(index), content.clone(), Some(*message));
                        let mut app = app.lock().await;
                        app.server.push_message(*channel, message.clone());
                        app.send_all(CommandServer::Message(message)).await;
                    }
                    CommandClient::MessageDelete(channel, message) => {
                        let mut app = app.lock().await;
                        app.server.delete_message(*channel, *message);
                        app.send_all(CommandServer::MessageDelete(*channel, *message))
                            .await;
                    }
                    CommandClient::PollVote(channel, message, choice) => {
                        let mut app = app.lock().await;
                        app.server.poll_vote(index, *channel, *message, *choice);
                        app.send_all(CommandServer::PollVote(index, *channel, *message, *choice))
                            .await;
                    }
                    CommandClient::AccountChannel(channel) => {
                        let mut app = app.lock().await;
                        app.server.set_account_channel(index, *channel);
                        app.send_all(CommandServer::AccountChannel(index, *channel))
                            .await;
                    }
                    CommandClient::AccountPresence(presence) => {
                        let mut app = app.lock().await;
                        app.server.set_account_presence(index, presence.clone());
                        app.send_all(CommandServer::AccountPresence(index, presence.clone()))
                            .await;
                    }
                    CommandClient::AccountState(state) => {
                        let mut app = app.lock().await;
                        app.server.set_account_state(index, state.clone());
                        app.send_all(CommandServer::AccountState(index, state.clone()))
                            .await;
                    }
                    CommandClient::AccountWrite(write) => {
                        let mut app = app.lock().await;
                        app.server.set_account_write(index, *write);
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
            app.lock().await.port,
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
            self.file,
            self.port,
            if self.uPnP { "active" } else { "inactive" }
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
    let app = Arc::new(Mutex::new(App::default()));
    let socket = App::socket(app.clone()).await?;
    let (tx, rx) = channel::<(AccountID, CommandClient)>(256);

    app.lock().await.welcome();

    App::handle(app.clone(), rx);
    App::listen(app, socket, tx).await?;

    Ok(())
}
