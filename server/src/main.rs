use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc::channel;

//================================================================

use common::prelude::*;

//================================================================

#[derive(Default)]
struct App {
    server: Server,
    client: HashMap<u64, (Account, tokio::sync::mpsc::Sender<CommandServer>)>,
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
    let listen = TcpListener::bind("127.0.0.1:8080").await?;
    let app = Arc::new(Mutex::new(App::default()));
    let app_c = app.clone();
    let (a_tx, mut a_rx) = channel::<(u64, CommandClient)>(256);

    let command = tokio::spawn(async move {
        while let Some((index, command)) = a_rx.recv().await {
            println!("[SERVER::Loop] {command:#?}");

            let account = app_c.lock().await.server.account[&index].clone();

            match &command {
                CommandClient::Leave => {
                    let mut app = app_c.lock().await;
                    app.client.remove(&index);

                    // TO-DO maybe should be CommandServer::AccountLeave?
                    app.server.set_account_state(index, AccountState::Offline);
                    app.server.set_account_write(index, false);
                    app.send_all(CommandServer::AccountState(index, AccountState::Offline))
                        .await;
                    app.send_all(CommandServer::AccountWrite(index, false))
                        .await;
                }
                CommandClient::Message(message) => {
                    let message = Message {
                        from: account.name,
                        kind: message.clone(),
                    };
                    let mut app = app_c.lock().await;
                    app.server.push_message(0, message.clone());
                    app.send_all(CommandServer::Message(message)).await;
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
                    loop {
                        let command = CommandClient::read(&mut socket).await;

                        println!("[SERVER] {command:#?}");

                        if let Ok(command) = command {
                            if let CommandClient::Enter(account) = command {
                                let mut lock = app.lock().await;
                                let find = lock.server.account.iter().position(|x| x.1.name == account.name);
                                let index = match find {
                                    Some(i) => {
                                        i as u64
                                    },
                                    None => {
                                        lock.server.push_account(account.clone());
                                        lock.server.count_account - 1
                                    }
                                };

                                lock.client.insert(index, (account.clone(), tx));

                                // TO-DO send to EVERY other client that this client is now online
                                CommandServer::Enter(lock.server.clone())
                                    .write(&mut socket)
                                    .await;

                                Connection::new(socket, a_tx, rx, index).await;

                                break;
                            }
                        }

                        /*
                        let n = match socket.read(&mut buf).await {
                            // socket closed
                            Ok(0) => return,
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("failed to read from socket; err = {:?}", e);
                                return;
                            }
                        };

                        // Write the data back
                        if let Err(e) = socket.write_all(&buf[0..n]).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return;
                        }
                        */
                    }
                });
            }
        }
    }

    //command.await?;

    Ok(())
}
