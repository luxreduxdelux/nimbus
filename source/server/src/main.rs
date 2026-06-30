use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use rand::RngCore;
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc::channel;

//================================================================

use common::prelude::*;

//================================================================

struct App {
    client: HashMap<u64, (Account, tokio::sync::mpsc::Sender<CommandServer>)>,
    server: Server,
    file: String,
    port: u32,
}

impl Default for App {
    fn default() -> Self {
        let mut file = "server.data".to_string();
        let mut port = 8080;
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
        }
    }
}

impl App {
    async fn send_all(&self, command: CommandServer) -> anyhow::Result<()> {
        for (_, (_, client)) in &self.client {
            client.send(command.clone()).await?;
        }

        Ok(())
    }
}

struct Connection {}

impl Connection {
    async fn new(
        mut handle: TcpStream,
        tx: tokio::sync::mpsc::Sender<(u64, CommandClient)>,
        mut rx: tokio::sync::mpsc::Receiver<CommandServer>,
        account: u64,
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listen = TcpListener::bind("0.0.0.0:8080").await?;
    let (a_tx, mut a_rx) = channel::<(u64, CommandClient)>(256);
    let app = Arc::new(Mutex::new(App::default()));
    let app_c = app.clone();

    /*
    println!(
        "Nimbus server up and running on port {}, on file \"{}\".",
        app_c.lock().await.port,
        app_c.lock().await.file
    );
    */

    tokio::spawn(async move {
        while let Some((index, command)) = a_rx.recv().await {
            println!("[SERVER::Loop] {command:#?}");

            match &command {
                CommandClient::Leave => {
                    let mut app = app_c.lock().await;
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
                    let message = Message::new(*channel, index, message.clone(), None);
                    let mut app = app_c.lock().await;
                    // TO-DO check if channel ID is valid
                    app.server.push_message(*channel, message.clone());
                    app.send_all(CommandServer::Message(message)).await;
                }
                CommandClient::MessageDelete(channel, message) => {
                    let mut app = app_c.lock().await;
                    app.server.delete_message(*channel, *message);
                    app.send_all(CommandServer::MessageDelete(*channel, *message))
                        .await;
                }
                CommandClient::PollVote(channel, message, choice) => {
                    let mut app = app_c.lock().await;
                    app.server.poll_vote(index, *channel, *message, *choice);
                    app.send_all(CommandServer::PollVote(index, *channel, *message, *choice))
                        .await;
                }
                CommandClient::AccountChannel(channel) => {
                    let mut app = app_c.lock().await;
                    app.server.set_account_channel(index, *channel);
                    app.send_all(CommandServer::AccountChannel(index, *channel))
                        .await;
                }
                CommandClient::AccountPresence(presence) => {
                    let mut app = app_c.lock().await;
                    app.server.set_account_presence(index, presence.clone());
                    app.send_all(CommandServer::AccountPresence(index, presence.clone()))
                        .await;
                }
                CommandClient::AccountState(state) => {
                    let mut app = app_c.lock().await;
                    app.server.set_account_state(index, state.clone());
                    app.send_all(CommandServer::AccountState(index, state.clone()))
                        .await;
                }
                CommandClient::AccountWrite(write) => {
                    let mut app = app_c.lock().await;
                    app.server.set_account_write(index, *write);
                    app.send_all(CommandServer::AccountWrite(index, *write))
                        .await;
                }
                _ => {}
            }
        }
    });

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Exiting...");
                break;
            }
            result = listen.accept() => {
                let (mut socket, _) = result?;

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
                                    if !account_connect.is_valid() {
                                        println!("invalid account connect");
                                        CommandServer::Error(Error::Account)
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

                                        // TO-DO send to EVERY other client that this client is now online
                                        CommandServer::Enter(account_i, app.server.clone())
                                            .write(&mut socket)
                                            .await;

                                        Connection::new(socket, a_tx, rx, account_i).await;
                                        break;
                                    }
                                },
                                CommandClient::Nonce(signature) => {
                                    let mut app = app.lock().await;

                                    if let Some((ref account_connect, challenge)) = key_challenge {
                                        let v_key = VerifyingKey::from_bytes(&account_connect.key).unwrap();
                                        let v_sig = Signature::from_bytes(signature.as_slice().try_into().unwrap());

                                        if v_key.verify_strict(&challenge, &v_sig).is_ok() {
                                            if let Some(account_i) = app.server.account_key.get(&account_connect.key).cloned() {
                                                let account_c = account_connect.clone().into_account(account_i);
                                                app.server.account.insert(account_i, account_c.clone());

                                                // TO-DO do I need to store account_c?
                                                app.client.insert(account_i, (account_c, tx));

                                                // TO-DO send to EVERY other client that this client is now online
                                                CommandServer::Enter(account_i, app.server.clone())
                                                    .write(&mut socket)
                                                    .await;

                                                Connection::new(socket, a_tx, rx, account_i).await;
                                                break;
                                            }
                                            else {
                                                CommandServer::Error(Error::Connect)
                                                    .write(&mut socket)
                                                    .await;
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                });
            }
        }
    }

    //command.await?;

    Ok(())
}
